use std::collections::HashSet;

use crate::{
    lexer::{CursorPosition, HasRange},
    parser::Parser,
    visitor::Visitor,
};

pub struct Loc {
    pub lines: HashSet<usize>,
    pub commentlines: HashSet<usize>,
}

impl Loc {
    pub fn new(parser: &Parser) -> Self {
        let mut loc = Self {
            lines: HashSet::new(),
            commentlines: HashSet::new(),
        };
        loc.visit_parser(parser);
        loc
    }
}

impl Visitor for Loc {
    fn visit_macro(&mut self, macro_exp: &crate::parser::Macro, _parser: &Parser) {
        for line in macro_exp.name.get_lines() {
            self.lines.insert(line);
        }
        self.lines.insert(macro_exp.end.line());
    }

    fn visit_macro_end(&mut self, _: &crate::parser::Macro, _parser: &Parser) {}

    fn visit_exec(&mut self, exec_exp: &crate::parser::MadExec, _parser: &Parser) {
        for line in exec_exp.name.get_lines() {
            self.lines.insert(line);
        }
    }

    fn visit_label(&mut self, label: &crate::parser::Label, _parser: &Parser) {
        for line in label.name.get_lines() {
            self.lines.insert(line);
        }
    }

    fn visit_if(&mut self, if_exp: &crate::parser::If, _parser: &Parser) {
        for line in if_exp.if_pos.get_lines() {
            self.lines.insert(line);
        }
    }

    fn visit_if_end(&mut self, _: &crate::parser::If, _parser: &Parser) {}

    fn visit_generic(&mut self, generic: &crate::parser::MadGeneric, _parser: &Parser) {
        for line in generic.name.get_lines() {
            self.lines.insert(line);
        }
    }

    fn visit_call(&mut self, call_exp: &crate::parser::MadCall, _parser: &Parser) {
        for line in call_exp.name.get_lines() {
            self.lines.insert(line);
        }
    }

    fn visit_assignment_lhs(&mut self, lhs: &crate::parser::Expression, _parser: &Parser) {
        for line in lhs.get_lines() {
            self.lines.insert(line);
        }
    }

    fn visit_comment(&mut self, comment: &(CursorPosition, CursorPosition), _parser: &Parser) {
        for line in comment.get_lines() {
            self.commentlines.insert(line);
        }
    }

    fn visit_string(&mut self, string: &(CursorPosition, CursorPosition), _parser: &Parser) {
        for line in string.get_lines() {
            self.lines.insert(line);
        }
    }

    fn visit_subparser(&mut self, _: &str, _: &crate::parser::Parser) {}
}
