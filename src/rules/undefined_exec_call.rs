use std::collections::HashSet;

use crate::{lexer::CursorPosition, parser::Parser, visitor::Visitor};

pub struct UndefinedExecCall<'a> {
    labels: &'a HashSet<Vec<u8>>,
    problems: Vec<(CursorPosition, CursorPosition)>,
}

impl<'a> UndefinedExecCall<'a> {
    pub fn new(parser: &'a Parser, labels: &'a HashSet<Vec<u8>>) -> Self {
        let mut new_visitor = Self {
            labels,
            problems: Vec::new(),
        };

        new_visitor.visit_parser(parser);
        new_visitor
    }

    pub fn check(&mut self, callee: &[u8], start: CursorPosition, end: CursorPosition) {
        if !self.labels.contains(callee) {
            self.problems.push((start, end));
        }
    }

    pub fn get_problems(&self) -> &Vec<(CursorPosition, CursorPosition)> {
        &self.problems
    }
}

impl<'a> Visitor for UndefinedExecCall<'a> {
    fn visit_macro(&mut self, _: &crate::parser::Macro, _parser: &Parser) {}
    fn visit_exec(&mut self, exec_exp: &crate::parser::MadExec, parser: &Parser) {
        let callee = exec_exp.get_callee();
        let callee_str = parser.get_element_bytes(&callee);
        self.check(callee_str, callee.0, callee.1);
    }
    fn visit_label(&mut self, _: &crate::parser::Label, _parser: &Parser) {}

    fn visit_if(&mut self, _: &crate::parser::If, _parser: &Parser) {}

    fn visit_generic(&mut self, _: &crate::parser::MadGeneric, _parser: &Parser) {}

    fn visit_call(&mut self, _: &crate::parser::MadCall, _parser: &Parser) {}

    fn visit_macro_end(&mut self, _: &crate::parser::Macro, _parser: &Parser) {}

    fn visit_if_end(&mut self, _: &crate::parser::If, _parser: &Parser) {}

    fn visit_assignment_lhs(&mut self, _: &crate::parser::Expression, _parser: &Parser) {}

    fn visit_comment(&mut self, _: &(CursorPosition, CursorPosition), _parser: &Parser) {}

    fn visit_string(&mut self, _: &(CursorPosition, CursorPosition), _parser: &Parser) {}

    fn visit_subparser(&mut self, _: &str, _: &crate::parser::Parser) {}
}
