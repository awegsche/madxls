
use std::{fmt::Display, collections::HashMap, error::Error};

use tower_lsp::lsp_types::{Url, SemanticToken, SemanticTokenType, Position};

use crate::{lexer::{Token, Lexer, CursorPosition, HasRange}, error::{MadxLsError, UTF8_PARSER_MSG}};

pub mod expression;
pub mod madgeneric;
pub mod label;
pub mod madenvironment;
pub mod assignment;
pub mod madmacro;

pub use expression::*;
pub use madgeneric::*;
pub use label::*;
pub use madenvironment::*;
pub use assignment::*;
pub use madmacro::*;


#[derive(Debug)]
pub struct Parser {
    pub uri: Option<Url>,
    pub lexer: Lexer,
    elements: Vec<Expression>,
    pub labels: HashMap<Vec<u8>, usize>,
    pub position: usize,
    pub includes: Vec<Url>,
}

pub const LEGEND_TYPE: &[SemanticTokenType] = &[
    SemanticTokenType::TYPE,        // 0
    SemanticTokenType::STRING,      // 1
    SemanticTokenType::COMMENT,     // 2
    SemanticTokenType::OPERATOR,    // 3
    SemanticTokenType::FUNCTION,    // 4
    SemanticTokenType::PARAMETER,   // 5
    SemanticTokenType::MACRO,       // 6
    SemanticTokenType::NAMESPACE,   // 7
    SemanticTokenType::KEYWORD,     // 8
];

impl Parser {
    pub fn open(uri: Url) -> Result<Self, Box<dyn Error>> {
        let lexer = Lexer::open(uri.to_file_path().or(MadxLsError::new("couldn't parse uri"))?)?;
        Ok(Self::from_lexer(Some(uri), lexer))
    }

    pub fn from_path<P: AsRef<std::path::Path>>(path: P) -> std::io::Result<Self> {
        let lexer = Lexer::open(path)?;
        Ok(Self::from_lexer(None, lexer))
    }

    pub fn from_lexer(uri: Option<Url>, lexer: Lexer) -> Self {
        lexer.get_tokens();


        let mut parser = Self {
            uri,
            lexer,
            elements: Vec::new(),
            labels: HashMap::new(),
            includes: Vec::new(),
            position: 0,
        };
        parser.parse_elements();
        parser.scan_includes();
        parser
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self::from_lexer(None, Lexer::from_bytes(bytes))
    }

    pub fn from_str(str: &str) -> Self {
        Self::from_lexer(None, Lexer::from_str(str))
    }

    /// reloads the parser from a given text (as bytes)
    pub fn reload(&mut self, bytes: &[u8]) {
        log::debug!("reloading lexer");
        log::debug!("text {}", std::str::from_utf8(bytes).unwrap_or(UTF8_PARSER_MSG));
        self.lexer = Lexer::from_bytes(bytes.to_vec());
        self.position = 0;
        //log::debug!("lexer: {:#?}", self.lexer);
        //log::debug!("tokens: {:#?}", self.tokens);

        self.elements.clear();
        self.labels.clear();
        self.parse_elements();
    }

    pub fn scan_includes(&mut self) {
        log::info!("scanning includes");
        self.includes = self.elements.iter().filter_map(|e| match e {
            Expression::MadGeneric(g) => {
                if g.match_name == b"call" {
                    Some(g)
                } else {
                    None
                }
            },_ => None
        })
        .filter_map(|g| g.args.first()?.value.as_ref())
            .filter_map(|arg| String::from_utf8(self.get_element_bytes(&**arg)[1..].to_vec()).ok())
            .filter_map(|filename| {log::info!("try include {}", filename); std::path::Path::new(&filename).canonicalize().ok() })
            .filter_map(|filename| {
                if let Some(fname) = filename.extension() {
                    if fname == "mad" || fname == "madx" {
                        if filename.exists() {
                            return Some(filename);
                        }
                    }
                }
                None
            })
        .filter_map(|filename| 
             Url::from_file_path(filename).ok())
            .collect::<Vec<_>>();
    }

    fn parse_elements(&mut self) {
        while let Some(expr) = Assignment::parse(self) {
            match &expr {
                Expression::Label(label) => {
                self.labels.insert(
                    self.get_element_bytes(&label.name).to_ascii_lowercase().to_vec(),
                    self.elements.len());
                },
                Expression::Assignment(assignment) => {
                self.labels.insert(
                    self.get_element_bytes(&*assignment.lhs).to_ascii_lowercase().to_vec(),
                    self.elements.len());
                },
                Expression::Macro(m) => {
                    self.labels.insert(
                        self.get_element_bytes(&m.name).to_ascii_lowercase().to_vec(),
                        self.elements.len());
                },
                _ => { }
            }
            self.elements.push(expr);
        }
    }

