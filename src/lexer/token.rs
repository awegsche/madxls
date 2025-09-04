use std::fmt::Display;

use super::{CursorPosition, HasRange};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    BraceOpen(CursorPosition),
    BraceClose(CursorPosition),
    ParentOpen(CursorPosition),
    ParentClose(CursorPosition),
    Ident((CursorPosition, CursorPosition)),
    Number((CursorPosition, CursorPosition)),
    Operator(CursorPosition),
    Equal(CursorPosition),
    DoubleEqual(CursorPosition),
    ColonEqual(CursorPosition),
    Dot(CursorPosition),
    SemiColon(CursorPosition),
    Colon(CursorPosition),
    Komma(CursorPosition),
    Quotes(CursorPosition),
    DoubleQuotes(CursorPosition),
    Comment((CursorPosition, CursorPosition)),
    MultilineComment(Vec<(CursorPosition, CursorPosition)>),
    Char(CursorPosition),
    EOF,
}

impl HasRange for Token {
    fn get_range(&self) -> (CursorPosition, CursorPosition) {
        match self {
            Token::BraceOpen(p) => (*p, p + 1),
            Token::BraceClose(p) => (*p, p + 1),
            Token::ParentOpen(p) => (*p, p + 1),
            Token::ParentClose(p) => (*p, p + 1),
            Token::Ident((s, e)) => (*s, *e),
            Token::Number((s, e)) => (*s, *e),
            Token::Operator(p) => (*p, p + 1),
            Token::Equal(p) => (*p, p + 1),
            Token::ColonEqual(p) => (*p, p + 2),
            Token::Dot(p) => (*p, p + 1),
            Token::SemiColon(p) => (*p, p + 1),
            Token::Colon(p) => (*p, p + 1),
            Token::Komma(p) => (*p, p + 1),
            Token::Quotes(p) => (*p, p + 1),
            Token::DoubleQuotes(p) => (*p, p + 1),
            Token::Comment((s, e)) => (*s, *e),
            Token::MultilineComment(v) => (v[0].0, v.last().unwrap().1),
            Token::Char(p) => (*p, p + 1),
            Token::EOF => (CursorPosition::default(), CursorPosition::default()),
            Token::DoubleEqual(p) => (*p, p + 2),
        }
    }
}

impl Default for Token {
    fn default() -> Self {
        Token::EOF
    }
}

impl Token {
    pub fn is_eof(&self) -> bool {
        matches!(self, Token::EOF)
    }

    pub fn is_operator(&self) -> bool {
        matches!(self, Token::Operator(_))
    }

    pub fn is_colon(&self) -> bool {
        matches!(self, Token::Colon(_))
    }

    pub fn is_ident(&self) -> bool {
        matches!(self, Token::Ident(_))
    }

    pub fn is_assignment(&self) -> bool {
        match self {
            Token::Equal(_) => true,
            Token::ColonEqual(_) => true,
            _ => false,
        }
    }
}
