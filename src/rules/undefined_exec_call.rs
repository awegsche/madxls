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

use crate::parser::Expression;
impl<'a> Visitor for UndefinedExecCall<'a> {
    fn visit(&mut self, expression: &crate::parser::Expression) {
        match expression {
            Expression::Exec(exec) => {
                let callee = exec.get_callee();
                let callee_str = self.parser.get_element_bytes(&callee);
                println!("Checking callee: {}", String::from_utf8_lossy(callee_str));
                self.check(callee_str, exec.get_range().0, exec.get_range().1);
            }
            Expression::Label(label) => {
                let label_str = self.parser.get_element_bytes(&label.name.get_range());
                if !self.labels.iter().any(|l| l == label_str) {
                    self.labels.push(label_str.to_vec());
                }
            }
            Expression::Macro(macro_exp) => {
                self.labels
                    .push(self.parser.get_element_bytes(&macro_exp.name).to_vec());

                for e in &macro_exp.body {
                    self.visit(e);
                }
            }
            Expression::MadEnvironment(env) => {
                for e in env.expressions.iter() {
                    self.visit(e);
                }
            }
            _ => {
                println!(
                    "Ignoring expression: {}",
                    self.parser.get_element_str(expression)
                );
            }
        }
    }
}
