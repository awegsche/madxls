use std::path::Path;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, DocumentHighlight, MarkedString, Position, Range,
    SemanticTokens, SemanticTokensResult, Url,
};

use crate::error::UTF8_PARSER_MSG;
use crate::parser::{Expression, Parser, Problem, GENERIC_BUILTINS};

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
}

pub fn sanitize_string_for_md(s: String) -> String {
    s.replace("*", "\\*").replace("_", "\\_")
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_macros() {
        let elements = vec![
            "// test file", ";",
            "/* this is a multiline comment\n* explaining what the macro does\n* in a very detailed way */",
            "do_twiss(filename): macro = {\n  twiss, sequence=lhcb1;\n}",
            ";",
        ];
        let doc = Document::new(None, elements.join("\n").as_bytes());
        let expressions = doc.parser.get_elements();

        assert_eq!(doc.parser.get_element_str(&expressions[0]), elements[0]);
        assert_eq!(doc.parser.get_element_str(&expressions[2]), elements[2]);
        if let Expression::Macro(m) = &expressions[3] {
            assert_eq!(doc.parser.get_element_str(m), elements[3]);
        } else {
            assert!(
                false,
                "exprected macro, got: {:?}\nrange: {}",
                expressions[3],
                doc.parser.get_element_str(&expressions[3])
            );
        }
    }
}
