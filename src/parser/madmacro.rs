use crate::lexer::{CursorPosition, HasRange, Token};

use super::{Assignment, Expression, Parser};

#[derive(Debug, PartialEq, Default)]
pub struct Macro {
    pub name: Token,
    pub parenopen: CursorPosition,
    pub args: Vec<Token>,
    pub parenclose: CursorPosition,
    pub macro_pos: Token,
    pub body: Vec<Expression>,
    pub end: CursorPosition,
}

impl Macro {
    pub fn parse(parser: &mut Parser) -> Option<Self> {
        let before = parser.get_position();

        if let Some(m) = Self::parse_inner(parser) {
            return Some(m);
        }
        parser.set_position(before);
        None
    }

    pub fn parse_inner(parser: &mut Parser) -> Option<Self> {
        if let Some(token) = parser.peek_token() {
            if !token.is_ident() {
                return None;
            }
            let mut m = Self::default();
            m.name = token.clone();
            parser.advance();

            if let Some((parenopen, tokens, parenclose)) = Self::read_parenthesis(parser) {
                m.parenopen = parenopen;
                m.args = tokens;
                m.parenclose = parenclose;
            } else {
                return None;
            }

            if let Some(Token::Colon(_)) = parser.peek_token() {
                parser.advance();
            } else {
                return None;
            }

            if let Some(Token::Ident(macro_name)) = parser.peek_token() {
                if parser.lexer.compare_range(macro_name, b"macro") {
                    m.macro_pos = Token::Ident(macro_name.clone());
                    parser.advance();
                } else {
                    return None;
                }
            } else {
                return None;
            }

            if let Some(Token::Equal(_)) = parser.peek_token() {
                parser.advance();
            } else {
                return None;
            }

            if let Some(Token::BraceOpen(_)) = parser.peek_token() {
                parser.advance();
            } else {
                return None;
            }

            while let Some(expr) = Assignment::parse(parser) {
                if let Expression::TokenExp(Token::BraceClose(end)) = expr {
                    m.end = end + 1;
                    break;
                }
                m.body.push(expr);
            }

            return Some(m);
        }
        None
    }

    pub fn read_parenthesis(
        parser: &mut Parser,
    ) -> Option<(CursorPosition, Vec<Token>, CursorPosition)> {
        let mut tokens = Vec::new();

        let mut start = CursorPosition::default();
        if let Some(Token::ParentOpen(parenopen)) = parser.peek_token() {
            start = *parenopen;
            parser.advance();
        } else {
            return None;
        }

        while let Some(token) = parser.peek_token().cloned() {
            parser.advance();
            match token {
                Token::ParentClose(parenclose) => {
                    let end = parenclose;

                    return Some((start, tokens, end));
                }
                Token::Ident(ident) => {
                    tokens.push(Token::Ident(ident));
                }
                _ => {}
            }
        }

        // if we get here, there was no close parenthesis
        // and we reached EOF
        //
        None
    }

    pub(crate) fn accept<V: crate::visitor::Visitor>(
        &self,
        visitor: &mut V,
        parser: &crate::parser::Parser,
    ) {
        visitor.visit_macro(self, parser);
        for e in self.body.iter() {
            e.accept(visitor, parser);
        }
        visitor.visit_macro_end(self, parser);
    }
}

impl HasRange for Macro {
    fn get_range(&self) -> (CursorPosition, CursorPosition) {
        (self.name.get_range().0, self.end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn match_macro() {
        let parser = Parser::from_str("m1(a, b): macro = {\n twiss,sequence=lhcb1;\na=b;\n}");

        let m = &parser.get_elements()[0];

        if let Expression::Macro(m) = m {
            assert_eq!(parser.get_element_str(&m.name), "m1");
            assert_eq!(
                parser.get_element_str(m),
                "m1(a, b): macro = {\n twiss,sequence=lhcb1;\na=b;\n}"
            );

            //assert!(false, "m: {:?}\n\n element after:\n{:?}", m, parser.get_elements()[1]);
        } else {
            assert!(false, "should be macro");
        }
        assert_eq!(parser.get_elements().len(), 1);
    }
}
