use std::env;

use crate::{
    lexer::{CursorPosition, HasRange},
    visitor::Visitor,
};

pub struct UndefinedExecCall<'a> {
    parser: &'a crate::parser::Parser,
    labels: Vec<Vec<u8>>,
    problems: Vec<(CursorPosition, CursorPosition)>,
}

impl<'a> UndefinedExecCall<'a> {
    pub fn new(parser: &'a crate::parser::Parser) -> Self {
        Self {
            parser,
            labels: Vec::new(),
            problems: Vec::new(),
        }
    }

    pub fn check(&mut self, callee: &[u8], start: CursorPosition, end: CursorPosition) {
        if !self.labels.iter().any(|l| l == callee) {
            self.problems.push((start, end));
        }
    }

    pub fn get_problems(&self) -> &Vec<(CursorPosition, CursorPosition)> {
        &self.problems
    }
}

impl<'a> Visitor for UndefinedExecCall<'a> {
    fn visit_macro(&mut self, expression: &crate::parser::Macro) {
        self.labels.push(self.parser.get_element_bytes(&expression.name).to_vec());
    }
    fn visit_exec(&mut self, exec_exp: &crate::parser::MadExec) {
        let callee = exec_exp.get_callee();
        let callee_str = self.parser.get_element_bytes(&callee);
        println!("Checking callee: {}", String::from_utf8_lossy(callee_str));
        println!("against: {:?}", self.labels);
        self.check(callee_str, exec_exp.get_range().0, exec_exp.get_range().1);
    }
    fn visit_label(&mut self, label: &crate::parser::Label) {
        println!("Checking label: {}", self.parser.get_element_str(&label.name));
        let label_str = self.parser.get_element_bytes(&label.name.get_range());
        if !self.labels.iter().any(|l| l == label_str) {
            self.labels.push(label_str.to_vec());
        }
    }

    fn visit_if(&mut self, if_exp: &crate::parser::If) {
        
    }

    fn visit_generic(&mut self, generic: &crate::parser::MadGeneric) {
        
    }
}
