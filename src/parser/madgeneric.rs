use std::collections::HashMap;

use once_cell::sync::Lazy;
use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind};

use crate::{lexer::{Token, CursorPosition, HasRange}, semantic_tokens::{get_range_token}, error::UTF8_PARSER_MSG};

use super::{Expression, Parser, Problem};

pub type MatchParam = (Vec<u8>, Vec<Vec<u8>>);

// ---- const map of generic madx commands ---------------------------------------------------------

pub const GENERIC_BUILTINS: Lazy<HashMap<&'static [u8], MadGenericBuilder>> = Lazy::new(|| {
    let mut builtins = HashMap::new();
    insert_generic_builder(&mut builtins, b"option", &["echo", "warn", "verbose", "debug", "echomacro",
                           "trace", "verify", "tell", "reset", "no_fatal_stop", "keep_exp_move", "rbarc", "thin_foc", "bborbit", "sympl",
                           "twiss_print", "threader"]);
    insert_generic_builder(&mut builtins, b"set", &["format", "sequence"]);
    insert_generic_builder(&mut builtins, b"use", &["sequence", "period", "survey", "range"]);
    insert_generic_builder(&mut builtins, b"select", &["flag", "range", "class", "pattern", "sequence",
                           "full", "clear", "column", "slice", "thick", "step", "at", "seqedit", "error", "makethin",
                           "sectormap", "save", "interpolate", "twiss"]);
    insert_generic_builder(&mut builtins, b"assign", &["echo", "truncate"]);
    insert_generic_builder(&mut builtins, b"call", &["file"]);
    insert_generic_builder(&mut builtins, b"print", &["text"]);
    insert_generic_builder(&mut builtins, b"printf", &["text", "value"]);
    insert_generic_builder(&mut builtins, b"renamefile", &["file", "to"]);
    insert_generic_builder(&mut builtins, b"copyfile", &["file", "to", "append"]);
    insert_generic_builder(&mut builtins, b"create", &["table", "column"]);
    insert_generic_builder(&mut builtins, b"delete", &["table", "sequence"]);
    insert_generic_builder(&mut builtins, b"readmytable", &["table", "file"]);
    insert_generic_builder(&mut builtins, b"twiss", &["sequence", "line", "range",
                           "deltap", "chrom", "centre", "tolerance", "file", "table", "notable",
                           "rmatrix", "sectormap", "sectortable", "sectorfile", "sectorpure",
                           "eigenvector", "eigenfile", "keeporbit", "useorbit", "couple", "exact",
                           "ripken", "tapering"]);
    insert_generic_builder(&mut builtins, b"fill", &["table", "row"]);
    insert_generic_builder(&mut builtins, b"setvars", &["table", "row", "knob", "const", "noappend"]);
    insert_generic_builder(&mut builtins, b"fill_knob", &["table", "row", "knob", "scale"]);
    insert_generic_builder(&mut builtins, b"setvars_lin", &["table", "row1", "row2", "param"]);

    insert_generic_builder(&mut builtins, b"beam", &["particle", "mass", "charge",
                           "energy", "pc", "gamma", "beta", "brho",
                           "ex", "ey",
                           "exn", "eyn",
                           "et", "sigt", "sigt",
                           "kbunch", "npart", "bcurrent",
                           "bunched", "radiate", "bv",
                           "sequence",
                           "positron", "electron", "proton", "antiproton", "posmuon", "negmuon", "ion"]);
    insert_generic_builder(&mut builtins, b"resbeam", &["sequence"]);
    insert_generic_builder(&mut builtins, b"chdir", &["dir"]);

    // ---- lattice elements -----------------------------------------------------------------------
    insert_generic_builder(&mut builtins, b"rbend", &["l", "angle", "tilt",
                           "k0", "k0s", "k1", "k1s", "k2", "k2s", "e1", "e2", "fint", "fintx",
                           "hgap", "h1", "h2", "thick", "add_angle", "kill_ent_fringe"
    ]);
    insert_generic_builder(&mut builtins, b"rbend", &["l", "angle", "tilt",
                           "k0", "k0s", "k1", "k1s", "e1", "e2", "fint", "fintx",
                           "hgap", "h1", "h2", "thick", "kill_ent_fringe"
    ]);
    insert_generic_builder(&mut builtins, b"drift", &["l"]);
    insert_generic_builder(&mut builtins, b"dipedge", &["h", "e1", "fint", "hgap", "tilt"]);
    insert_generic_builder(&mut builtins, b"quadrupole", &["l", "k1", "k1s", "tilt", "thick"]);
    insert_generic_builder(&mut builtins, b"sextupole", &["l", "k2", "k2s", "tilt"]);
    insert_generic_builder(&mut builtins, b"octupole", &["l", "k3", "k3s", "tilt"]);
    insert_generic_builder(&mut builtins, b"multipole", &["lrad", "knl", "ksl", "tilt"]);
    insert_generic_builder(&mut builtins, b"solenoid", &["l", "ks", "ksi"]);
    insert_generic_builder(&mut builtins, b"nllens", &["knll", "kcll"]);
    insert_generic_builder(&mut builtins, b"hkicker", &["l", "tilt", "sinkick", "kick", "sintune", "sinpeak", "sinphase"]);
    insert_generic_builder(&mut builtins, b"vkicker", &["l", "tilt", "sinkick", "kick", "sintune", "sinpeak", "sinphase"]);
    insert_generic_builder(&mut builtins, b"kicker", &["l", "hkick", "vkick", "tilt"]);
    insert_generic_builder(&mut builtins, b"rfcavity", &["l", "volt", "lag", "freq", "harmon", "n_bessel", "no_cavity_totalpath"]);
    insert_generic_builder(&mut builtins, b"twcavity", &["l", "volt", "lag", "freq", "psi", "delta_lag"]);
    insert_generic_builder(&mut builtins, b"crabcavity", &["l", "volt", "lag", "freq", "harmon",
                           "rv1", "rv2", "rv3", "rv4", "rph1","rph2", "lagf"]);
    insert_generic_builder(&mut builtins, b"hacdipole", &["l", "volt", "lag", "freq", "ramp1","ramp2","ramp3","ramp4", ]);
    insert_generic_builder(&mut builtins, b"vacdipole", &["l", "volt", "lag", "freq", "ramp1","ramp2","ramp3","ramp4", ]);
    insert_generic_builder(&mut builtins, b"rfmultipole", &["l", "volt", "lag", "freq", "harmon", "lrad", "tilt",
                           "knl", "ksl", "pnl", "psl"]);
    insert_generic_builder(&mut builtins, b"save", &["file"]);


    builtins  
});

