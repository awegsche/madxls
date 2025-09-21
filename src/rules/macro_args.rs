use std::fmt::Display;

use crate::{lexer::CursorPosition, parser::Parser, visitor::Visitor};

#[derive(Debug, Clone)]
pub enum MacroArgProblem {
    TooShort(tower_lsp::lsp_types::Range),
    Unused(tower_lsp::lsp_types::Range),
}

pub struct MacroArgs {
    problems: Vec<MacroArgProblem>,
}

impl MacroArgs {
    pub fn new(parser: &Parser) -> Self {
        let mut new_visitor = Self {
            problems: Vec::new(),
        };

        new_visitor.visit_parser(parser);
        new_visitor
    }

    pub fn get_problems(&self) -> &Vec<MacroArgProblem> {
        &self.problems
    }
}

impl Visitor for MacroArgs {
    fn visit_macro(&mut self, macro_exp: &crate::parser::Macro, parser: &Parser) {
        let mut macro_body = String::new();
        for expr in macro_exp.body.iter().filter(|e| match e {
            crate::parser::Expression::Comment(_) => false,
            _ => true,
        }) {
            macro_body.push_str(&parser.get_element_str(expr));
            macro_body.push(' '); // Add space to separate expressions
        }

        for arg in macro_exp.args.iter() {
            let range = parser.lexer.cursor_range_to_text_range(arg);
            let arg_str = parser.get_element_str(arg);

            if range.end.character - range.start.character < 4 {
                self.problems.push(MacroArgProblem::TooShort(range));
            }

            // Check if argument appears verbatim in the macro body
            if !macro_body.contains(&arg_str) {
                self.problems.push(MacroArgProblem::Unused(range));
            }
        }
    }
    fn visit_exec(&mut self, _: &crate::parser::MadExec, _parser: &Parser) {}

    fn visit_label(&mut self, _: &crate::parser::Label, _parser: &Parser) {}

    fn visit_if(&mut self, _: &crate::parser::If, _parser: &Parser) {}

    fn visit_generic(&mut self, _: &crate::parser::MadGeneric, _parser: &Parser) {}

    fn visit_call(&mut self, _: &crate::parser::MadCall, _parser: &Parser) {}

    fn visit_macro_end(&mut self, _: &crate::parser::Macro, _parser: &Parser) {}

    fn visit_if_end(&mut self, _: &crate::parser::If, _parser: &Parser) {}

    fn visit_parser(&mut self, parser: &Parser) {
        for e in parser.get_elements() {
            e.accept(self, parser);
        }
    }

    fn visit_assignment_lhs(&mut self, _: &crate::parser::Expression, _parser: &Parser) {}

    fn visit_comment(&mut self, _: &(CursorPosition, CursorPosition), _parser: &Parser) {}

    fn visit_string(&mut self, _: &(CursorPosition, CursorPosition), _parser: &Parser) {}

    fn visit_subparser(&mut self, _: &str, _: &crate::parser::Parser) {}
}

impl Display for MacroArgProblem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MacroArgProblem::TooShort(range) => {
                write!(
                    f,
                    "MacroArgTooShort | ({}, {}) -- ({}, {})",
                    range.start.line, range.start.character, range.end.line, range.end.character,
                )
            }
            MacroArgProblem::Unused(range) => {
                write!(
                    f,
                    "MacroArgUnused | ({}, {}) -- ({}, {})",
                    range.start.line, range.start.character, range.end.line, range.end.character,
                )
            }
        }
    }
}
