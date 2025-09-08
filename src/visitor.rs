use std::fmt::Write;

use crate::{lexer::Token, parser::Expression};

pub trait Visitor {
    fn visit(&mut self, expression: &Expression);
}

pub struct PrintVisitor<'a> {
    parser: &'a crate::parser::Parser,
    indent: usize,
    pub buffer: String,
}

impl<'a> PrintVisitor<'a> {
    pub fn new(parser: &'a crate::parser::Parser) -> Self {
        Self {
            parser,
            indent: 0,
            buffer: String::new(),
        }
    }
}

fn print_token_exp(t: &Token, visitor: &mut PrintVisitor) {
    match t {
        Token::SemiColon(_) => {
            writeln!(visitor.buffer, ";").unwrap();
        }
        _ => {}
    }
}

impl<'a> Visitor for PrintVisitor<'a> {
    fn visit(&mut self, expression: &Expression) {
        write!(self.buffer, "{:indent$}", "", indent = self.indent).unwrap();
        match expression {
            Expression::Label(l) => write!(
                self.buffer,
                "Visiting Label: {}",
                self.parser.get_element_str(l)
            )
            .unwrap(),
            Expression::Assignment(a) => write!(
                self.buffer,
                "Visiting Assignment: {}",
                self.parser.get_element_str(a)
            )
            .unwrap(),
            Expression::Macro(m) => {
                writeln!(
                    self.buffer,
                    "Visiting Macro: {}",
                    self.parser.get_element_str(&m.name)
                )
                .unwrap();
                self.indent += 2;
                for e in &m.body {
                    self.visit(e);
                }
                self.indent -= 2;
            }
            Expression::TokenExp(t) => print_token_exp(t, self),
            _ => write!(
                self.buffer,
                "Visiting Expression: {:?} {}",
                expression,
                self.parser.get_element_str(expression)
            )
            .unwrap(),
        }
    }
}
