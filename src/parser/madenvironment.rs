use std::collections::HashMap;

use once_cell::sync::Lazy;
use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind};

use crate::{lexer::{Token, HasRange, CursorPosition}, semantic_tokens::{get_range_token}, error::UTF8_PARSER_MSG};

use super::{Expression, MadGenericBuilder, Parser, insert_generic_builder, MadParam, MatchParam};

pub const GENERIC_ENVS: Lazy<HashMap<&'static [u8], EnvironmentBuilder>> = Lazy::new(|| {
    let mut envs = HashMap::new();

    insert_generic_env(
        &mut envs, b"seqedit", b"endedit",
        &[
        ("flatten", &[]),
        ("cycle", &["start"]),
        ("install", &["element", "class", "at", "from", "selected"])
        ],
        &["sequence"]
        );

    insert_generic_env(
        &mut envs, b"match", b"endmatch",
        &[
        ("vary", &["name", "step", "lower", "upper", "slope", "opt"]),
        ("constraint", &["sequence", "range",
         "betx", "alfx", "mux", "bety", "alfy", "muy", "x", "px", "y", "py", "dx", "dy", "dpx", "dpy"
        ]),
         ("global", &["sequence", "q1", "q2", "dq1", "dq2"]),
         ("weight", &["betx", "alfx", "mux", "bety", "alfy", "muy", "x", "px", "y", "py", "dx", "dy", "dpx", "dpy"]),
         ("lmdif", &["calls", "tolerance"]),
         ("migrad", &["calls", "tolerance", "strategy"]),
         ("simplex", &["calls", "tolerance"]),
         ("jacobian", &["calls", "tolerance", "repeat", "strategy", "cool", "balance", "random"]),
        ],
         &["sequence", "betx", "alfx", "mux", "bety", "alfy", "muy", "x", "px", "y", "py", "dx", "dy", "dpx", "dpy",
         "deltap", "slow"]
         );

    insert_generic_env(
        &mut envs, b"track", b"endtrack",
        &[
        ("start", &[
         "x", "px", "y", "py", "t", "pt",
         "fx", "phix", "fy", "phiy", "ft", "phit",
        ]),
        ("observe", &["place"]),
        ("run", &["turns", "maxaper", "ffile", "keeptrack"]),
        ("dynap", &["turns", "fastune", "lyapunov", "maxaper", "orbit"]),
        ],
        &[
        "deltap", "onepass", "damp", "quantum", "seed", "update", "onetable", "recloss", "file",
        "aperture", "dump"
        ]);

    envs
});

