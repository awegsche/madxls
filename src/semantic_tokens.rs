use tower_lsp::lsp_types::SemanticToken;

use crate::{lexer::HasRange, parser::Parser};

pub fn get_range_token<R: HasRange>( token: &R, token_type: u32, pline: &mut u32,
                                     pstart: &mut u32, parser: &Parser) -> SemanticToken {

    let range = token.get_range();
    let line = range.0.line() as u32;
    let start = range.0.character(parser.lexer.lines()) as u32;
    let delta_line = line - *pline;
    let length = (range.1.absolute() - range.0.absolute()) as u32;
    let delta_start = if delta_line == 0 {
        start - *pstart
    }
    else {
        start
    };


    let token = SemanticToken {
        delta_line,
        delta_start,
        length,
        token_type,
        token_modifiers_bitset: 0
    };

    *pline = line;
    *pstart = start;

    token
}

