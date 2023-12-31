use std::{collections::HashMap, fmt::Display};

use once_cell::sync::Lazy;
use tower_lsp::lsp_types::{SemanticToken, CompletionItem};

use crate::{lexer::{CursorPosition, Token, HasRange}, semantic_tokens::{get_range_token}};

use super::{Parser, MadGenericBuilder, insert_generic_builder, MadGeneric, Label, GENERIC_BUILTINS, Environment, Assignment, Macro, Problem, MadExec};
#[derive(Debug, PartialEq)]
pub enum Expression {
    Label(Label),
    Macro(Macro),
    Assignment(Assignment),
    String((CursorPosition, CursorPosition)),
    Comment((CursorPosition, CursorPosition)),
    Symbol(String),
    MadGeneric(MadGeneric),
    MadEnvironment(Environment),
    Exit(Exit),
    Operator(Operator),
    Exec(MadExec),
    TokenExp(Token), // debug, todo: remove
}

impl HasRange for Expression {
    fn get_range(&self) -> (CursorPosition, CursorPosition) {
        match self {
            Expression::String(r) => *r,
            Expression::Comment(r) => *r,
            Expression::Macro(m) => m.get_range(),
            Expression::Label(_) => todo!(),
            Expression::Symbol(_) => todo!(),
            Expression::MadGeneric(g) => g.get_range(),
            Expression::MadEnvironment(e) => e.get_range(),
            Expression::Assignment(a) => a.get_range(),
            Expression::Operator(_) => todo!(),
            Expression::TokenExp(token) => token.get_range(),
            Expression::Exit(exit) => (exit.start, exit.end),
            Expression::Exec(exec) => exec.get_range(),
        }
    }
}

impl Expression {
    pub fn parse(parser: &mut Parser) -> Option<Self> {
        if let Some(m) = Macro::parse(parser) {
            return Some(Expression::Macro(m));
        }
        if let Some(string) = Self::parse_string(parser) {
            return Some(string);
        }
        if let Some(label) = Label::parse(parser) {
            return Some(Expression::Label(label));
        }
        if let Some(env) = Environment::parse(parser) {
            return Some(Expression::MadEnvironment(env));
        }
        if let Some(generic) = MadGeneric::parse(parser) {
            return Some(Expression::MadGeneric(generic));
        }
        if let Some(exec) = MadExec::parse(parser) {
            return Some(Expression::Exec(exec));
        }

        if let Some(exit) = Exit::parse(parser) {
            return Some(Expression::Exit(exit));
        }
        if let Some(token) = parser.peek_token().cloned() {
            parser.advance();
            return Some(Self::TokenExp(token));
        }
        None
    }

    pub fn get_problems(&self, problems: &mut Vec<Problem>) {
        match self {
            Expression::Label(_) => {},
            Expression::Macro(m) => {m.get_problems(problems)},
            Expression::Assignment(a) => {a.get_problems(problems)},
            Expression::String(_) => {},
            Expression::Comment(_) => {},
            Expression::Symbol(_) => {},
            Expression::MadGeneric(g) => g.get_problems(problems),
            Expression::MadEnvironment(e) => {e.get_problems(problems)},
            Expression::Exit(_) => {},
            Expression::Operator(_) => {},
            Expression::Exec(e) => e.get_problems(problems),
            Expression::TokenExp(_) => {},
        }
    }

    fn parse_string(parser: &mut Parser) -> Option<Self> {

        if let Some(Token::DoubleQuotes(p)) = parser.peek_token().cloned() {
            parser.advance();
            while let Some(token) = parser.peek_token().cloned() {
                parser.advance();
                if let Token::DoubleQuotes(p_end) = token {
                    return Some(Self::String((p, p_end)));
                }
            }
        }
        if let Some(Token::Quotes(p)) = parser.peek_token().cloned() {
            parser.advance();
            while let Some(token) = parser.peek_token().cloned() {
                parser.advance();
                if let Token::Quotes(p_end) = token {
                    return Some(Self::String((p, p_end)));
                }
            }
        }
        None
    }

