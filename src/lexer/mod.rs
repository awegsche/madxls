use std::{path::Path, io, fmt::{Display}, ops::AddAssign, borrow::Cow};

pub mod token;
pub mod cursor;

pub use token::*;
pub use cursor::*;
use tower_lsp::lsp_types::Position;

use crate::error::UTF8_PARSER_MSG;


pub trait HasRange {
    fn get_range(&self) -> (CursorPosition, CursorPosition);
}

impl HasRange for (CursorPosition, CursorPosition) {
    fn get_range(&self) -> (CursorPosition, CursorPosition) {
        (self.0, self.1)
    }
}

impl HasRange for &(CursorPosition, CursorPosition) {
    fn get_range(&self) -> (CursorPosition, CursorPosition) {
        **self
    }
}


#[derive(Debug)]
pub struct Lexer {
    buffer: Vec<u8>,
    lines: Vec<usize>,
    position: CursorPosition,
    tokens: Vec<Token>,
}

impl Lexer {
    /// ---- init ----------------------------------------------------------------------------------
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Lexer> {
        let buffer = std::fs::read(path)?;
        Ok(Lexer::from_bytes(buffer))
    }

    pub fn from_bytes(buffer: Vec<u8>) -> Self {
        let mut lines = Vec::new();
        lines.push(0);
        for (i,b) in buffer.iter().enumerate() {
            if *b == b'\n' {
                lines.push(i+1);
            }
        }
        let mut lexer = Lexer {
            buffer,
            lines,
            position: Default::default(),
            tokens: Vec::new(),
        };
        lexer.parse_tokens();
        lexer
    }

    /// mainly for debuging reasons
    pub fn from_str(data: &str) -> Self {
        Lexer::from_bytes(data.as_bytes().to_vec())
    }


    /// ---- getters -------------------------------------------------------------------------------

    /// returns a vector of line start positions.
    /// e.g. if you have lines with lengths (5, 7, 8) chars, `self.lines()` will return
    /// [0, 5, 12]
    pub fn lines(&self) -> &Vec<usize> {
        &self.lines
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn cursor_pos_from_text_pos(&self, pos: Position) -> CursorPosition {
        CursorPosition::new(pos.character as usize + self.lines[pos.line as usize - 1], pos.line as usize)
    }

    pub fn cursor_pos_to_text_pos(&self, pos: CursorPosition) -> Position {
        Position::new(pos.line() as u32, pos.character(&self.lines) as u32)
    }

    pub fn get_token_byte(&self, position: &CursorPosition) -> &u8 {
        &self.buffer[position.absolute()]
    }

    pub fn get_range_bytes<R: HasRange>(&self, element: &R) -> &[u8] {
        let range = element.get_range();
        &self.buffer[range.0.absolute()..range.1.absolute()]
    }

    /// Compares the token at range with the given bytes, case insensitive
    pub fn compare_range<R: HasRange>(&self, element: &R, bytes: &[u8]) -> bool {
        self.get_range_bytes(element).to_ascii_lowercase() == bytes
    }

    pub fn get_token_bytes(&self, token: &Token) -> &[u8] {
        self.get_range_bytes(token)
    }

    pub fn get_range_str<R: HasRange>(&self, token: &R) -> Cow<str> {
        String::from_utf8_lossy(self.get_range_bytes(token))
    }

    pub fn format_position(&self, pos: &CursorPosition) -> String {
        let pos = pos.absolute();
        format!("{}", String::from_utf8_lossy(&self.buffer[pos..pos+1]))
    }

    pub fn format_range_ref(&self, range: &(CursorPosition, CursorPosition)) -> String {
        let s = range.0.absolute();
        let e = range.1.absolute();
        format!("{}", String::from_utf8_lossy(&self.buffer[s..e]))
    }


    pub fn format_range(&self, range: &(CursorPosition, CursorPosition)) -> String {
        let s = range.0.absolute();
        let e = range.1.absolute();
        format!("{}", String::from_utf8_lossy(&self.buffer[s..e]))
    }
    

    /// ---- printing ------------------------------------------------------------------------------
    pub fn format_token(&self, token: &Token) -> String {
        match token {
            Token::BraceOpen(p) => self.format_position(p),
            Token::BraceClose(p) => self.format_position(p),
            Token::ParentOpen(p) => self.format_position(p),
            Token::ParentClose(p) => self.format_position(p),
            Token::Ident((s,e)) => format!("Ident({})", self.format_range(&(*s, *e))),
            Token::Number((s,e)) => format!("Number({})", self.format_range(&(*s, *e))),
            Token::Operator(p) => self.format_position(p),
            Token::Equal(p) => self.format_position(p),
            Token::ColonEqual(p) => self.format_range(&(*p, p+1)),
            Token::Dot(p) => self.format_position(p),
            Token::SemiColon(p) => self.format_position(p),
            Token::Colon(p) => self.format_position(p),
            Token::Komma(p) => self.format_position(p),
            Token::Quotes(p) => self.format_position(p),
            Token::DoubleQuotes(p) => self.format_position(p),
            Token::Comment((s,e)) => format!("Comment({})", self.format_range(&(*s, *e))),
            Token::MultilineComment(_) => format!("Comment({})", self.format_range(&token.get_range())),
            Token::Char(p) => self.format_position(p),
            Token::EOF => "EOF".to_string(),
        }
    }

    /// ---- get token(s) --------------------------------------------------------------------------
    ///
    fn parse_tokens(&mut self) {
        while let Some(token) = self.next_token() {
            if token == Token::EOF {
                break;
            }
            self.tokens.push(token);
        }
    }

    pub fn get_tokens(&self) -> &Vec<Token> {
        &self.tokens
    }

    pub fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace(); 

        if let Some(p) = self.peak_char() {
            if p.is_ascii_digit() {
                return self.read_number();
            }
            if p.is_ascii_alphanumeric() {
                return self.read_ident();
            }
            let token = match p {
                b'{' => Some(Token::BraceOpen(self.position)),
                b'}' => Some(Token::BraceClose(self.position)),
                b'(' => Some(Token::ParentOpen(self.position)),
                b')' => Some(Token::ParentClose(self.position)),
                b'.' => Some(Token::Dot(self.position)),
                b',' => Some(Token::Komma(self.position)),
                b';' => Some(Token::SemiColon(self.position)),
                b':' => self.read_colon(),
                b'+' => Some(Token::Operator(self.position)),
                b'-' => Some(Token::Operator(self.position)),
                b'_' => Some(Token::Operator(self.position)),
                b'*' => Some(Token::Operator(self.position)),
                b'<' => Some(Token::Operator(self.position)),
                b'>' => Some(Token::Operator(self.position)),
                b'=' => Some(Token::Equal(self.position)),
                b'/' => self.read_forward_slash(),
                b'\'' => Some(Token::Quotes(self.position)),
                b'"' => Some(Token::DoubleQuotes(self.position)),
                b'!' => self.read_comment(),
                b'#' => {
                    let p0 = self.position;
                    self.position += 1;
                    if let Some(Token::Ident(mut p)) = self.read_ident() {
                        p.0 = p0;
                        self.position -= 1;
                        Some(Token::Ident(p))
                    }
                    else {
                        None
                    }
                },
                _ => Some(Token::Char(self.position)),
            };
            self.position += 1;
            return token;
        }
        self.position += 1;
        None
    }

