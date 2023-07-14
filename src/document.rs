use std::collections::HashMap;
use std::path::Path;
use std::string::ParseError;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{SemanticTokensResult, SemanticTokens, Position, CompletionResponse, CompletionItem, CompletionItemKind, MarkedString, Url, Diagnostic, Range, DiagnosticSeverity};

use crate::error::UTF8_PARSER_MSG;
use crate::lexer::HasRange;
use crate::parser::{Parser, GENERIC_BUILTINS, Expression, Problem, MaybeProblem};

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
        Self {
            parser: Parser::from_bytes(text.to_vec())
        }
    }

    pub fn reload(&mut self, text: &[u8]) {
        self.parser = Parser::from_bytes(text.to_vec());
        //self.parser.scan_includes();
    }

    pub fn get_diagnostics(&self) -> Vec<MaybeProblem> {
        self.parser.problems.iter().map(|p| {
            let range = match p {
                crate::parser::Problem::MissingCallee(_, range) => range,
                crate::parser::Problem::InvalidParam(range) => range,
                crate::parser::Problem::Error(_, _, _) => todo!(),
                crate::parser::Problem::Warning(_, _, _) => todo!(),
                crate::parser::Problem::Hint(_, _, _) => todo!(),
            };
            MaybeProblem{
            problem: Some(p.clone()),
            range: Range::new(
                self.parser.lexer.cursor_pos_to_text_pos(range.0),
                self.parser.lexer.cursor_pos_to_text_pos(range.1)
                )
            }
        }).collect()
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

    /// gets the hover information for the given set of labels
    pub fn get_hover(&self, labels: &Vec<&[u8]>, items: &mut Vec<MarkedString>, infile: Option<&Url>) {

        for label in labels.iter() {
            // first, look in named labels
            if let Some(index) = self.parser.labels.get(*label) {
                let comment = if *index > 0 {
                    self.parser.get_element_str(&self.parser.get_elements()[*index-1]).to_string()}
                else {
                     String::new()
                };
                                    
                let element = &self.parser.get_elements()[*index];

                let line = element.get_range().0.line();
                let location = match infile {
                    Some(uri) => {
                        format!("\"{}\", ", uri.path())
                    },
                    None => String::new(),
                };
                /*
                let comment = match self.parser.get_elements()[*index - 1] {
                    Expression::Comment(range) => String::from_utf8(self.parser.get_element_bytes(&range).to_vec()).unwrap(),
                    _ => String::new()
                };
                */


                let signature = match element {
                    Expression::Label(l) => format!("`{}`  : **LABEL**", self.parser.get_element_str(l)),
                    Expression::Macro(m) => format!("`{}{}`  : **MACRO**",
                                                    self.parser.get_element_str(&m.name),
                                                    self.parser.get_element_str(&(m.parenopen, m.parenclose+1)),
                                                    ),
                    Expression::Assignment(a) => format!("`{}`", self.parser.get_element_str(a)),
                    Expression::String(_) => todo!(),
                    Expression::Comment(_) => todo!(),
                    Expression::Symbol(s) => s.clone(),
                    Expression::MadGeneric(_) => todo!(),
                    Expression::MadEnvironment(_) => todo!(),
                    Expression::Exit(_) => todo!(),
                    Expression::Operator(_) => todo!(),
                    Expression::TokenExp(_) => todo!(),
                    Expression::Exec(_) => todo!(),
                };

                items.push(MarkedString::String(
                        format!("{}\n---\n{}\n---\ndefined in {}line {}",
                                signature, comment, location, line
                                )));
            }
        }

    }

    pub fn get_completion(&self, position: Option<Position>) -> Vec<CompletionItem> {
        let mut items = Vec::new();
        for label in self.parser.labels.keys() {
            items.push(CompletionItem{
                label: String::from_utf8(label.clone()).unwrap_or_else(|_| {UTF8_PARSER_MSG.to_string()}),
                kind: Some(CompletionItemKind::VARIABLE),
                ..Default::default()
            });

        }

        if let Some(position) = position {
            let pos = self.parser.lexer.cursor_pos_from_text_pos(position);
            log::debug!("completion triggered at {:#?}", pos);
            for name in GENERIC_BUILTINS.keys() {
                items.push(CompletionItem{
                    label: String::from_utf8(name.to_vec()).unwrap_or_else(|_| {UTF8_PARSER_MSG.to_string()}),
                    kind: Some(CompletionItemKind::FUNCTION),
                    ..Default::default()});
            }
            for e in self.parser.get_elements() {
                e.get_completion(&pos, &mut items);
            }
        }

        items
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_simple() {
        let doc = Document::new(b"option, echo;\ntwiss, sequence=lhcb1, file=\"twiss.dat\";");

        let st = doc.get_semantic_tokens();
        let completion = doc.get_completion(Some(Position { line: 0, character: 0 }));
         
    }

    #[test]
    fn test_incomplete_env() {
        let doc = Document::new(b"option, echo;\nseqedit; flatten;\ntwiss, sequence = lhcb1;");

        let st = doc.get_semantic_tokens();
        let completion = doc.get_completion(Some(Position { line: 1, character: 10 }));

    }

    #[test]
    fn test_incomplete_generic() {
        let doc = Document::new(b"option, echo;\ncall, fi");

        let st = doc.get_semantic_tokens();
        let completion = doc.get_completion(Some(Position { line: 1, character: 21 }));

        for i in completion.iter() {
            if i.label == "file" {
                return;
            }
        }

        assert!(false, "didn't provide 'file' as possible completion\ncompletions are:\n{:?}\nsemantic tokens:\n{:?}",
                completion.iter().filter(|c| c.kind.unwrap() == CompletionItemKind::FIELD).collect::<Vec<_>>(),
                st
                );

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

    #[test]
    fn get_hover() {
        let doc = Document::new(b"
do_twiss(a,b): macro = { twiss, sequence=lhcb1;};
option, echo;

exec, do_twiss(0, 0);
                                ");

        let labels = doc.get_labels_under_cursor(Position::new(5, 9));

        assert_eq!(labels, [b"do_twiss"]);
        let mut items = vec![];
        let uri = Url::from_file_path("/home").unwrap();
        doc.get_hover(&labels, &mut items, Some(&uri));

        for item in items.iter() {
            if let MarkedString::String(s) = item {
                if s == "`do_twiss(a,b)`  : **MACRO**\n---\n\n---\ndefined in \"/home\", line 1" { return; }
            }
        }
        assert!(false, "expected macro do_twiss in hover, items: {:?}", items);
    }

}
