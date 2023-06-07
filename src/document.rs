use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{SemanticTokensResult, SemanticTokens, Position, CompletionResponse, CompletionItem, CompletionItemKind};

use crate::error::UTF8_PARSER_MSG;
use crate::lexer::HasRange;
use crate::parser::{Parser, GENERIC_BUILTINS, Expression};

#[derive(Debug)]
pub struct Document {
    pub parser: Parser,
}

impl Document {
    pub fn new(text: &[u8]) -> Self {
        Self {
            parser: Parser::from_bytes(text.to_vec()),
        }
    }

    pub fn reload(&mut self, text: &[u8]) {
        self.parser = Parser::from_bytes(text.to_vec());
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
        log::info!("data: {:#?}", data);

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
    fn test_modelcreation() {
    }
}
