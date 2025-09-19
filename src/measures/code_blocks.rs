use crate::{lexer::CursorPosition, parser::Parser, visitor::Visitor};

pub struct CodeBlocks {
    pub macros: usize,
    pub statements: usize,
    pub elements: usize,
}

impl CodeBlocks {
    pub fn new(parser: &Parser) -> Self {
        let mut code_blocks = Self {
            macros: 0,
            statements: 0,
            elements: 0,
        };
        code_blocks.visit_parser(parser);
        code_blocks
    }
}

impl Visitor for CodeBlocks {
    fn visit_macro(&mut self, _macro_exp: &crate::parser::Macro, _parser: &Parser) {
        self.macros += 1;
        self.statements += 1;
    }

    fn visit_macro_end(&mut self, _macro_exp: &crate::parser::Macro, _parser: &Parser) {}

    fn visit_exec(&mut self, _exec_exp: &crate::parser::MadExec, _parser: &Parser) {
        self.statements += 1;
    }

    fn visit_label(&mut self, _label: &crate::parser::Label, _parser: &Parser) {
        self.statements += 1;
    }

    fn visit_if(&mut self, _if_exp: &crate::parser::If, _parser: &Parser) {
        self.statements += 1;
    }

    fn visit_if_end(&mut self, _if_exp: &crate::parser::If, _parser: &Parser) {}

    fn visit_generic(&mut self, _generic: &crate::parser::MadGeneric, _parser: &Parser) {
        self.statements += 1;
    }

    fn visit_call(&mut self, _call_exp: &crate::parser::MadCall, _parser: &Parser) {
        self.statements += 1;
    }

    fn visit_assignment_lhs(&mut self, _lhs: &crate::parser::Expression, _parser: &Parser) {
        self.statements += 1;
    }

    fn visit_comment(&mut self, _comment: &(CursorPosition, CursorPosition), _parser: &Parser) {}

    fn visit_string(&mut self, _string: &(CursorPosition, CursorPosition), _parser: &Parser) {}

    fn visit_subparser(&mut self, _: &str, _: &crate::parser::Parser) {}
}
