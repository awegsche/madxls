use std::fmt::Write;

use crate::{lexer::Token, parser::{Label, Macro, MadExec}};

pub trait Visitor {
    fn visit_macro(&mut self, macro_exp: &Macro);
    fn visit_exec(&mut self, exec_exp: &MadExec);
    fn visit_label(&mut self, label: &Label);
    fn visit_if(&mut self, if_exp: &crate::parser::If);
    fn visit_generic(&mut self, generic: &crate::parser::MadGeneric);
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
    fn visit_macro(&mut self, macro_exp: &Macro) {
        writeln!(self.buffer, "macro {} = {{", self.parser.get_element_str(&macro_exp.name)).unwrap();
    }

    fn visit_exec(&mut self, exec_exp: &MadExec) {
        writeln!(self.buffer, "exec {}", self.parser.get_element_str(&exec_exp.get_callee())).unwrap();
    }

    fn visit_label(&mut self, label: &Label) {
        writeln!(self.buffer, "label {}", self.parser.get_element_str(label)).unwrap();
    }

    fn visit_if(&mut self, if_exp: &crate::parser::If) {
        writeln!(self.buffer, "if").unwrap();
    }
    fn visit_generic(&mut self, generic: &crate::parser::MadGeneric) {
        print_token_exp(&generic.name, self);
    }
}
