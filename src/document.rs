use std::collections::HashMap;
use std::path::Path;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{SemanticTokensResult, SemanticTokens, Position, CompletionResponse, CompletionItem, CompletionItemKind, MarkedString, Url};

use crate::error::UTF8_PARSER_MSG;
use crate::lexer::HasRange;
use crate::parser::{Parser, GENERIC_BUILTINS, Expression};

#[derive(Debug)]
pub struct Document {
    pub parser: Parser,
}

impl Document {
    pub fn open<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        Ok(Self{
            parser: Parser::from_path(path)?,
        })
    }

    pub fn new(text: &[u8]) -> Self {
        let mut parser = Parser::from_bytes(text.to_vec());
        Self {
            parser,
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

    pub fn get_labels_under_cursor(&self, position: Position) -> Vec<&[u8]> {
        let pos = self.parser.lexer.cursor_pos_from_text_pos(position);

        self.parser.get_elements().iter()
            .filter_map(|e| e.get_label(&pos, &self.parser))
            .collect()
    }

    pub fn get_hover(&self, labels: &Vec<&[u8]>, items: &mut Vec<MarkedString>, infile: Option<&Url>) {

        for label in labels.iter() {
            if let Some(index) = self.parser.labels.get(*label) {
                let comment = String::from_utf8(self.parser.get_element_bytes(&self.parser.get_elements()[*index-1]).to_vec()).unwrap();

                let line = self.parser.get_elements()[*index].get_range().0.line();
                let location = match infile {
                    Some(uri) => {
                        format!("{}, ", uri.path())
                    },
                    None => String::new(),
                };
                /*
                let comment = match self.parser.get_elements()[*index - 1] {
                    Expression::Comment(range) => String::from_utf8(self.parser.get_element_bytes(&range).to_vec()).unwrap(),
                    _ => String::new()
                };
                */
                items.push(MarkedString::String(
                        format!("`{}`\n\n{}\n---\ndefined in {}line {}",
                                String::from_utf8(label.to_vec()).unwrap(),
                                comment, location, line
                                )));
            }
        }

    }

    pub fn get_completion(&self, position: Position) -> Vec<CompletionItem> {
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

        for e in self.parser.get_elements() {
            e.get_completion(&pos, &mut items);
        }

        items
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

}
