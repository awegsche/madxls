use std::fmt::Display;

use tower_lsp::lsp_types::Range;

use crate::lexer::CursorPosition;

#[derive(Debug, Clone)]
pub struct MaybeProblem {
    pub problem: Option<Problem>,
    pub range: Range
}

#[derive(Debug, Clone)]
pub enum Problem {
    MissingCallee(Vec<u8>, (CursorPosition, CursorPosition)),
    InvalidParam((CursorPosition, CursorPosition)),
    Error(String, CursorPosition, CursorPosition),
    Warning(String, CursorPosition, CursorPosition),
    Hint(String, CursorPosition, CursorPosition),
}

impl Display for Problem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Problem::MissingCallee(_,_) => write!(f, "Missing Macro, check includes"),
            Problem::InvalidParam(_) => write!(f, "Invalid Mad Parameter"),
            Problem::Error(_, _, _) => todo!(),
            Problem::Warning(_, _, _) => todo!(),
            Problem::Hint(_, _, _) => todo!(),
        }
    }
}

