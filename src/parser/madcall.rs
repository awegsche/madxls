use super::Expression;
use crate::lexer::{CursorPosition, HasRange, Token};

#[derive(Debug, PartialEq, Default)]
pub struct MadCall {
    pub name: Token,                       // usually "call"
    pub file_kw: Token,                    // usually "file"
    pub eq: Option<Token>,                 // '=' token, optional for robustness
    pub filename: Option<Box<Expression>>, // the filename expression (can be string, ident, etc.)
    pub range: (CursorPosition, CursorPosition),
}

impl MadCall {
    pub(crate) fn parse(parser: &mut super::Parser) -> Option<MadCall> {
        let start = parser
            .peek_token()
            .map(|t| t.get_range().0)
            .unwrap_or_default();

        // Expect "call"
        let name = parser.peek_token()?.clone();
        if !parser.lexer.compare_range(&name, b"call") {
            return None;
        }
        parser.advance();

        // Expect comma
        if let Some(Token::Komma(_)) = parser.peek_token() {
            parser.advance();
        } else {
            return Some(MadCall {
                name,
                file_kw: Token::default(),
                eq: None,
                filename: None,
                range: (start, start),
            });
        }

        // Expect "file"
        let file_kw = parser.peek_token()?.clone();
        let range_end = file_kw.get_range().1;
        if !parser.lexer.compare_range(&file_kw, b"file") {
            return Some(MadCall {
                name,
                file_kw,
                eq: None,
                filename: None,
                range: (start, range_end),
            });
        }
        parser.advance();

        // Optional '='
        let mut eq = None;
        if let Some(Token::Equal(_)) = parser.peek_token() {
            eq = parser.peek_token().cloned();
            parser.advance();
        }

        // Parse filename expression
        let filename = if let Some(expr) = Expression::parse(parser) {
            Some(Box::new(expr))
        } else {
            None
        };

        // End position
        let end = filename
            .as_ref()
            .map(|e| e.get_range().1)
            .or_else(|| eq.as_ref().map(|t| t.get_range().1))
            .unwrap_or_else(|| file_kw.get_range().1);

        Some(MadCall {
            name,
            file_kw,
            eq,
            filename,
            range: (start, end),
        })
    }

    pub(crate) fn accept<V: crate::visitor::Visitor>(
        &self,
        visitor: &mut V,
        parser: &crate::parser::Parser,
    ) {
        visitor.visit_call(self, parser);
        if let Some(filename) = &self.filename {
            filename.accept(visitor, parser);

            if let Some(filename) = self.get_filename(parser) {
                visitor.visit_subparser(&filename, parser);
            }
        }
    }

    pub fn get_filename(&self, parser: &crate::parser::Parser) -> Option<String> {
        match self.filename.as_ref()?.as_ref() {
            Expression::String(s) => Some(parser.get_element_str(&(s.0 + 1, s.1))),
            Expression::TokenExp(t) => Some(parser.get_element_str(t)),
            _ => None,
        }
    }
}

impl HasRange for MadCall {
    fn get_range(&self) -> (CursorPosition, CursorPosition) {
        self.range
    }
}
