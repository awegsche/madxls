use std::fmt::Write;

use crate::{
    lexer::{CursorPosition, HasRange, Token},
    parser::{Label, Macro, MadExec, Parser},
};

pub trait Visitor: Sized {
    fn visit_macro(&mut self, macro_exp: &Macro, parser: &Parser);
    fn visit_macro_end(&mut self, macro_exp: &Macro, parser: &Parser);
    fn visit_exec(&mut self, exec_exp: &MadExec, parser: &Parser);
    fn visit_label(&mut self, label: &Label, parser: &Parser);
    fn visit_if(&mut self, if_exp: &crate::parser::If, parser: &Parser);
    fn visit_if_end(&mut self, if_exp: &crate::parser::If, parser: &Parser);
    fn visit_generic(&mut self, generic: &crate::parser::MadGeneric, parser: &Parser);
    fn visit_call(&mut self, call_exp: &crate::parser::MadCall, parser: &Parser);
    fn visit_assignment_lhs(&mut self, lhs: &crate::parser::Expression, parser: &Parser);
    fn visit_comment(&mut self, comment: &(CursorPosition, CursorPosition), parser: &Parser);
    fn visit_string(&mut self, string: &(CursorPosition, CursorPosition), parser: &Parser);
    fn visit_subparser(&mut self, path: &str, parser: &crate::parser::Parser);

    fn visit_parser(&mut self, parser: &Parser) {
        for e in parser.get_elements() {
            e.accept(self, parser);
        }
    }
}

pub struct PrintVisitor {
    indent: usize,
    pub buffer: String,
}

impl PrintVisitor {
    pub fn new(parser: &Parser) -> Self {
        let mut print_visitor = Self {
            indent: 0,
            buffer: String::new(),
        };
        print_visitor.visit_parser(parser);
        print_visitor
    }

    pub fn indent_str(&self) -> String {
        " ".repeat(self.indent)
    }
}

impl Visitor for PrintVisitor {
    fn visit_macro(&mut self, macro_exp: &Macro, parser: &Parser) {
        write!(
            self.buffer,
            "{}{}",
            self.indent_str(),
            parser.get_element_str(&macro_exp.name)
        )
        .unwrap();
        write!(self.buffer, "(").unwrap();
        for (i, arg) in macro_exp.args.iter().enumerate() {
            if i > 0 {
                write!(self.buffer, ", ").unwrap();
            }
            write!(self.buffer, "{}", parser.get_element_str(arg)).unwrap();
        }
        write!(self.buffer, "): MACRO = {{\n").unwrap();
        self.indent += 4;
    }

    fn visit_macro_end(&mut self, _macro_exp: &Macro, _parser: &Parser) {
        self.indent -= 4;
        writeln!(self.buffer, "{}}}", self.indent_str()).unwrap();
    }

    fn visit_exec(&mut self, exec_exp: &MadExec, parser: &Parser) {
        writeln!(
            self.buffer,
            "{}EXEC, {}",
            self.indent_str(),
            parser.get_element_str(&(exec_exp.get_callee().0, exec_exp.get_range().1))
        )
        .unwrap();
    }

    fn visit_label(&mut self, label: &Label, parser: &Parser) {
        writeln!(
            self.buffer,
            "{}label {}",
            self.indent_str(),
            parser.get_element_str(label)
        )
        .unwrap();
    }

    fn visit_if(&mut self, if_exp: &crate::parser::If, parser: &Parser) {
        // Print the IF header with condition
        write!(self.buffer, "{}IF (", self.indent_str()).unwrap();
        for (i, cond) in if_exp.condition.iter().enumerate() {
            if i > 0 {
                write!(self.buffer, " ").unwrap();
            }
            write!(self.buffer, "{}", parser.get_element_str(cond)).unwrap();
        }
        writeln!(self.buffer, ") {{").unwrap();

        // Increase indent for body
        self.indent += 4;
    }

    fn visit_generic(&mut self, generic: &crate::parser::MadGeneric, _parser: &Parser) {
        let indent = self.indent_str();
        match &generic.name {
            Token::SemiColon(_) => {
                writeln!(self.buffer, "{};", indent).unwrap();
            }
            _ => {}
        }
    }

    fn visit_call(&mut self, call_exp: &crate::parser::MadCall, parser: &Parser) {
        writeln!(
            self.buffer,
            "{}CALL, {}",
            self.indent_str(),
            parser.get_element_str(&(call_exp.file_kw.get_range().0, call_exp.range.1))
        )
        .unwrap();
    }

    fn visit_if_end(&mut self, _: &crate::parser::If, _parser: &Parser) {
        self.indent -= 4;
        writeln!(self.buffer, "{}}}", self.indent_str()).unwrap();
    }

    fn visit_assignment_lhs(&mut self, lhs: &crate::parser::Expression, parser: &Parser) {
        let _ = writeln!(self.buffer, "{} = ", parser.get_element_str(lhs));
    }

    fn visit_comment(&mut self, comment: &(CursorPosition, CursorPosition), parser: &Parser) {
        let _ = writeln!(
            self.buffer,
            "{}// {}",
            self.indent_str(),
            parser.get_element_str(comment)
        );
    }

    fn visit_string(&mut self, string: &(CursorPosition, CursorPosition), parser: &Parser) {
        let _ = writeln!(
            self.buffer,
            "{}'{}'",
            self.indent_str(),
            parser.get_element_str(string)
        );
    }

    fn visit_subparser(&mut self, _: &str, _: &crate::parser::Parser) {}
}
