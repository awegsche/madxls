use std::collections::HashMap;

use once_cell::sync::Lazy;

use crate::lexer::{HasRange, Token};

use super::{insert_generic_builder, Expression, MadGenericBuilder, MadParam, MatchParam, Parser};

pub const GENERIC_ENVS: Lazy<HashMap<&'static [u8], EnvironmentBuilder>> = Lazy::new(|| {
    let mut envs = HashMap::new();

    insert_generic_env(
        &mut envs,
        b"seqedit",
        b"endedit",
        &[
            ("flatten", &[]),
            ("cycle", &["start"]),
            ("install", &["element", "class", "at", "from", "selected"]),
        ],
        &["sequence"],
    );

    insert_generic_env(
        &mut envs,
        b"match",
        b"endmatch",
        &[
            ("vary", &["name", "step", "lower", "upper", "slope", "opt"]),
            (
                "constraint",
                &[
                    "sequence", "range", "betx", "alfx", "mux", "bety", "alfy", "muy", "x", "px",
                    "y", "py", "dx", "dy", "dpx", "dpy",
                ],
            ),
            ("global", &["sequence", "q1", "q2", "dq1", "dq2"]),
            (
                "weight",
                &[
                    "betx", "alfx", "mux", "bety", "alfy", "muy", "x", "px", "y", "py", "dx", "dy",
                    "dpx", "dpy",
                ],
            ),
            ("lmdif", &["calls", "tolerance"]),
            ("migrad", &["calls", "tolerance", "strategy"]),
            ("simplex", &["calls", "tolerance"]),
            (
                "jacobian",
                &[
                    "calls",
                    "tolerance",
                    "repeat",
                    "strategy",
                    "cool",
                    "balance",
                    "random",
                ],
            ),
        ],
        &[
            "sequence", "betx", "alfx", "mux", "bety", "alfy", "muy", "x", "px", "y", "py", "dx",
            "dy", "dpx", "dpy", "deltap", "slow",
        ],
    );

    insert_generic_env(
        &mut envs,
        b"track",
        b"endtrack",
        &[
            (
                "start",
                &[
                    "x", "px", "y", "py", "t", "pt", "fx", "phix", "fy", "phiy", "ft", "phit",
                ],
            ),
            ("observe", &["place"]),
            ("run", &["turns", "maxaper", "ffile", "keeptrack"]),
            (
                "dynap",
                &["turns", "fastune", "lyapunov", "maxaper", "orbit"],
            ),
        ],
        &[
            "deltap", "onepass", "damp", "quantum", "seed", "update", "onetable", "recloss",
            "file", "aperture", "dump",
        ],
    );

    insert_generic_env(
        &mut envs,
        b"ptc_create_universe",
        b"ptc_end",
        &[
            (
                "ptc_create_layout",
                &[
                    "time",
                    "model",
                    "method",
                    "nst",
                    "exact",
                    "offset_deltap",
                    "errors_out",
                    "magnet_name",
                    "resplit",
                    "thin",
                    "xbend",
                    "even",
                ],
            ),
            ("ptc_move_to_layout", &["index"]),
            ("ptc_read_errors", &["overwrite"]),
            ("ptc_align", &[]),
            (
                "ptc_start",
                &[
                    "x", "y", "px", "py", "t", "pt", "fx", "phix", "phiy", "ft", "fy", "phit",
                ],
            ),
            ("ptc_observe", &["place"]),
            (
                "ptc_track",
                &[
                    "icase",
                    "deltap",
                    "closed_orbit",
                    "element_by_element",
                    "turns",
                    "dump",
                    "onetable",
                    "maxaper",
                    "norm",
                    "norm_out",
                    "file",
                    "extension",
                    "ffile",
                    "radiation",
                    "radiation_model1",
                    "radiation_energy_loss",
                    "radiation_quadr",
                    "beam_envelope",
                    "space_charge",
                ],
            ),
            (
                "ptc_track_line",
                &[
                    "turns",
                    "onetable",
                    "file",
                    "extension",
                    "rootntuple",
                    "everystep",
                    "tableallsteps",
                    "gcs",
                ],
            ),
            ("ptc_track_end", &[]),
            (
                "ptc_twiss",
                &[
                    "icase",
                    "deltap",
                    "closed_orbit",
                    "deltap_dependency",
                    "slice_magnets",
                    "range",
                    "file",
                    "table",
                    "initial_matrix_table",
                    "initial_matrix_manual",
                    "initial_map_manual",
                    "beta0",
                    "maptable",
                    "ignore_map_orbit",
                    "ring_parameters",
                    "betx",
                    "bety",
                    "alfx",
                    "alfy",
                    "mux",
                    "muy",
                    "dx",
                    "dy",
                    "dpx",
                    "dpy",
                    "x",
                    "y",
                    "px",
                    "py",
                    "t",
                    "pt",
                    // todo: see what can be done about re11 .. re66
                ],
            ),
        ],
        &["sector_nmul_max", "sector_nmul", "ntpsa", "symprint"],
    );
    envs
});

/// this should be a macro
pub fn insert_generic_env(
    map: &mut HashMap<&'static [u8], EnvironmentBuilder>,
    match_start: &'static [u8],
    match_end: &'static [u8],
    generic_builders: &[(&'static str, &[&str])],
    match_params: &[&str],
) {
    let mut genericmap = HashMap::new();

    for words in generic_builders {
        insert_generic_builder(&mut genericmap, words.0.as_bytes(), words.1);
    }
    let match_params = match_params
        .into_iter()
        .map(|x| x.as_bytes().to_vec())
        .collect::<Vec<_>>();

    map.insert(
        match_start,
        EnvironmentBuilder::new(
            match_start,
            match_end,
            genericmap,
            match_params.into_iter().map(|p| (p, vec![])).collect(),
        ),
    );
}

#[derive(Debug, PartialEq, Default)]
pub struct Environment {
    match_start: &'static [u8],
    args: Vec<MadParam>,
    start: Token,
    end: Token,
    pub expressions: Vec<Expression>,
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

    pub(crate) fn accept<V: crate::visitor::Visitor>(
        &self,
        visitor: &mut V,
        parser: &crate::parser::Parser,
    ) {
        for e in self.expressions.iter() {
            e.accept(visitor, parser);
        }
    }
}

impl HasRange for Environment {
    fn get_range(&self) -> (crate::lexer::CursorPosition, crate::lexer::CursorPosition) {
        (self.start.get_range().0, self.end.get_range().1)
    }
}

impl EnvironmentBuilder {
    pub fn new(
        match_start: &'static [u8],
        match_end: &'static [u8],
        generic_builders: HashMap<&'static [u8], MadGenericBuilder>,
        match_params: Vec<MatchParam>,
    ) -> Self {
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
            if let Some(last) = env.expressions.last() {
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
        } else {
            assert!(false, "should be an env");
        }
    }

    #[test]
    fn test_incomplete() {
        let parser = Parser::from_str("seqedit; flatten; twiss, sequence=lhcb1;");

        let seqedit = &parser.get_elements()[0];

        if let Expression::MadEnvironment(env) = seqedit {
            assert_eq!(
                parser.get_element_str(env),
                "seqedit; flatten; twiss, sequence=lhcb1;"
            );
            assert_eq!(parser.get_element_str(&env.start), "seqedit");
            assert_eq!(parser.get_element_str(&env.expressions[1]), "flatten");
            assert_eq!(parser.get_element_str(&env.end), ";");
        } else {
            assert!(false, "should be an env");
        }
    }
}
