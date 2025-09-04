use clap::Parser;
use madxls::parser::{self, Problem};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(long)]
    pub input_file: Option<String>,
}

fn main() {
    println!("hello\n");
    let args = Args::parse();

    if let Some(file) = args.input_file {
        let parser = parser::Parser::from_path(file).unwrap();
        println!("{} Elements", parser.get_elements().len());
        println!("- - - - - - - - - - ");
        for e in parser.get_elements() {
            println!("{:?}", e);
        }
        println!("----------------------------------------\n");

        println!("{} Labels", parser.labels.len());
        println!("- - - - - - - - - - ");

        for l in parser.labels.iter() {
            println!("{:?}", l);
        }
        println!("----------------------------------------\n");

        println!("{} Problems", parser.problems.len());
        println!("- - - - - - - - - - ");

        for p in parser.problems.iter() {
            match p {
                Problem::MissingCallee(c, range) => {
                    match parser
                        .labels
                        .iter()
                        .find(|(l, _)| parser.get_element_bytes(range) == **l)
                    {
                        None => println!("{:?}, {}", p, parser.get_element_str(range)),
                        Some(_) => {}
                    }
                }
                _ => {}
            };
        }
        return;
    }
}
