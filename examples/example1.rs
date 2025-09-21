use clap::Parser;
use madxls::parser::{self};

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
    }
}
