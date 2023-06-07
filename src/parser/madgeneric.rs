use std::collections::HashMap;

use once_cell::sync::Lazy;
use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind};

use crate::{lexer::{Token, CursorPosition, HasRange}, semantic_tokens::{get_range_token}, error::UTF8_PARSER_MSG};

use super::{Expression, Parser};

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
    insert_generic_builder(&mut builtins, b"rbend", &["L", "ANGLE", "TILT",
                           "K0", "K0S",
                           "K1", "K1S",
                           "K2", "K2S",
                           "E1", "E2",
                           "FINT", "FINTX",
                           "HGAP", "H1", "H2",
                           "THICK", "ADD_ANGLE", "KILL_ENT_FRINGE"
    ]);

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
    pub match_params: Vec<Vec<u8>>,
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
                for arg in builder.match_params.iter() {
                    items.push(CompletionItem{
                        label: String::from_utf8(arg.to_vec()).unwrap_or_else(|_| UTF8_PARSER_MSG.to_string()),
                        kind: Some(CompletionItemKind::FIELD),
                        ..Default::default()
                    });
                }
            }

        }
    }
    pub fn to_semantic_token(&self, semantic_tokens: &mut Vec<tower_lsp::lsp_types::SemanticToken>, pre_line: &mut u32, pre_start: &mut u32, parser: &Parser) {
        if let Token::Ident(range) = self.name {semantic_tokens.push(get_range_token(&range, 4, pre_line, pre_start, parser));}

        for arg in self.args.iter() {
            if !arg.valid {continue;}
            let range = arg.attribute.get_range();
            semantic_tokens.push(get_range_token(&arg.attribute, 5, pre_line, pre_start, parser));
            if let Some(value) = &arg.value {
                value.to_semantic_token(semantic_tokens, pre_line, pre_start, parser);
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
                // todo: missing test for syntax error
                if let Some(expr) = Expression::parse(parser) {
                    param.value = Some(Box::new(expr));
                }
            }
            if !param.attribute.is_eof() {
                return Some(param);
            }
        }
        None
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

            while let Some(token) = parser.peek_token() {
                if let Token::SemiColon(_) = token {
                    return Some(mad);
                }
                if let Token::Komma(_) = token {
                    parser.advance();
                    //let param = MadParam::parse(parser)?;
                    if let Some(mut param) = MadParam::parse(parser) {
                        if self.match_params.contains(&parser.get_element_bytes(&param.attribute).to_vec()) {
                            param.valid = true;
                        }
                        mad.args.push(param);
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
            return Some(mad);
        }
        None
    }

    pub fn has_attribute(&self, name: &[u8]) -> bool {
        self.match_params.iter().any(|p| p == name)
    }
}

pub fn insert_generic_builder(map: &mut HashMap<&'static [u8], MadGenericBuilder>,
                          match_name: &'static [u8],
                          match_params: &[&str]) {
    map.insert(
        match_name,
        MadGenericBuilder {
            match_name,
            match_params: match_params.iter().map(|s| s.as_bytes().to_vec()).collect(),
        }
        );
}