// ---- structs ------------------------------------------------------------------------------------

/// Represents a generic MadX command.
/// This class is used to easily define the MadX syntax.
/// Certain important MadX commands will be represented by their own struct.
///
/// A Madx Command is of the form
/// `COMMANDNAME {, ATTRIBUTE}*;`
///
/// where an ATTRIBUTE is represented by the `MadParam` struct below.
///
/// # Creation
///
/// The insert_generic method can be used to insert a generic MadX command  into a map (via the
/// `MadGenericBuilder` helper struct).
#[derive(Debug, PartialEq)]
pub struct MadGeneric{
    pub match_name: &'static [u8],
    pub name: Token,
    pub args: Vec<MadParam>,
}

/// Represents a parameter of a MadX command.\
///
/// The syntax is `ATTRIBUTE = EXPRESSION`,
/// where, for boolean flags, the assignment is optional and
/// `ATTRIBUTE` -> `ATTRIBUTE = true`
/// `-ATTRIBUTE` -> `ATTRIBUTE = false`
///
#[derive(Debug, PartialEq)]
pub struct MadParam{
    pub valid: bool,
    pub sign: Option<Token>,
    pub attribute: Token,
    pub value: Option<Box<Expression>>,
}

pub struct MadGenericBuilder {
    pub match_name: &'static [u8],
    pub match_params: Vec<MatchParam>,
}

// ---- impls --------------------------------------------------------------------------------------

// ---- MadGeneric ---------------------------------------------------------------------------------
impl MadGeneric {
    pub fn parse(parser: &mut Parser) -> Option<Self> {
        for (_, builder) in GENERIC_BUILTINS.iter() {
            if let Some(builtin) = builder.parse(parser){
                return Some(builtin);
            }
        }
        None
    }

