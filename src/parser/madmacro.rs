use tower_lsp::lsp_types::{CompletionItem, SemanticToken};

use crate::{lexer::{Token, CursorPosition, HasRange}, semantic_tokens::get_range_token};

use super::{Expression, Parser, Assignment, Problem};

#[derive(Debug, PartialEq, Default)]
pub struct Macro {
    pub name: Token,
    pub parenopen: CursorPosition,
    pub args: Vec<Token>,
    pub parenclose: CursorPosition,
    pub macro_pos: Token,
    pub body: Vec<Expression>,
    pub end: CursorPosition,
}



impl Macro {
    pub fn parse(parser: &mut Parser) -> Option<Self> {
        let before = parser.get_position();

        if let Some(m) = Self::parse_inner(parser) {
            return Some(m);
        }
        parser.set_position(before);
        None
    }

    pub fn parse_inner(parser: &mut Parser) -> Option<Self> {
        if let Some(token) = parser.peek_token() {
            if !token.is_ident() {
                return None;
            }
            let mut m = Self::default();
            m.name = token.clone();
            parser.advance();

            if let Some((parenopen, tokens, parenclose)) = Self::read_parenthesis(parser) {
                m.parenopen = parenopen;
                m.args = tokens;
                m.parenclose = parenclose;

            }
            else {
                return None;
            }

            if let Some(Token::Colon(_)) = parser.peek_token() {
                parser.advance();
            }
            else {
                return None;
            }

            if let Some(Token::Ident(macro_name)) = parser.peek_token() {
                if parser.lexer.compare_range(macro_name, b"macro") {
                    m.macro_pos = Token::Ident(macro_name.clone());
                    parser.advance();
                }
                else {
                    return None;
                }
            }
            else {
                return None;
            }

            if let Some(Token::Equal(_)) = parser.peek_token() {
                parser.advance();
            }
            else {
                return None;
            }


            if let Some(Token::BraceOpen(_)) = parser.peek_token() {
                parser.advance();
            }
            else {
                return None;
            }

            while let Some(expr) = Assignment::parse(parser) {
                if let Expression::TokenExp(Token::BraceClose(end)) = expr {
                    m.end = end + 1;
                    break;
                }
                m.body.push(expr);
            }

            return Some(m);
        }
        None
    }

    pub fn read_parenthesis(parser: &mut Parser) -> Option<(CursorPosition, Vec<Token>, CursorPosition)> {

        let mut start = CursorPosition::default(); 
        let mut end = CursorPosition::default(); 
        let mut tokens = Vec::new();

        if let Some(Token::ParentOpen(parenopen)) = parser.peek_token() {
            start = *parenopen;
            parser.advance();
        }
        else {
            return None;
        }

        while let Some(token) = parser.peek_token().cloned() {
            parser.advance();
            match token {
                Token::ParentClose(parenclose) => {
                    end = parenclose;

                    return Some((start, tokens, end));
                }
                Token::Ident(ident) => {
                    tokens.push(Token::Ident(ident));
                }
                _ => {}
            }
        }

        // if we get here, there was no close parenthesis
        // and we reached EOF
        //
        None

    }

    pub fn get_completion(&self, pos: &CursorPosition, items: &mut Vec<CompletionItem>) {
        for e in self.body.iter() {
            e.get_completion(pos, items);
        }
    }