/// this should be a macro
pub fn insert_generic_env(map: &mut HashMap<&'static [u8], EnvironmentBuilder>,
                          match_start: &'static [u8],
                          match_end: &'static [u8],
                          generic_builders: &[(&'static str, &[&str])],
                          match_params: &[&str]) {

    let mut genericmap = HashMap::new();

    for words in generic_builders {
        insert_generic_builder(&mut genericmap, words.0.as_bytes(), words.1);
    }
    let match_params = match_params.into_iter().map(|x| x.as_bytes().to_vec()).collect::<Vec<_>>();

    map.insert(match_start, EnvironmentBuilder::new(match_start, match_end, genericmap,
                                                    match_params.into_iter().map(|p| (p, vec![])).collect()
                                                    ));
}

#[derive(Debug, PartialEq, Default)]
pub struct Environment {
    match_start: &'static [u8],
    args: Vec<MadParam>,
    start: Token,
    end: Token,
    expressions: Vec<Expression>,
}

pub struct EnvironmentBuilder {
    match_start: &'static [u8],
    match_end: &'static [u8],
    generic_builders: HashMap<&'static [u8], MadGenericBuilder>,    
    match_params: Vec<MatchParam>,
}

impl Environment {
    pub fn parse(parser: &mut Parser) -> Option<Self> {
        for (_, builder) in GENERIC_ENVS.iter() {
            if let Some(env) = builder.parse(parser) {
                return Some(env);
            }
        }
        None
    }

    pub fn get_completion(&self, pos: &CursorPosition, items: &mut Vec<CompletionItem>) {
        if &self.start.get_range().0 < pos && &self.end.get_range().1 > pos {
            for expr in self.expressions.iter() {
                expr.get_completion(pos, items);
            }     
            if let Some(builder) = &GENERIC_ENVS.get(self.match_start) {
                for name in builder.generic_builders.keys() {
                    items.push(CompletionItem{
                        label: String::from_utf8(name.to_vec()).unwrap_or_else(|_| {UTF8_PARSER_MSG.to_string()}),
                        kind: Some(CompletionItemKind::FUNCTION),
                        ..Default::default()});
                }
            }
        }

    }

    pub fn to_semantic_token(&self, semantic_tokens: &mut Vec<tower_lsp::lsp_types::SemanticToken>, pre_line: &mut u32, pre_start: &mut u32, parser: &Parser) {
        semantic_tokens.push(get_range_token(&self.start.get_range(), 7, pre_line, pre_start, parser));

        MadParam::to_semantic_token(&self.args, semantic_tokens, pre_line, pre_start, parser);
        for expr in self.expressions.iter() {
            expr.to_semantic_token(semantic_tokens, pre_line, pre_start, parser);
        }

        semantic_tokens.push(get_range_token(&self.end.get_range(), 7, pre_line, pre_start, parser));
    }

    pub(crate) fn get_label<'a>(&'a self, pos: &CursorPosition, parser: &'a Parser) -> Option<&[u8]> {
        let range = self.start.get_range();
        if &range.0 < pos && pos < &range.1 {
            return Some(parser.get_element_bytes(&range));
        }
        None
    }
}

impl HasRange for Environment {
    fn get_range(&self) -> (crate::lexer::CursorPosition, crate::lexer::CursorPosition) {
        (self.start.get_range().0, self.end.get_range().1)
    }
}


impl EnvironmentBuilder {
    pub fn new(match_start: &'static [u8], match_end: &'static [u8],
               generic_builders: HashMap<&'static [u8], MadGenericBuilder>,
               match_params: Vec<MatchParam>) -> Self {
        Self {
            match_start,
            match_end,
            generic_builders,
            match_params,
        }
    }

    pub fn parse(&self, parser: &mut Parser) -> Option<Environment> {
        if let Some(Token::Ident(name)) = parser.peek_token() {
            let mut env = Environment::default();

            if !parser.lexer.compare_range(name, self.match_start) {
                return None;
            }
            
            env.match_start = self.match_start;
            env.start = Token::Ident(*name);
            parser.advance();

            env.args = MadParam::parse_params(parser, &self.match_params);

            'l: loop {
                for (_, local) in self.generic_builders.iter() {
                    if let Some(expr) = local.parse(parser) {
                        env.expressions.push(Expression::MadGeneric(expr));
                        continue 'l;
                    }
                }
                if let Some(expr) = Expression::parse(parser) {
                    if let Expression::TokenExp(end) = &expr {
                        if parser.lexer.compare_range(end, self.match_end) {
                            env.end = end.clone();
                            return Some(env);
                        }
                    }
                    env.expressions.push(expr);
                    continue;
                } 
                break;
            }
            if let Some(last)=env.expressions.last() {
                env.end = Token::Ident(last.get_range());
            }
            return Some(env);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let parser = Parser::from_str("seqedit; flatten; endedit;");

        let seqedit = &parser.get_elements()[0];

        if let Expression::MadEnvironment(env) = seqedit {
            assert_eq!(parser.get_element_bytes(env), b"seqedit; flatten; endedit");    
            assert_eq!(parser.get_element_bytes(&env.start), b"seqedit");
            assert_eq!(parser.get_element_bytes(&env.expressions[1]), b"flatten");
            assert_eq!(parser.get_element_bytes(&env.end), b"endedit");

        }
        else {
            assert!(false, "should be an env");
        }
    }

    #[test]
    fn test_incomplete() {
        let parser = Parser::from_str("seqedit; flatten; twiss, sequence=lhcb1;");

        let seqedit = &parser.get_elements()[0];

        if let Expression::MadEnvironment(env) = seqedit {
            assert_eq!(parser.get_element_str(env), "seqedit; flatten; twiss, sequence=lhcb1;");    
            assert_eq!(parser.get_element_str(&env.start), "seqedit");
            assert_eq!(parser.get_element_str(&env.expressions[1]), "flatten");
            assert_eq!(parser.get_element_str(&env.end), ";");

            let mut st = Vec::new();
            let mut pre_line = 0;
            let mut pre_start = 0;
            env.to_semantic_token(&mut st, &mut pre_line, &mut pre_start, &parser);

            //assert!(false, "{:#?}\n{:#?}", env.expressions, st);
        }
        else {
            assert!(false, "should be an env");
        }
    }
}
