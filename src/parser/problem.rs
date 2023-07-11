use std::fmt::Display;

use crate::lexer::CursorPosition;


#[derive(Debug)]
pub enum Problem {
    InvalidParam((CursorPosition, CursorPosition)),
    Error(String, CursorPosition, CursorPosition),
    Warning(String, CursorPosition, CursorPosition),
    Hint(String, CursorPosition, CursorPosition),
}

impl Display for Problem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Problem::InvalidParam(_) => write!(f, "Invalid Mad Parameter"),
            Problem::Error(_, _, _) => todo!(),
            Problem::Warning(_, _, _) => todo!(),
            Problem::Hint(_, _, _) => todo!(),
        }
    }
}

