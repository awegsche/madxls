use std::fmt::Display;

use tower_lsp::lsp_types::Range;

use crate::{
    lexer::{print_range, CursorPosition},
    parser::Parser,
    visitor::Visitor,
};

pub enum Highlight {
    Keyword(Range),
    Type(Range),
    Class(Range),
    Function(Range),
    Parameter(Range),
    Comment(Range),
    Constant(Range),
    String(Range),
}

pub struct Highlighter<'a> {
    pub highlights: Vec<Highlight>,
    pub parser: &'a Parser,
}

impl<'a> Highlighter<'a> {
    pub fn new(parser: &'a Parser) -> Self {
        let mut highlighter = Self {
            highlights: Vec::new(),
            parser,
        };

        highlighter.visit_parser(parser);
        highlighter
    }
}

impl Visitor for Highlighter<'_> {
    fn visit_macro(&mut self, macro_exp: &crate::parser::Macro, parser: &Parser) {
        self.highlights.push(Highlight::Type(
            parser.lexer.cursor_range_to_text_range(&macro_exp.name),
        ));
        self.highlights.push(Highlight::Keyword(
            parser
                .lexer
                .cursor_range_to_text_range(&macro_exp.macro_pos),
        ));
    }

    fn visit_macro_end(&mut self, _: &crate::parser::Macro, _parser: &Parser) {}

    fn visit_exec(&mut self, exec_exp: &crate::parser::MadExec, parser: &Parser) {
        self.highlights.push(Highlight::Keyword(
            parser.lexer.cursor_range_to_text_range(&exec_exp.name),
        ));
    }

    fn visit_label(&mut self, label: &crate::parser::Label, parser: &Parser) {
        self.highlights.push(Highlight::Constant(
            parser.lexer.cursor_range_to_text_range(&label.name),
        ));
    }

    fn visit_if(&mut self, if_exp: &crate::parser::If, parser: &Parser) {
        self.highlights.push(Highlight::Keyword(
            parser.lexer.cursor_range_to_text_range(&if_exp.if_pos),
        ));
    }

    fn visit_if_end(&mut self, _if_exp: &crate::parser::If, _parser: &Parser) {}

    fn visit_generic(&mut self, generic: &crate::parser::MadGeneric, parser: &Parser) {
        self.highlights.push(Highlight::Function(
            parser.lexer.cursor_range_to_text_range(&generic.name),
        ));
    }

    fn visit_call(&mut self, call_exp: &crate::parser::MadCall, parser: &Parser) {
        self.highlights.push(Highlight::Function(
            parser.lexer.cursor_range_to_text_range(&call_exp.name),
        ));
    }

    fn visit_assignment_lhs(&mut self, _lhs: &crate::parser::Expression, _parser: &Parser) {}

    fn visit_comment(&mut self, comment: &(CursorPosition, CursorPosition), parser: &Parser) {
        self.highlights.push(Highlight::Comment(
            parser.lexer.cursor_range_to_text_range(comment),
        ));
    }

    fn visit_string(&mut self, string: &(CursorPosition, CursorPosition), parser: &Parser) {
        self.highlights.push(Highlight::String(
            parser.lexer.cursor_range_to_text_range(string),
        ));
    }

    fn visit_subparser(&mut self, _: &str, _: &crate::parser::Parser) {}
}

impl Display for Highlight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Highlight::Keyword(range) => write!(f, "hi kw | {}", print_range(range)),
            Highlight::Type(range) => write!(f, "hi ty | {}", print_range(range)),
            Highlight::Class(range) => write!(f, "hi cl | {}", print_range(range)),
            Highlight::Function(range) => write!(f, "hi fn | {}", print_range(range)),
            Highlight::Parameter(range) => write!(f, "hi pa | {}", print_range(range)),
            Highlight::Comment(range) => write!(f, "hi // | {}", print_range(range)),
            Highlight::Constant(range) => write!(f, "hi co | {}", print_range(range)),
            Highlight::String(range) => write!(f, "hi st | {}", print_range(range)),
        }
    }
}
