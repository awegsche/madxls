use std::{collections::HashMap, error::Error, fmt::Display, path::PathBuf};

use tower_lsp::lsp_types::{Range, SemanticTokenType, Url};

use crate::{
    error::{MadxLsError, UTF8_PARSER_MSG},
    lexer::{CursorPosition, HasRange, Lexer, Token},
};

pub mod assignment;
pub mod expression;
pub mod label;
pub mod madcall;
pub mod madenvironment;
pub mod madexec;
pub mod madgeneric;
pub mod madif;
pub mod madmacro;
pub mod problem;

pub use assignment::*;
pub use expression::*;
pub use label::*;
pub use madcall::*;
pub use madenvironment::*;
pub use madexec::*;
pub use madgeneric::*;
pub use madif::*;
pub use madmacro::*;
pub use problem::*;

#[derive(Debug)]
pub struct Parser {
    pub uri: Option<Url>,
    pub lexer: Lexer,
    elements: Vec<Expression>,
    pub position: usize,
    subparsers: HashMap<Url, Parser>,
    missing_files: Vec<Range>,
}

pub const LEGEND_TYPE: &[SemanticTokenType] = &[
    SemanticTokenType::KEYWORD,   // 0
    SemanticTokenType::TYPE,      // 1
    SemanticTokenType::CLASS,     // 2
    SemanticTokenType::FUNCTION,  // 3
    SemanticTokenType::PARAMETER, // 4
    SemanticTokenType::COMMENT,   // 5
    SemanticTokenType::MACRO,     // 6
    SemanticTokenType::STRING,    // 7
    SemanticTokenType::KEYWORD,   // 8
    SemanticTokenType::KEYWORD,   // 9
];

impl Parser {
    pub fn open(uri: Url) -> Result<Self, Box<dyn Error>> {
        let lexer = Lexer::open(
            uri.to_file_path()
                .or(MadxLsError::new("couldn't parse uri"))?,
        )?;
        Ok(Self::from_lexer(Some(uri), lexer))
    }

    pub fn from_path<P: AsRef<std::path::Path>>(path: P) -> std::io::Result<Self> {
        let lexer = Lexer::open(path.as_ref())?;
        Ok(Self::from_lexer(
            Some(Url::from_file_path(path.as_ref().canonicalize().unwrap()).unwrap()),
            lexer,
        ))
    }

    pub fn from_lexer(uri: Option<Url>, lexer: Lexer) -> Self {
        lexer.get_tokens();

        let mut parser = Self {
            uri,
            lexer,
            elements: Vec::new(),
            position: 0,
            subparsers: HashMap::new(),
            missing_files: Vec::new(),
        };
        parser.parse_elements();
        parser
    }

    pub fn from_bytes(bytes: Vec<u8>, uri: Option<Url>) -> Self {
        Self::from_lexer(uri, Lexer::from_bytes(bytes))
    }

    pub fn from_str(str: &str) -> Self {
        Self::from_lexer(None, Lexer::from_str(str))
    }

    /// reloads the parser from a given text (as bytes)
    pub fn reload(&mut self, bytes: &[u8]) {
        log::debug!("reloading lexer");
        log::debug!(
            "text {}",
            std::str::from_utf8(bytes).unwrap_or(UTF8_PARSER_MSG)
        );
        self.lexer = Lexer::from_bytes(bytes.to_vec());
        self.position = 0;
        //log::debug!("lexer: {:#?}", self.lexer);
        //log::debug!("tokens: {:#?}", self.tokens);

        self.elements.clear();
        self.parse_elements();
    }

    fn parse_elements(&mut self) {
        while let Some(expr) = Assignment::parse(self) {
            if let Expression::Call(call) = &expr {
                if let Some(uri) = call.get_filename(self) {
                    let url = self.get_subparser_uri(&uri).unwrap();
                    if let Ok(mut subparser) = Parser::open(url.clone()) {
                        subparser.parse_elements();
                        self.subparsers.insert(url, subparser);
                    } else {
                        self.missing_files
                            .push(self.lexer.cursor_range_to_text_range(&call.get_range()));
                    }
                }
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

    fn get_subparser_uri(&self, uri: &str) -> Option<Url> {
        if let Some(mut self_uri) = self.uri.clone() {
            let path_to_file = PathBuf::from(self_uri.path());
            self_uri.set_path(path_to_file.parent().unwrap().join(uri).to_str().unwrap());
            return Some(self_uri);
        }
        None
    }

    pub fn get_subparser(&self, uri: &str) -> Option<&Parser> {
        // check relative to self.uri
        if let Some(self_uri) = self.get_subparser_uri(uri) {
            if let Some(subparser) = self.subparsers.get(&self_uri) {
                return Some(subparser);
            }
        }
        None
    }

    pub fn peek_token(&self) -> Option<&Token> {
        self.lexer.get_tokens().get(self.position)
    }

    pub fn next_token(&mut self) -> Option<&Token> {
        let pos = self.position;
        self.advance();
        self.lexer.get_tokens().get(pos)
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

    pub fn get_missing_files(&self) -> &Vec<Range> {
        &self.missing_files
    }
}

impl Display for Parser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for expr in &self.elements {
            match expr {
                Expression::String(_) => writeln!(
                    f,
                    "String({})",
                    String::from_utf8_lossy(self.get_element_bytes(expr))
                )?,
                Expression::Macro(m) => writeln!(
                    f,
                    "Macro({})",
                    String::from_utf8_lossy(self.get_element_bytes(m))
                )?,
                Expression::Comment(_) => writeln!(
                    f,
                    "Comment({})",
                    String::from_utf8_lossy(self.get_element_bytes(expr))
                )?,
                Expression::Symbol(_) => todo!(),
                Expression::Label(l) => writeln!(
                    f,
                    "Label({})",
                    String::from_utf8_lossy(self.get_element_bytes(l))
                )?,
                Expression::Assignment(a) => writeln!(
                    f,
                    "Assignment({})",
                    String::from_utf8_lossy(self.get_element_bytes(a))
                )?,
                Expression::MadGeneric(generic) => {
                    write!(
                        f,
                        "MadGeneric(({})",
                        self.lexer.format_range_ref(&generic.name.get_range())
                    )?;
                    for param in generic.args.iter() {
                        write!(f, " {}", self.lexer.format_range_ref(&param.get_range()))?;
                    }
                    writeln!(f, ")")?;
                }
                Expression::MadEnvironment(env) => writeln!(
                    f,
                    "Environment({})",
                    String::from_utf8_lossy(self.get_element_bytes(env))
                )?,
                Expression::Operator(_) => todo!(),
                Expression::TokenExp(_) => writeln!(
                    f,
                    "Token({})",
                    String::from_utf8_lossy(self.get_element_bytes(expr))
                )?,
                Expression::Exit(_) => writeln!(f, "EXIT")?,
                Expression::Exec(_) => writeln!(f, "exec (??)")?,
                Expression::If(_) => writeln!(f, "if(...) {{ }}")?,
                Expression::Noop(_) => writeln!(f, "NOOP")?,
                Expression::Call(_) => writeln!(f, "call (??)")?,
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
        } else {
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
        } else {
            assert!(false, "expression: {:#?}\nparser:{:}", string, parser);
        }
    }

    #[test]
    fn parse_empty() {
        let parser = Parser::from_str("");

        assert!(parser.get_elements().is_empty());
    }
}
