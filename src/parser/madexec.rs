use crate::lexer::{HasRange, Token, CursorPosition};

use super::{Macro, Problem, Expression};

#[derive(Debug, Eq, PartialEq, Clone, Default)]
pub struct MadExec {
    name: Token,
    callee: Token,
    parenopen: CursorPosition,
    args: Vec<Token>,
    parenclose: CursorPosition,
}

impl MadExec {

    pub(crate) fn get_problems(&self, problems: &mut Vec<Problem>) {
        problems.push(Problem::MissingCallee(vec![], self.callee.get_range()));
    }

    pub(crate) fn parse(parser: &mut super::Parser) -> Option<MadExec> {
        if let Some(token) = parser.peek_token() {
            if parser.lexer.compare_range(token, b"exec") {

                let mut exec = Self::default();
                exec.name = token.clone();
                parser.advance();


                if let Some(Token::Komma(_)) = parser.peek_token() {
                    parser.advance();
                }
                else {
                    return Some(exec);
                }


                if let Some(Token::Ident(ident)) = parser.peek_token() {
                    exec.callee = Token::Ident(ident.clone());
                    parser.advance();
                }
                else {
                    return Some(exec);
                }

                if let Some((popen, args, pclose)) = Macro::read_parenthesis(parser) {
                    exec.parenopen = popen;
                    exec.parenclose = pclose;
                    exec.args = args;
                }

                return Some(exec);
            }
        }
        None
    }

    pub(crate) fn get_label<'a>(&'a self, pos: &CursorPosition, parser: &'a super::Parser) -> Option<&[u8]> {
        let range = self.callee.get_range();
        if &range.0 < pos && pos < &range.1 {
            Some(parser.get_element_bytes(&range))
        }
        else {
            None
        }
    }
}

impl HasRange for MadExec {
    fn get_range(&self) -> (crate::lexer::CursorPosition, crate::lexer::CursorPosition) {
        (self.name.get_range().0, self. parenclose)
    }
}
