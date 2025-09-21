use std::{collections::HashMap, fmt::Display};

use tower_lsp::lsp_types::Range;

use crate::{
    lexer::{print_range, CursorPosition},
    parser::{self, Expression, Parser},
    visitor::Visitor,
};

#[derive(Debug, Clone)]
pub enum MacroVarProblem {
    ExistWhileDef(Range, Range),
    ExistsWhileCall(Range, Range),
}

pub struct MacroVars {
    problems: Vec<MacroVarProblem>,
    labels: HashMap<Vec<u8>, Range>,
}

impl MacroVars {
    pub fn new(parser: &Parser) -> Self {
        let mut new_visitor = Self {
            problems: Vec::new(),
            labels: HashMap::new(),
        };

        new_visitor.visit_parser(parser);
        new_visitor
    }

    pub fn get_problems(&self) -> &Vec<MacroVarProblem> {
        &self.problems
    }
}

impl Visitor for MacroVars {
    fn visit_macro(&mut self, macro_exp: &parser::Macro, parser: &Parser) {
        for lhs in macro_exp.body.iter().filter_map(|e| match e {
            Expression::Assignment(ass) => Some(ass.lhs.as_ref()),
            _ => None,
        }) {
            if let Some(location) = self.labels.get(parser.get_element_bytes(lhs)) {
                self.problems.push(MacroVarProblem::ExistWhileDef(
                    parser.lexer.cursor_range_to_text_range(lhs),
                    *location,
                ));
            }
        }
    }
    fn visit_exec(&mut self, _: &parser::MadExec, _parser: &Parser) {}

    fn visit_label(&mut self, _: &parser::Label, _parser: &Parser) {}

    fn visit_if(&mut self, _: &parser::If, _parser: &Parser) {}

    fn visit_generic(&mut self, _: &parser::MadGeneric, _parser: &Parser) {}

    fn visit_call(&mut self, _: &parser::MadCall, _parser: &Parser) {}

    fn visit_macro_end(&mut self, _: &parser::Macro, _parser: &Parser) {}

    fn visit_if_end(&mut self, _: &parser::If, _parser: &Parser) {}

    fn visit_assignment_lhs(&mut self, lhs: &Expression, parser: &Parser) {
        self.labels.insert(
            parser.get_element_bytes(lhs).to_vec(),
            parser.lexer.cursor_range_to_text_range(lhs),
        );
    }

    fn visit_comment(&mut self, _: &(CursorPosition, CursorPosition), _parser: &Parser) {}

    fn visit_string(&mut self, _: &(CursorPosition, CursorPosition), _parser: &Parser) {}

    fn visit_subparser(&mut self, _: &str, _: &crate::parser::Parser) {}
}

impl Display for MacroVarProblem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MacroVarProblem::ExistWhileDef(range, orig) => {
                write!(
                    f,
                    "MacroVarExistDef | {} | {}",
                    print_range(range),
                    print_range(orig),
                )
            }
            MacroVarProblem::ExistsWhileCall(range, orig) => {
                write!(
                    f,
                    "MacroVarExistCall | {} | {}",
                    print_range(range),
                    print_range(orig),
                )
            }
        }
    }
}
