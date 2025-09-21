use std::collections::HashSet;

use crate::{
    lexer::CursorPosition,
    parser::{Expression, Label, Macro, Parser},
    visitor::Visitor,
};

pub struct CollectLabels {
    labels: HashSet<Vec<u8>>,
}

impl CollectLabels {
    pub fn new(parser: &Parser) -> Self {
        let mut self_ = Self {
            labels: HashSet::new(),
        };
        self_.visit_parser(parser);
        self_
    }

    pub fn get_labels(&self) -> &HashSet<Vec<u8>> {
        &self.labels
    }
}

impl Visitor for CollectLabels {
    fn visit_macro(&mut self, expression: &Macro, parser: &Parser) {
        self.labels
            .insert(parser.get_element_bytes(&expression.name).to_vec());
    }
    fn visit_macro_end(&mut self, _: &Macro, _: &Parser) {}

    fn visit_label(&mut self, label: &Label, parser: &Parser) {
        let label_str = parser.get_element_bytes(&label.name);
        self.labels.insert(label_str.to_vec());
    }

    fn visit_if(&mut self, _: &crate::parser::If, _: &Parser) {}

    fn visit_exec(&mut self, _: &crate::parser::MadExec, _: &Parser) {}

    fn visit_generic(&mut self, _: &crate::parser::MadGeneric, _: &Parser) {}

    fn visit_call(&mut self, _: &crate::parser::MadCall, _: &Parser) {}

    fn visit_if_end(&mut self, _: &crate::parser::If, _: &Parser) {}

    fn visit_assignment_lhs(&mut self, _: &Expression, _: &Parser) {}

    fn visit_comment(&mut self, _: &(CursorPosition, CursorPosition), _: &Parser) {}

    fn visit_string(&mut self, _: &(CursorPosition, CursorPosition), _: &Parser) {}

    fn visit_subparser(&mut self, path: &str, parser: &crate::parser::Parser) {
        if let Some(subparser) = parser.get_subparser(path) {
            self.visit_parser(subparser);
        }
    }
}