    pub fn get_completion(&self, pos: &CursorPosition, items: &mut Vec<CompletionItem>) {
        let range = self.get_range();
        if &range.0 < pos && pos < &range.1 {
            if let Some(builder) = &GENERIC_BUILTINS.get(self.match_name) {
                for (arg, known_flags) in builder.match_params.iter() {
                    items.push(CompletionItem{
                        label: String::from_utf8(arg.to_vec()).unwrap_or_else(|_| UTF8_PARSER_MSG.to_string()),
                        kind: Some(CompletionItemKind::FIELD),
                        ..Default::default()
                    });
                    for flag in known_flags.iter() {
                    items.push(CompletionItem{
                        label: String::from_utf8(flag.to_vec()).unwrap_or_else(|_| UTF8_PARSER_MSG.to_string()),
                        kind: Some(CompletionItemKind::CONSTANT),
                        ..Default::default()
                    });
                    }
                }
            }

        }
    }
    pub fn to_semantic_token(&self, semantic_tokens: &mut Vec<tower_lsp::lsp_types::SemanticToken>, pre_line: &mut u32, pre_start: &mut u32, parser: &Parser) {
        if let Token::Ident(range) = self.name {semantic_tokens.push(get_range_token(&range, 4, pre_line, pre_start, parser));}

        MadParam::to_semantic_token(&self.args, semantic_tokens, pre_line, pre_start, parser);

    }

    pub(crate) fn get_label<'a>(&'a self, pos: &CursorPosition, parser: &'a Parser) -> Option<&[u8]> {
        for p in self.args.iter() {
            if let Some(label) = p.get_label(pos, parser) { return Some(label); }
        }
        None
    }

    pub(crate) fn get_problems(&self, problems: &mut Vec<Problem>) {
        for arg in self.args.iter() {
            if !arg.valid {
                problems.push(Problem::InvalidParam(arg.get_range()));
            }
        }
    }
}

impl HasRange for MadGeneric {
    fn get_range(&self) -> (CursorPosition, CursorPosition) {
        let r1 = self.name.get_range();
        if let Some(last) = self.args.last(){
            let r2 = last.get_range();
            return (r1.0, r2.1);
        }
        return r1;
    }
}

// ---- MadParam -----------------------------------------------------------------------------------
impl MadParam {
    pub fn parse(parser: &mut Parser) -> Option<Self> {
        if let Some(token) = parser.peek_token() {
            let mut param = Self{
                valid: false,
                sign: None,
                attribute: Default::default(),
                value: None,
            };
            if let Token::Operator(_) = token {
                param.sign = Some(token.clone());
                parser.advance();
            }
            if let Some(attribute_token) = parser.peek_token() {
                if let Token::Ident(_) = attribute_token {
                    param.attribute = attribute_token.clone();
                    parser.advance();
                }
            }
            if let Some(Token::Equal(_)) = parser.peek_token() {
                parser.advance();
                
                let last_pos = parser.get_position();
                // todo: missing test for syntax error
                if let Some(expr) = Expression::parse(parser) {
                    param.value = match expr {
                        Expression::MadGeneric(_) => {
                            parser.set_position(last_pos);
                            let token = parser.next_token().unwrap(); // we know that here's a
                                                                      // valid token
                            Some(Box::new(Expression::TokenExp(token.clone())))
                        },
                        Expression::MadEnvironment(_) => {
                            parser.set_position(last_pos);
                            let token = parser.next_token().unwrap(); // we know that here's a
                                                                      // valid token
                            Some(Box::new(Expression::TokenExp(token.clone())))
                        },
                        _ => Some(Box::new(expr))
                    };
                }
            }
            if !param.attribute.is_eof() {
                return Some(param);
            }
        }
        None
    }

    pub fn to_semantic_token(args: &[Self], semantic_tokens: &mut Vec<tower_lsp::lsp_types::SemanticToken>, pre_line: &mut u32, pre_start: &mut u32, parser: &Parser) { 
        for arg in args.iter() {
            if !arg.valid {continue;}
            let range = arg.attribute.get_range();
            semantic_tokens.push(get_range_token(&arg.attribute, 5, pre_line, pre_start, parser));
            if let Some(value) = &arg.value {
                value.to_semantic_token(semantic_tokens, pre_line, pre_start, parser);
            }
        }
    }

