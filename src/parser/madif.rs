use crate::{
    lexer::{CursorPosition, HasRange, Token},
    parser::{Assignment, Expression, Parser},
};

#[derive(Debug, PartialEq, Default)]
pub struct If {
    pub parenopen: CursorPosition,
    pub parenclose: CursorPosition,
    pub condition: Vec<Expression>, // should be only one
    pub body: Vec<Expression>,
    pub end: CursorPosition,
}

impl If {
    pub fn parse(parser: &mut Parser) -> Option<Self> {
        let before = parser.get_position();

        let maybe_if = Self::parse_inner(parser);

        if maybe_if.is_some() {
            return maybe_if;
        }

        parser.set_position(before);
        None
    }

    pub fn parse_inner(parser: &mut Parser) -> Option<Self> {
        if let Some(Token::Ident(if_keyword)) = parser.peek_token() {
            if parser.lexer.compare_range(if_keyword, b"if") {
                parser.advance();
            } else {
                return None;
            }
        } else {
            return None;
        }

        let mut if_object = If::default();

        if let Some(Token::ParentOpen(pos)) = parser.next_token() {
            if_object.parenopen = pos.clone();
        } else {
            return None;
        }
        while let Some(expr) = Assignment::parse(parser) {
            if let Expression::TokenExp(Token::ParentClose(end)) = expr {
                if_object.parenclose = end + 1;
                break;
            }
            if_object.condition.push(expr);
        }

        if let Some(Token::BraceOpen(_)) = parser.peek_token() {
            parser.advance();
        } else {
            return None;
        }

        while let Some(expr) = Assignment::parse(parser) {
            if let Expression::TokenExp(Token::BraceClose(end)) = expr {
                if_object.end = end + 1;
                break;
            }
            if_object.body.push(expr);
        }

        Some(if_object)
    }
}

impl HasRange for If {
    fn get_range(&self) -> (CursorPosition, CursorPosition) {
        (self.end, self.end)
    }
}