    /// returns the label of the element under cursor, this is to find the definition and,
    /// possibly, jump to it
    pub fn get_label<'a>(&'a self, pos: &CursorPosition, parser: &'a Parser) -> Option<&[u8]> {
        match self {
            Expression::Label(_) => None,
            Expression::Macro(_) => None,
            Expression::Assignment(a) => a.get_label(pos, parser),
            Expression::String(_) => None,
            Expression::Comment(_) => None,
            Expression::Symbol(s) => Some(s.as_bytes()),
            Expression::MadGeneric(m) => m.get_label(pos, parser),
            Expression::MadEnvironment(m) => m.get_label(pos, parser),
            Expression::Exit(_) => None,
            Expression::Operator(_) => None,
            Expression::Exec(s) => s.get_label(pos, parser),
            Expression::TokenExp(t) => {
                let range = t.get_range();
                if &range.0 < pos && pos < &range.1 {
                    Some(parser.get_element_bytes(&range))
                }
                else { None }
            },
        }
    }

    pub fn get_completion(&self, pos: &CursorPosition, items: &mut Vec<CompletionItem>) {
        match self {
            Expression::Label(_) => {},
            Expression::Macro(m) => m.get_completion(pos, items),
            Expression::Assignment(_) => {},
            Expression::String(_) => {},
            Expression::Comment(_) => {},
            Expression::Symbol(_) => {},
            Expression::MadGeneric(g) => g.get_completion(pos, items),
            Expression::MadEnvironment(e) => {e.get_completion(pos, items)},
            Expression::Exit(_) => {},
            Expression::Exec(_) => {},
            Expression::Operator(_) => {},
            Expression::TokenExp(_) => {},
        }
    }
    pub fn to_semantic_token(&self, semantic_tokens: &mut Vec<SemanticToken>, pre_line: &mut u32, pre_start: &mut u32, parser: &Parser) {
        
        match self {
            Self::String(range) =>semantic_tokens.push(get_range_token(range, 0, pre_line, pre_start, parser)),
            Self::TokenExp(Token::Comment(range)) => semantic_tokens.push(get_range_token(range, 2, pre_line, pre_start, parser)),
            Self::TokenExp(Token::MultilineComment(lines)) => {
                for range in lines.iter() {
                    semantic_tokens.push(get_range_token(range, 2, pre_line, pre_start, parser));
                }
            },
            Self::Macro(m) => m.to_semantic_token(semantic_tokens, pre_line, pre_start, parser),
            Self::Label(label) => {
                semantic_tokens.push(get_range_token(&label.name, 6, pre_line, pre_start, parser));
                label.command.to_semantic_token(semantic_tokens, pre_line, pre_start, parser);
            },
            Self::MadGeneric(mad_generic) => {
                mad_generic.to_semantic_token(semantic_tokens, pre_line, pre_start, parser);
            }
            Self::MadEnvironment(env) => {
                env.to_semantic_token(semantic_tokens, pre_line, pre_start, parser);
            }
            Self::Exit(exit) => {
                semantic_tokens.push(get_range_token(exit, 0, pre_line, pre_start, parser));

                for lines in parser.lexer.lines()[exit.start.line()+1..].windows(2) {
                    let length = lines[1] - lines[0];
                    semantic_tokens.push(SemanticToken {
                        delta_line: 1,
                        delta_start: 0,
                        length: length as u32,
                        token_type: 2,
                        token_modifiers_bitset: 0 });
                }
            }
            _ => {},
        }

    }

    pub(crate) fn get_highlights(&self, pos: &CursorPosition, parser: &Parser) -> Vec<(CursorPosition, CursorPosition)> {
        match self {
            Expression::Label(_) => vec![],
            Expression::Macro(m) => m.get_highlights(pos, parser),
            Expression::Assignment(_) => vec![],
            Expression::String(_) => vec![],
            Expression::Comment(_) => vec![],
            Expression::Symbol(_) => vec![],
            Expression::MadGeneric(_) => vec![],
            Expression::MadEnvironment(_) => vec![],
            Expression::Exit(_) => vec![],
            Expression::Operator(_) => vec![],
            Expression::Exec(_) => vec![],
            Expression::TokenExp(_) => vec![],
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Exit {
    start: CursorPosition,
    end: CursorPosition,
    length: usize,
}

impl HasRange for Exit {
    fn get_range(&self) -> (CursorPosition, CursorPosition) {
        (self.start, self.end)
    }
}

impl Exit {
    pub fn parse(parser: &mut Parser) -> Option<Self> {
        if let Some(Token::Ident(name)) = parser.peek_token() {
            if parser.lexer.compare_range(name, b"exit")
                || parser.lexer.compare_range(name, b"quit")
                || parser.lexer.compare_range(name, b"stop") {

                    let name = name.clone();
                    parser.position += 1;
                    return Some(Self {
                        start: name.0,
                        end: name.1,
                        length: parser.lexer.len() - name.1.absolute(),
                    })
                }
        }
        None
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum OpKind {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, PartialEq)]
pub struct Operator {
    kind: OpKind,
    left: Box<Expression>,
    right: Box<Expression>,
}
