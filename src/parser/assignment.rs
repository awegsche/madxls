use crate::lexer::{Token, HasRange, CursorPosition};

use super::{Expression, Parser, Problem};

#[derive(Debug, PartialEq)]
pub struct Assignment {
    pub lhs: Box<Expression>,
    pub rhs: Option<Box<Expression>>,
}

impl Assignment {
    pub fn parse(parser: &mut Parser) -> Option<Expression> {
        
        if let Some(expr) = Expression::parse(parser) {
            if let Some(token) = parser.peek_token() {
                if !token.is_assignment() {
                    //parser.go_back();
                    return Some(expr);
                }
                parser.advance();

                if let Some(right) = Expression::parse(parser) {
                    return Some(Expression::Assignment(Self {
                        lhs: Box::new(expr),
                        rhs: Some(Box::new(right)),
                    }));
                }
                else {
                    return Some(Expression::Assignment(Self {
                        lhs: Box::new(expr),
                        rhs: None,
                    }));
                }
            }
            return Some(expr);
        }
        None
    }

    pub(crate) fn get_label<'a>(&'a self, pos: &CursorPosition, parser: &'a Parser) -> Option<&[u8]> {
        if let Some(rhs) = &self.rhs {
            return rhs.get_label(pos, parser);
        }
        None
    }

    pub(crate) fn get_problems(&self, problems: &mut Vec<Problem>) {
        if let Some(e) = &self.rhs {
            e.get_problems(problems);
        }
    }
}

impl HasRange for Assignment{
    fn get_range(&self) -> (crate::lexer::CursorPosition, crate::lexer::CursorPosition) {
        if let Some(rhs) = self.rhs.as_ref() {
            (self.lhs.get_range().0, rhs.get_range().1)
        }
        else {
            self.lhs.get_range()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assignment() {
        let parser = Parser::from_str("a = 1");

        assert_eq!(parser.get_elements().len(), 1);
        if let Expression::Assignment(assignment) = &parser.get_elements()[0] {
            assert_eq!(parser.get_element_bytes(assignment), b"a = 1");
            assert_eq!(parser.get_element_bytes(&*assignment.lhs), b"a");
            assert_eq!(parser.get_element_bytes(&**assignment.rhs.as_ref().unwrap()), b"1");
        }
    }
}
