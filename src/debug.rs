use std::time::Instant;

use crate::parser::Parser;

pub fn print_ast(file: String) {
    let start = Instant::now();
    println!("opening parser");
    let parser = Parser::from_path(&file).unwrap();

    let opening_time = Instant::now() - start;

    println!("took {}ms", opening_time.as_millis());
}