    /// ---- internal reading functions ------------------------------------------------------------
    fn next_char(&mut self) -> Option<u8> {
        if self.position.absolute() >= self.buffer.len() {
            return None;
        }
        let c = self.buffer[self.position.absolute()];
        self.position += 1; // position is now one character ahead
        Some(c)
    }

    fn peak_char(&self) -> Option<u8> {
        if self.position.absolute() >= self.buffer.len() {
            return None;
        }
        Some(self.buffer[self.position.absolute()])
    }

    fn skip_whitespace(&mut self) {

        while let Some(c) = self.peak_char() {
            if !c.is_ascii_whitespace() {
                break;
            }
            if c == b'\n' {
                self.position.advance_line();    
            }
            self.position += 1;
        }
    }

    pub fn read_colon(&mut self) -> Option<Token> {
        self.position += 1;

        if let Some(b'=') = self.peak_char() {
            Some(Token::ColonEqual(self.position))
        }
        else {
            self.position -= 1;
            Some(Token::Colon(self.position))
        }
    }

    pub fn read_forward_slash(&mut self) -> Option<Token> {
        let p1 = self.position;
        self.position += 1;
        if let Some(b'/') = self.peak_char() {
            if let Some(Token::Comment(mut comment)) = self.read_comment() {
                comment.0 = p1;
                return Some(Token::Comment(comment));
            }
        }
        else if let Some(b'*') = self.peak_char() {
            let mut start = p1;
            let mut lines = Vec::new();
            self.position += 1;

            while let Some(c) = self.peak_char(){
                if c == b'\n' {
                    lines.push((start, self.position));
                    //self.position += 1;
                    self.position.advance_line();
                    start = self.position+1;
                }
                else if c == b'*' {
                    self.position += 1;
                    if let Some(b'/') = self.peak_char() {
                        lines.push((start, self.position+1));
                        break;
                    }
                }
                self.position += 1;
            }
            return Some(Token::MultilineComment(lines));
        }
        self.position -= 1;
        Some(Token::Operator(p1))
    }

    pub fn read_comment(&mut self) -> Option<Token> {
        let p1 = self.position;
        while let Some(c) = self.peak_char(){
            if c == b'\n' {
                break;
            }
            self.position += 1;
        }
        let end = self.position;
        self.position.advance_line();
        Some(Token::Comment((p1, end)))

    }

    pub fn read_number(&mut self) -> Option<Token> {
        let p1 = self.position;
        while let Some(c) = self.peak_char(){
            if !c.is_ascii_digit() && c != b'.' {
                break;
            }
            self.position += 1;
        }
        Some(Token::Number((p1, self.position)))
    }

    pub fn read_ident(&mut self) -> Option<Token> {
        let p1 = self.position;
        while let Some(c) = self.peak_char(){
            if !(c.is_ascii_alphanumeric() || c == b'_' || c == b'.')  {
                break;
            }
            self.position += 1;
        }
        Some(Token::Ident((p1, self.position)))
    }

}

