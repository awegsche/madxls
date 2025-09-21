use std::fmt::Display;

use tower_lsp::lsp_types;

use crate::lexer::CursorPosition;

#[derive(Debug, Clone)]
pub enum Problem {
    MissingCallee(Vec<u8>, (CursorPosition, CursorPosition)),
    InvalidParam((CursorPosition, CursorPosition)),
    Error(String, CursorPosition, CursorPosition),
    Warning(String, CursorPosition, CursorPosition),
    Hint(String, CursorPosition, CursorPosition),
}

impl Problem {
    pub fn to_diagnostic(&self, parser: &crate::parser::Parser) -> lsp_types::Diagnostic {
        use lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

        let severity = match self {
            Problem::MissingCallee(_, _) => DiagnosticSeverity::ERROR,
            Problem::InvalidParam(_) => DiagnosticSeverity::ERROR,
            Problem::Error(_, _, _) => DiagnosticSeverity::ERROR,
            Problem::Warning(_, _, _) => DiagnosticSeverity::WARNING,
            Problem::Hint(_, _, _) => DiagnosticSeverity::HINT,
        };

        let (start, end) = match self {
            Problem::MissingCallee(_, range) => *range,
            Problem::InvalidParam(range) => *range,
            Problem::Error(_, start, end) => (*start, *end),
            Problem::Warning(_, start, end) => (*start, *end),
            Problem::Hint(_, start, end) => (*start, *end),
        };

        let message = match self {
            Problem::MissingCallee(_, _) => "Missing Macro, check includes".to_string(),
            Problem::InvalidParam(_) => "Invalid Mad Parameter".to_string(),
            Problem::Error(msg, _, _) => msg.clone(),
            Problem::Warning(msg, _, _) => msg.clone(),
            Problem::Hint(msg, _, _) => msg.clone(),
        };

        let (start, end) = (
            parser.lexer.cursor_pos_to_text_pos(start),
            parser.lexer.cursor_pos_to_text_pos(end),
        );

        Diagnostic {
            range: Range {
                start: Position {
                    line: start.line as u32,
                    character: start.character as u32,
                },
                end: Position {
                    line: end.line as u32,
                    character: end.character as u32,
                },
            },
            severity: Some(severity),
            code: None,
            code_description: None,
            source: Some("madxls".to_string()),
            message,
            related_information: None,
            tags: None,
            data: None,
        }
    }
}

impl Display for Problem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Problem::MissingCallee(_, _) => write!(f, "Missing Macro, check includes"),
            Problem::InvalidParam(_) => write!(f, "Invalid Mad Parameter"),
            Problem::Error(_, _, _) => todo!(),
            Problem::Warning(_, _, _) => todo!(),
            Problem::Hint(_, _, _) => todo!(),
        }
    }
}