    pub fn uri(&self) -> Option<&Url> {
        self.uri.as_ref()
    }

    pub fn get_elements(&self) -> &Vec<Expression> {
        &self.elements
    }

    pub fn peek_token(&self) -> Option<&Token> {
        self.lexer.get_tokens().get(self.position)
    }

    // ---- cursor movement ------------------------------------------------------------------------

    pub fn advance(&mut self) {
        self.position += 1;
    }

    pub fn go_back(&mut self) {
        self.position -= 1;
    }

    pub fn get_position(&self) -> usize {
        self.position
    }

    pub fn set_position(&mut self, pos: usize) {
        self.position = pos;
    }

    // ---- print elements -------------------------------------------------------------------------
    //
    pub fn get_element_bytes<R: HasRange>(&self, element: &R) -> &[u8] {
        self.lexer.get_range_bytes(element)
    }
    pub fn get_element_str<R: HasRange>(&self, element: &R) -> String {
        String::from_utf8(self.lexer.get_range_bytes(element).to_vec())
            .unwrap_or_else(|_| UTF8_PARSER_MSG.to_string())
    }

    pub fn get_expression_at(&self, pos: CursorPosition) -> Option<&Expression> {
        for expr in self.elements.iter().rev() {
            if expr.get_range().0 <= pos {
                return Some(expr);
            }
        }
        None
    }

}

impl Display for Parser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for expr in &self.elements {
            match expr {
                Expression::String(_) => writeln!(f, "String({})", String::from_utf8_lossy(self.get_element_bytes(expr)))?,
                Expression::Macro(m) => writeln!(f, "Macro({})", String::from_utf8_lossy(self.get_element_bytes(m)))?,
                Expression::Comment(_) => writeln!(f, "Comment({})", String::from_utf8_lossy(self.get_element_bytes(expr)))?,
                Expression::Symbol(_) => todo!(),
                Expression::Label(l) => writeln!(f, "Label({})", String::from_utf8_lossy(self.get_element_bytes(l)))?,
                Expression::Assignment(a) => writeln!(f, "Assignment({})", String::from_utf8_lossy(self.get_element_bytes(a)))?,
                Expression::MadGeneric(generic) => {
                    write!(f, "MadGeneric(({})", self.lexer.format_range_ref(&generic.name.get_range()))?;
                    for param in generic.args.iter() {
                        write!(f, " {}", self.lexer.format_range_ref(&param.get_range()))?;
                    }
                    writeln!(f, ")")?;
                },
                Expression::MadEnvironment(env) => writeln!(f, "Environment({})", String::from_utf8_lossy(self.get_element_bytes(env)))?,
                Expression::Operator(_) => todo!(),
                Expression::TokenExp(_) => writeln!(f, "Token({})", String::from_utf8_lossy(self.get_element_bytes(expr)))?,
                Expression::Exit(_) => writeln!(f, "EXIT")?,
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn parse_str() {
        let parser = Parser::from_str("\"this is a string\"");
        let string = &parser.get_elements()[0];
        if let Expression::String(_) = string {
            assert!(true);
        }
        else {
            assert!(false);
        }

    }

    #[test]
    fn parse_option() {
        let parser = Parser::from_str("option, echo, -warn;");
        let string = &parser.get_elements()[0];
        if let Expression::MadGeneric(mad_generic) = string {
            //assert!(false, "{:#?}", mad_generic);
            assert!(mad_generic.match_name == b"option");
        }
        else {
            assert!(false, "expression: {:#?}\nparser:{:}", string, parser);
        }
    }

    /// this test initialises a parser from the string
    /// "! hello\noption, echo, -warn;" and returns a list of parsed expressions
    /// those are then converted to semantic tokens using the `to_semantic_token` function
    #[test]
    fn semantic_tokens() {
        let mut parser = Parser::from_str("! hello\ncall, file;");
        let mut semantic_tokens = Vec::new();

        let mut pstart = 0;
        let mut pline = 0;

        let p = &mut parser;

        for e in p.get_elements() {
            e.to_semantic_token(&mut semantic_tokens, &mut pline, &mut pstart, p);
        }


    }

    #[test]
    fn parse_unfinished() {
        let mut parser = Parser::from_str("call, \n! comment");
        let mut semantic_tokens = Vec::new();

        let mut pstart = 0;
        let mut pline = 0;

        let p = &mut parser;

        for e in p.get_elements() {
            e.to_semantic_token(&mut semantic_tokens, &mut pline, &mut pstart, p);
        }


    }

    #[test]
    fn parse_empty() {
        let parser = Parser::from_str("");

        assert!(parser.get_elements().is_empty());
    }
}

