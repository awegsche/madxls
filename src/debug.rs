use std::{path::{Path, PathBuf}, time::Instant};

use crate::{lexer::*, parser::{Expression, Parser}};

pub fn print_ast(file: String) {
    //let mut lexer = Lexer::from_bytes(include_bytes!("../test.madx").to_vec());

    let start = Instant::now();
    println!("opening parser");
    //let mut parser = Parser::from_str("token1 \"this is a string\"");
    let parser = Parser::from_path(&file).unwrap();

    let opening_time= Instant::now() - start;

    println!("took {}ms", opening_time.as_millis());
    //println!("{}", parser);

    let mut semantic_tokens = Vec::new();
    let mut pre_line = 0;
    let mut pre_start = 0;

    println!("provide semantic tokens");

    for e in parser.get_elements() {
        e.to_semantic_token(&mut semantic_tokens, &mut pre_line, &mut pre_start, &parser);
    }

    println!("done");

    //println!("{:#?}", semantic_tokens);

}

pub fn debug_parser() {
    //let mut parser = Parser::from_str("! hello\ncall, file;");
    let parser = Parser::from_bytes(include_bytes!("/home/awegsche/fellow/40_magnet_sorting/job.madx").to_vec());

    println!("{}", parser);

}