    pub fn to_semantic_token(&self, semantic_tokens: &mut Vec<SemanticToken>, pre_line: &mut u32, pre_start: &mut u32, parser: &Parser) {

        semantic_tokens.push(get_range_token(&self.name, 4, pre_line, pre_start, parser));
        semantic_tokens.push(get_range_token(&self.macro_pos, 8, pre_line, pre_start, parser));

        for e in self.body.iter() {
            e.to_semantic_token(semantic_tokens, pre_line, pre_start, parser);
        }

        // this doesn't work
        /*
        let mut arg_tokens = Vec::new();
        let start_inner = self.macro_pos.get_range().1;
        if let Ok(inner_text) = String::from_utf8(parser.get_element_bytes(&(start_inner, self.end))
            .to_ascii_lowercase()) {

            let mut pline = *pre_line;
            let mut pstart = *pre_start;

            for arg in self.args.iter()
                .filter_map(|arg| String::from_utf8(parser.get_element_bytes(arg).to_ascii_lowercase()).ok()) {
                    log::debug!("look for arg {}", arg);
                    arg_tokens.extend(inner_text.match_indices(&arg).map(|(idx, _)| {
                        let mut pos0 = start_inner;
                        let mut pos1 = start_inner;
                        parser.lexer.advance_cursor(&mut pos0, idx);
                        parser.lexer.advance_cursor(&mut pos1, idx + arg.len());
                        get_range_token(&(pos0, pos1), 9, &mut pline, &mut pstart, parser)
                    }));
                }
        }



        let mut expr_tokens = Vec::new();

        for e in self.body.iter() {
            e.to_semantic_token(&mut expr_tokens, pre_line, pre_start, parser);
        }

        let mut arg_iter = arg_tokens.iter_mut();
        let mut exp_iter = expr_tokens.iter_mut();

        let mut next_arg_token = arg_iter.next();
        let mut next_exp_token = exp_iter.next();

        loop {
            match (next_arg_token, next_exp_token) {
            (Some(arg), Some(exp)) => {
                if arg.delta_line < exp.delta_line {
                    exp.delta_line -= arg.delta_line;
                    semantic_tokens.push(*arg);
                    next_arg_token = arg_iter.next();
                    next_exp_token = Some(exp);
                    log::debug!("push arg, delta line");
                }
                else if arg.delta_start < exp.delta_start {
                    exp.delta_start -= arg.delta_start;
                    if arg.length > exp.delta_start { arg.length = exp.delta_start }
                    semantic_tokens.push(*arg);
                    next_arg_token = arg_iter.next();
                    next_exp_token = Some(exp);
                    log::debug!("push arg, delta start");
                }
                else if arg.delta_line > exp.delta_line{
                    arg.delta_line -= exp.delta_line;
                    semantic_tokens.push(*exp);
                    next_exp_token = exp_iter.next();
                    next_arg_token = Some(arg);
                    log::debug!("push exp, deltaline");
                }
                else {
                    arg.delta_start -= exp.delta_start;
                    if exp.length > arg.delta_start { exp.length = arg.delta_start }
                    semantic_tokens.push(*exp);
                    next_exp_token = exp_iter.next();
                    next_arg_token = Some(arg);
                    log::debug!("push exp, deltastart");
                }
                continue;
            },
            (Some(arg), None) => {
                semantic_tokens.push(*arg);
                next_arg_token = arg_iter.next();
                next_exp_token = None;
                log::debug!("push arg, exps exhausted");
                continue;
            },
            (None, Some(exp)) => {
                semantic_tokens.push(*exp);
                next_exp_token = exp_iter.next();
                next_arg_token = None;
                log::debug!("push exp, args exhausted");
                continue;
            },
            (None, None) => break,
            }

        }
    */
    }

    pub(crate) fn get_problems(&self, problems: &mut Vec<Problem>) {
        for e in self.body.iter() {
            e.get_problems(problems);
        }
    }

    pub(crate) fn get_highlights(&self, pos: &CursorPosition, parser: &Parser) -> Vec<(CursorPosition, CursorPosition)> {
        
        let mut arg_tokens: Vec<(CursorPosition, CursorPosition)> = Vec::new();
        let mut arg_tokens = Vec::new();
        let start_inner = self.macro_pos.get_range().1;
        if let Ok(inner_text) = String::from_utf8(parser.get_element_bytes(&(start_inner, self.end))
            .to_ascii_lowercase()) {


            for arg in self.args.iter()
                .filter_map(|arg| String::from_utf8(parser.get_element_bytes(arg).to_ascii_lowercase()).ok()) {
                    log::debug!("look for arg {}", arg);
                    arg_tokens.extend(inner_text.match_indices(&arg).map(|(idx, _)| {
                        let mut pos0 = start_inner;
                        let mut pos1 = start_inner;
                        parser.lexer.advance_cursor(&mut pos0, idx);
                        parser.lexer.advance_cursor(&mut pos1, idx + arg.len());
                        (pos0, pos1)
                    }));
                }
        }

        arg_tokens

    }

    pub(crate) fn accept<V: crate::visitor::Visitor>(&self, visitor: &mut V) {
        visitor.visit_macro(self);
        for e in self.body.iter() {
            e.accept(visitor);
        }
    }
}

impl HasRange for Macro {
    fn get_range(&self) -> (CursorPosition, CursorPosition) {
        (self.name.get_range().0, self.end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn match_macro() {

        let parser = Parser::from_str("m1(a, b): macro = {\n twiss,sequence=lhcb1;\na=b;\n}");


        let m = &parser.get_elements()[0];

        if let Expression::Macro(m) = m {
            assert_eq!(parser.get_element_str(&m.name), "m1");
            assert_eq!(parser.get_element_str(m), "m1(a, b): macro = {\n twiss,sequence=lhcb1;\na=b;\n}");

            //assert!(false, "m: {:?}\n\n element after:\n{:?}", m, parser.get_elements()[1]);
        } 
        else {
            assert!(false, "should be macro");
        }
        assert_eq!(parser.get_elements().len(), 1);
    }
}
