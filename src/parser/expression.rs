use crate::{
    lexer::{CursorPosition, HasRange, Token},
    parser::MadCall,
};

use super::{Assignment, Environment, If, Label, Macro, MadExec, MadGeneric, Parser};
#[derive(Debug, PartialEq)]
pub enum Expression {
    Label(Label),
    Macro(Macro),
    If(If),
    Assignment(Assignment),
    String((CursorPosition, CursorPosition)),
    Comment((CursorPosition, CursorPosition)),
    Symbol(String),
    MadGeneric(MadGeneric),
    MadEnvironment(Environment),
    Exit(Exit),
    Operator(Operator),
    Exec(MadExec),
    Call(MadCall),
    Noop(CursorPosition),
    TokenExp(Token), // debug, todo: remove
}

impl HasRange for Expression {
    fn get_range(&self) -> (CursorPosition, CursorPosition) {
        match self {
            Expression::String(r) => *r,
            Expression::Comment(r) => *r,
            Expression::If(i) => i.get_range(),
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
            Expression::Noop(pos) => (*pos, *pos),
            Expression::Call(call) => call.get_range(),
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
        if let Some(comment) = Self::parse_comment(parser) {
            return Some(comment);
        }
        if let Some(label) = Label::parse(parser) {
            return Some(Expression::Label(label));
        }
        if let Some(env) = Environment::parse(parser) {
            return Some(Expression::MadEnvironment(env));
        }
        if let Some(exec) = MadExec::parse(parser) {
            return Some(Expression::Exec(exec));
        }
        if let Some(call) = MadCall::parse(parser) {
            return Some(Expression::Call(call));
        }
        if let Some(generic) = MadGeneric::parse(parser) {
            return Some(Expression::MadGeneric(generic));
        }
        if let Some(if_object) = If::parse(parser) {
            return Some(Expression::If(if_object));
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

    pub fn accept<V: crate::visitor::Visitor>(
        &self,
        visitor: &mut V,
        parser: &crate::parser::Parser,
    ) {
        match self {
            Expression::Macro(m) => m.accept(visitor, parser),
            Expression::Assignment(a) => a.accept(visitor, parser),
            Expression::MadGeneric(g) => g.accept(visitor, parser),
            Expression::MadEnvironment(e) => e.accept(visitor, parser),
            Expression::Exec(e) => e.accept(visitor, parser),
            Expression::If(i) => i.accept(visitor, parser),
            Expression::Label(l) => l.accept(visitor, parser),
            Expression::Call(c) => c.accept(visitor, parser),
            Expression::Comment(c) => visitor.visit_comment(c, parser),
            Expression::String(s) => visitor.visit_string(s, parser),
            _ => {}
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

    fn parse_comment(parser: &mut Parser) -> Option<Self> {
        let mut comments = Vec::new();

        while let Some(token) = parser.peek_token().cloned() {
            match token {
                Token::Comment(range) => {
                    parser.advance();
                    comments.push(range);
                }
                Token::MultilineComment(ranges) => {
                    parser.advance();
                    for range in ranges {
                        comments.push(range);
                    }
                }
                _ => break, // Stop when we encounter a non-comment token
            }
        }

        if comments.is_empty() {
            None
        } else {
            Some(Expression::Comment((
                comments.first().unwrap().0,
                comments.last().unwrap().1,
            )))
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
                || parser.lexer.compare_range(name, b"stop")
            {
                let name = name.clone();
                parser.position += 1;
                return Some(Self {
                    start: name.0,
                    end: name.1,
                    length: parser.lexer.len() - name.1.absolute(),
                });
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
