use std::collections::HashMap;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{SemanticTokensResult, SemanticTokens, Position, CompletionResponse, CompletionItem, CompletionItemKind};

use crate::error::UTF8_PARSER_MSG;
use crate::lexer::HasRange;
use crate::parser::{Parser, GENERIC_BUILTINS, Expression};

#[derive(Debug)]
pub struct Document {
    pub parser: Parser,
    pub included_labels: HashMap<Vec<u8>, usize>,
}

impl Document {
    pub fn new(text: &[u8]) -> Self {
        let mut parser = Parser::from_bytes(text.to_vec());
        let included_labels = parser.scan_includes();
        Self {
            parser,
            included_labels
        }
    }

    pub fn reload(&mut self, text: &[u8]) {
        self.parser = Parser::from_bytes(text.to_vec());
        //self.parser.scan_includes();
    }

    pub fn get_semantic_tokens(&self) -> Result<Option<SemanticTokensResult>> {
        let mut pre_line = 0;
        let mut pre_start = 0;
        //log::info!("{:#?}", parser.get_elements());
        log::info!("parser elements: {}", self.parser.get_elements().len());
        let mut data = Vec::new();
        for e in self.parser.get_elements() {
            e.to_semantic_token(&mut data, &mut pre_line, &mut pre_start, &self.parser);
            if let Expression::Exit(_) = e {
                break;
            }
        }
        //log::info!("data: {:#?}", data);

        let tokens = Some(SemanticTokensResult::Tokens(
                    SemanticTokens{ 
                        data,
                        ..Default::default()
                    }));
        Ok(tokens)
    }

    pub fn get_completion(&self, position: Position) -> Result<Option<CompletionResponse>> {
        let pos = self.parser.lexer.cursor_pos_from_text_pos(position);
        log::debug!("completion triggered at {:#?}", pos);
        let mut items = Vec::new();
        for name in GENERIC_BUILTINS.keys() {
            items.push(CompletionItem{
                label: String::from_utf8(name.to_vec()).unwrap_or_else(|_| {UTF8_PARSER_MSG.to_string()}),
                kind: Some(CompletionItemKind::FUNCTION),
                ..Default::default()});
        }
        log::debug!("labels.len() = {}", self.parser.labels.len());
        for label in self.parser.labels.keys() {
            items.push(CompletionItem{
                label: String::from_utf8(label.clone()).unwrap_or_else(|_| {UTF8_PARSER_MSG.to_string()}),
                kind: Some(CompletionItemKind::VARIABLE),
                ..Default::default()
            });

        }

        for label in self.included_labels.keys() {
            items.push(CompletionItem{
                label: String::from_utf8(label.clone()).unwrap_or_else(|_| {UTF8_PARSER_MSG.to_string()}),
                kind: Some(CompletionItemKind::VARIABLE),
                ..Default::default()
            });

        }

        for e in self.parser.get_elements() {
            e.get_completion(&pos, &mut items);
        }

        Ok(Some(CompletionResponse::Array(items)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let doc = Document::new(b"option, echo;\ntwiss, sequence=lhcb1, file=\"twiss.dat\";");

        let st = doc.get_semantic_tokens();
        let completion = doc.get_completion(Position { line: 0, character: 0 });
         
    }

    #[test]
    fn test_incomplete_env() {
        let doc = Document::new(b"option, echo;\nseqedit; flatten;\ntwiss, sequence = lhcb1;");

        let st = doc.get_semantic_tokens();
        let completion = doc.get_completion(Position { line: 1, character: 10 });

    }

    #[test]
    fn test_macros() {
        let elements = vec![
            "// test file", ";",
            "/* this is a multiline comment\n* explaining what the macro does\n* in a very detailed way */",
            "do_twiss(filename): macro = {\n  twiss, sequence=lhcb1;\n}",
            ";",
        ];
        let doc = Document::new(elements.join("\n").as_bytes());
        let expressions = doc.parser.get_elements();

        assert_eq!(doc.parser.get_element_str(&expressions[0]), elements[0]);
        assert_eq!(doc.parser.get_element_str(&expressions[2]), elements[2]);
        if let Expression::Macro(m) = &expressions[3] {
            assert_eq!(doc.parser.get_element_str(m), elements[3]);
            
        }
        else {
            assert!(false, "exprected macro, got: {:?}\nrange: {}", expressions[3], doc.parser.get_element_str(&expressions[3]));
        }
    }

    #[test]
    fn test_file_lhc_macros() {
        let doc = Document::new(include_bytes!("../tests/macros/lhc.macros.run3.madx"));


    }

    /// this test loads a job file created by our model creation, including omc3 macros and the
    /// entire lattice definition
    /// If this test runs through, most of the functionality used for creating an lhc job should be
    /// fine
    #[test]
    fn test_modelcreation() {
        let document = Document::new(include_bytes!("../tests/job.create_model.madx"));


        // now, what do we expect?
        // lhcb1/2 should be defined
        //
        assert!(document.parser.labels.contains_key(&b"lhcb1".to_vec()), "lhcb1 not defined");
        assert!(document.parser.labels.contains_key(&b"lhcb2".to_vec()), "lhcb2 not defined");
    }
}