    pub fn get_range(&self) -> (CursorPosition, CursorPosition) {
        let start = if let Some(sign) = &self.sign {
            sign.get_range().0
        } else {
            self.attribute.get_range().0
        };
        let end = if let Some(value) = &self.value {
            value.get_range().1
        } else {
            self.attribute.get_range().1
        };
        (start, end)
    }

    pub fn parse_params(parser: &mut Parser, match_params: &Vec<MatchParam>) -> Vec<Self> {
        let mut args = Vec::new();
        while let Some(token) = parser.peek_token() {
            if let Token::SemiColon(_) = token {
                return args;
            }
            if let Token::Komma(_) = token {
                parser.advance();
                //let param = MadParam::parse(parser)?;
                if let Some(mut param) = MadParam::parse(parser) {
                    let bytes = &parser.get_element_bytes(&param.attribute).to_ascii_lowercase().to_vec();
                    for (p, _) in match_params.iter() {
                        if p == bytes {
                            param.valid = true;
                            break;
                        }
                    }
                    args.push(param);
                }
                else {
                    break;
                }
            }
            else {
                // this is actually an error state, but we continue for the moment
                parser.advance();
            }
        }
        args
    }

    fn get_label<'a>(&'a self, pos: &CursorPosition, parser: &'a Parser) -> Option<&[u8]> {
        self.value.as_ref()?.get_label(pos, parser)
    }
}

// ---- MadGenericBuilder --------------------------------------------------------------------------
impl MadGenericBuilder {
    pub fn parse(&self, parser: &mut Parser) -> Option<MadGeneric> {
        if let Some(Token::Ident(name)) = parser.peek_token().cloned() {
            if !parser.lexer.compare_range(&name, self.match_name){
                return None;
            }
            parser.advance();

            let mut mad = MadGeneric {
                match_name: self.match_name,
                name: Token::Ident(name),
                args: Vec::new(),
            };

            mad.args = MadParam::parse_params(parser, &self.match_params);

            return Some(mad);
        }
        None
    }

    //pub fn has_attribute(&self, name: &[u8]) -> bool {
    //    self.match_params.iter().any(|p| p == name)
    //}
}

pub fn insert_generic_builder(map: &mut HashMap<&'static [u8], MadGenericBuilder>,
                          match_name: &'static [u8],
                          match_params: &[&str]) {
    map.insert(
        match_name,
        MadGenericBuilder {
            match_name,
            match_params: match_params.iter()
                .map(|s|
                     (s.as_bytes().to_vec(),
                     vec![])
                    ).collect(),
        }
        );
}

/// Inserts a new generic builder with flags with known values, e.g. SELECT.
pub fn insert_generic_builder_known_flags(map: &mut HashMap<&'static [u8], MadGenericBuilder>,
                          match_name: &'static [u8],
                          match_params: &[&str],
                          known_flags: &[&[&str]]) {
    map.insert(
        match_name,
        MadGenericBuilder {
            match_name,
            match_params: match_params.iter()
                .zip(known_flags).map(|(s, f)| (
                    s.as_bytes().to_vec(),
                    f.iter().map(|flag| flag.as_bytes().to_vec()).collect()
                    )
                    ).collect(),
        }
        );
}



#[cfg(test)]
mod tests {
    use crate::parser::{Parser, Expression};

    #[test]
    pub fn incomplete() {
        let parser = Parser::from_str("call, fi");

        let call = &parser.get_elements()[0];

        if let Expression::MadGeneric(g) = call {
            assert_eq!(parser.get_element_str(&g.name), "call");
        }
        else {
            assert!(false, "this should be recognized as incomplete CALL");
        }
    }

    #[test]
    pub fn incomplete_inside() {
        let parser = Parser::from_str("call, fi\ntwiss, sequence=lhcb1;");

        let call = &parser.get_elements()[0];

        if let Expression::MadGeneric(g) = call {
            assert_eq!(parser.get_element_str(&g.name), "call");
        }
        else {
            assert!(false, "this should be recognized as incomplete CALL");
        }
    }
}