impl Display for Lexer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.buffer.len() > 1000 {
            write!(f, "{}\n ...", &String::from_utf8(self.buffer[0..1000].to_vec()).unwrap_or_else(|_| UTF8_PARSER_MSG.to_string()))?;
        }
        else {
            write!(f, "{}\n ...", &String::from_utf8(self.buffer.to_vec()).unwrap_or_else(|_| UTF8_PARSER_MSG.to_string()))?;
        }
        for token in self.tokens.iter() {
            writeln!(f, "{}", self.format_token(&token))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn check_string(buffer: &[u8], tokens: &[&str]) {
        let mut lexer = Lexer::from_bytes(buffer.to_vec());
        let lexer_tokens = lexer.get_tokens();
        for (token, repr) in lexer_tokens.iter().zip(tokens) {
            assert_eq!(lexer.format_token(token), repr.to_string());
        }
    }

    #[test]
    fn match_assignment() {
        let lexer = Lexer::from_str("a = b");
        let tokens = lexer.get_tokens();

        assert_eq!(tokens.len(), 3);
        assert!(tokens[1].is_assignment());

        let lexer = Lexer::from_str("a := b");
        let tokens = lexer.get_tokens();

        assert_eq!(tokens.len(), 3);
        assert!(tokens[1].is_assignment());
    }

    #[test]
    fn match_comment() {
        check_string(
            b"! this is a comment\nnextline",
            &[
            "Comment(! this is a comment)",
            "Ident(nextline)",
            ]
        )
    }

    #[test]
    fn match_single_punctuation() {
        check_string(b".", &["."]);
        check_string(b";", &[";"]);
    }

    #[test]
    fn match_fn_def() {
        check_string(
            b"fn fun()",
            &[
            "Ident(fn)",
            "Ident(fun)",
            "(",
            ")"
            ]);
    }

    #[test]
    fn match_number() {
        check_string(b"123", &["Number(123)"]);
    }

    #[test]
    fn match_option() {
        check_string(
            b"option,-echo,-info;",
            &[
            "Ident(option)",
            ",",
            "-",
            "Ident(echo)",
            ",",
            "-",
            "Ident(info)",
            ";",
            ]
            )
    }

    #[test]
    fn match_twolines() {
        check_string(
            b" option,-echo,-info;\nsystem,\"mkdir temp\";",
            &[
            "Ident(option)",
            ",",
            "-",
            "Ident(echo)",
            ",",
            "-",
            "Ident(info)",
            ";",
            "Ident(system)",
            ",",
            "\"",
            "Ident(mkdir)",
            "Ident(temp)",
            "\"",
            ";"
            ]
        );
    }

    #[test]
    fn match_test_madx() {
        let lexer = Lexer::from_bytes(include_bytes!("../../tests/test.madx").to_vec());
        let tokens = lexer.get_tokens();

        let made_it = lexer.format_token(tokens.last().unwrap());

        assert_eq!(
            made_it,
            "Ident(made_it)".to_string()
        );
    }

    #[test]
    fn match_c_style_comment() {
        let lexer = Lexer::from_str("first;\n// this is a comment\nsecond");
        let tokens = lexer.get_tokens();

        assert_eq!(lexer.get_token_bytes(&tokens[0]), b"first");
        assert_eq!(lexer.get_token_bytes(&tokens[1]), b";");
        assert_eq!(lexer.get_token_bytes(&tokens[2]), b"// this is a comment");
        assert_eq!(lexer.get_token_bytes(&tokens[3]), b"second");
    }

    #[test]
    fn match_multiline_comment() {
        let lexer = Lexer::from_str("first;\n/* this is a comment\nsecond line of comment\nthird line\n*/second");
        let tokens = lexer.get_tokens();

        assert_eq!(lexer.get_range_str(&tokens[0]), "first");
        assert_eq!(lexer.get_range_str(&tokens[1]), ";");
        assert_eq!(lexer.get_range_str(&tokens[2]), "/* this is a comment\nsecond line of comment\nthird line\n*/");
        assert_eq!(lexer.get_range_str(&tokens[3]), "second");

        if let Token::MultilineComment(v) = &tokens[2] {
            assert_eq!(lexer.get_range_str(&v[0]), "/* this is a comment");
            assert_eq!(lexer.get_range_str(&v[1]), "second line of comment");
            assert_eq!(lexer.get_range_str(&v[2]), "third line");
            assert_eq!(lexer.get_range_str(&v[3]), "*/");
        }
        else {
            assert!(false, "Expected multiline comment");
        }
    }

    #[test]
    fn underscore_at_end() {
        let lexer = Lexer::from_str("exec, a_");
        let tokens = lexer.get_tokens();

        assert_eq!(lexer.get_range_str(&tokens[0]), "exec");
        assert_eq!(lexer.get_range_str(&tokens[1]), ",");
        if let Token::Ident(range) = &tokens[2] {
            assert_eq!(lexer.get_range_str(range), "a_");
        }
        else {
            assert!(false, "Expected Ident(a_)");
        }
    }
}

