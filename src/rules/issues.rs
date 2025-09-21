use std::io::Write;

use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

use crate::{
    lexer::print_range,
    parser::Parser,
    rules::{
        collect_labels::CollectLabels,
        macro_args::{MacroArgProblem, MacroArgs},
        macro_variables::{MacroVarProblem, MacroVars},
        undefined_exec_call::{UndefinedExecCall, UndefinedExecCallProblem},
    },
};

pub struct Issues {
    missing_files: Vec<Range>,
    macro_args: Vec<MacroArgProblem>,
    macro_variables: Vec<MacroVarProblem>,
    undefined_exec_call: Vec<UndefinedExecCallProblem>,
}

impl Issues {
    pub fn new() -> Self {
        Self {
            missing_files: Vec::new(),
            macro_args: Vec::new(),
            macro_variables: Vec::new(),
            undefined_exec_call: Vec::new(),
        }
    }

    pub fn from_parser(parser: &Parser) -> Self {
        for mf in parser.get_missing_files().iter() {
            println!("MissingFile | {}", print_range(mf));
        }

        let collect_labels = CollectLabels::new(&parser);
        let missing_callee = UndefinedExecCall::new(&parser, collect_labels.get_labels());
        let macro_args = MacroArgs::new(&parser);
        let macro_vars = MacroVars::new(&parser);

        Self {
            missing_files: parser.get_missing_files().clone(),
            macro_args: macro_args.get_problems().clone(),
            macro_variables: macro_vars.get_problems().clone(),
            undefined_exec_call: missing_callee.get_problems().clone(),
        }
    }

    pub fn print_problems<W: Write>(&self, w: &mut W) -> Result<(), std::io::Error> {
        for problem in self.macro_args.iter() {
            writeln!(w, "{}", problem)?;
        }
        for problem in self.macro_variables.iter() {
            writeln!(w, "{}", problem)?;
        }
        for problem in self.undefined_exec_call.iter() {
            writeln!(w, "{}", problem)?;
        }
        for problem in self.missing_files.iter() {
            writeln!(w, "{}", print_range(problem))?;
        }
        Ok(())
    }

    fn range_to_diagnostic_range(range: Range) -> Range {
        Range {
            start: Position {
                line: range.start.line - 1,
                character: range.start.character,
            },
            end: Position {
                line: range.end.line - 1,
                character: range.end.character,
            },
        }
    }

    pub fn to_diagnostics(&self) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        for problem in self.macro_args.iter() {
            let (range, message) = match problem {
                MacroArgProblem::TooShort(range) => (*range, "macro arg too short".to_string()),
                MacroArgProblem::Unused(range) => (*range, "macro arg unused".to_string()),
            };
            let diagnostic = Diagnostic {
                range: Self::range_to_diagnostic_range(range),
                severity: Some(DiagnosticSeverity::HINT),
                code: None,
                code_description: None,
                source: Some("madxls".to_string()),
                message,
                related_information: None,
                tags: None,
                data: None,
            };
            diagnostics.push(diagnostic);
        }
        for problem in self.macro_variables.iter() {
            let (range, message) = match problem {
                MacroVarProblem::ExistWhileDef(range, _) => {
                    (*range, "macro var exist while def".to_string())
                }
                MacroVarProblem::ExistsWhileCall(range, _) => {
                    (*range, "macro var exist while call".to_string())
                }
            };
            let diagnostic = Diagnostic {
                range: Self::range_to_diagnostic_range(range),
                severity: Some(DiagnosticSeverity::WARNING),
                code: None,
                code_description: None,
                source: Some("madxls".to_string()),
                message,
                related_information: None,
                tags: None,
                data: None,
            };
            diagnostics.push(diagnostic);
        }
        for problem in self.undefined_exec_call.iter() {
            let (range, message) = match problem {
                UndefinedExecCallProblem::UndefinedExecCall(range) => {
                    (*range, "undefined exec call".to_string())
                }
            };
            let diagnostic = Diagnostic {
                range: Self::range_to_diagnostic_range(range),
                severity: Some(DiagnosticSeverity::ERROR),
                code: None,
                code_description: None,
                source: Some("madxls".to_string()),
                message,
                related_information: None,
                tags: None,
                data: None,
            };
            diagnostics.push(diagnostic);
        }
        diagnostics
    }
}
