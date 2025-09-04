use std::time::Instant;

use crate::parser::Parser;

pub fn print_ast(file: String) {

    let start = Instant::now();
    println!("opening parser");
    let parser = Parser::from_path(&file).unwrap();

    let opening_time= Instant::now() - start;

    println!("took {}ms", opening_time.as_millis());

    let mut semantic_tokens = Vec::new();
    let mut pre_line = 0;
    let mut pre_start = 0;

    println!("provide semantic tokens");

    for e in parser.get_elements() {
        e.to_semantic_token(&mut semantic_tokens, &mut pre_line, &mut pre_start, &parser);
    }

    println!("done");

    println!("{:#?}", parser.lexer.get_tokens());

    println!("{:#?}", semantic_tokens);

}
