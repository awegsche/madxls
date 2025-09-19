use std::path::Path;
use tower_lsp::lsp_types::SemanticTokensResult;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{SemanticTokens, Url};

use crate::highlighter::Highlighter;
use crate::parser::Parser;

#[derive(Debug)]
pub struct Document {
    pub parser: Parser,
}

impl Document {
    pub fn open<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        Ok(Self {
            parser: Parser::from_path(path)?,
        })
    }

    pub fn new(uri: Option<Url>, text: &[u8]) -> Self {
        Self {
            parser: Parser::from_bytes(text.to_vec(), uri),
        }
    }

    pub fn reload(&mut self, text: &[u8]) {
        let uri = self.parser.uri.clone();
        self.parser = Parser::from_bytes(text.to_vec(), uri);
        //self.parser.scan_includes();
    }

    pub fn get_semantic_tokens(&self) -> Result<Option<SemanticTokensResult>> {
        let highlighter = Highlighter::new(&self.parser);

        let mut pline = 1;
        let mut pstart = 0;

        Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
            result_id: None,
            data: highlighter
                .highlights
                .iter()
                .map(|h| h.into_semantic_token(&mut pline, &mut pstart, &self.parser))
                .collect(),
        })))
    }
}

pub fn sanitize_string_for_md(s: String) -> String {
    s.replace("*", "\\*").replace("_", "\\_")
}
