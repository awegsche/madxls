use crate::lexer::{HasRange, Token};

use super::{MadGeneric, Parser};

#[derive(Debug, PartialEq)]
pub struct Label {
    pub name: Token,
    pub command: MadGeneric,
}

impl Label {
    pub fn parse(parser: &mut Parser) -> Option<Self> {
        if let Some(name) = parser.peek_token().cloned() {
            if !name.is_ident() {
                return None;
            }
            parser.advance();

            if let Some(Token::Colon(_)) = parser.peek_token() {
                parser.advance();

                // try parsing as MAdGeneric
                if let Some(mad_generic) = MadGeneric::parse(parser) {
                    return Some(Self {
                        name: name,
                        command: mad_generic,
                    });
                }
            }

            parser.go_back();
        }
        None
    }

    pub(crate) fn accept<V: crate::visitor::Visitor>(
        &self,
        visitor: &mut V,
        parser: &crate::parser::Parser,
    ) {
        visitor.visit_label(self, parser);
    }
}

impl HasRange for Label {
    fn get_range(&self) -> (crate::lexer::CursorPosition, crate::lexer::CursorPosition) {
        (self.name.get_range().0, self.command.get_range().1)
    }
}
