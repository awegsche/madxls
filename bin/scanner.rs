use clap::{Parser, Subcommand};
use madxls::highlighter::Highlighter;
use madxls::lexer::print_range;
use madxls::measures::code_blocks::CodeBlocks;
use madxls::measures::loc::Loc;
use madxls::parser;
use madxls::rules::{
    collect_labels::CollectLabels, macro_args::MacroArgs, macro_variables::MacroVars,
    undefined_exec_call::UndefinedExecCall,
};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    pub input_file: Option<String>,

    #[arg(long)]
    pub metrics: bool,

    #[arg(long)]
    pub highlight: bool,
}

fn main() {
    let args = Args::parse();

    let Some(file) = args.input_file else {
        println!("No input file provided");
        return;
    };

    let parser = parser::Parser::from_path(file).unwrap();

    for mf in parser.get_missing_files().iter() {
        println!("MissingFile | {}", print_range(mf));
    }

    let collect_labels = CollectLabels::new(&parser);
    let missing_callee = UndefinedExecCall::new(&parser, collect_labels.get_labels());
    let macro_args = MacroArgs::new(&parser);
    let macro_vars = MacroVars::new(&parser);

    for p in missing_callee.get_problems().iter() {
        let range = parser.lexer.cursor_range_to_text_range(p);

        println!(
            "MissingCallee | ({}, {}) -- ({}, {})",
            range.start.line, range.start.character, range.end.line, range.end.character
        );
    }
    for ma in macro_args.get_problems().iter() {
        println!("{}", ma);
    }
    for mv in macro_vars.get_problems().iter() {
        println!("{}", mv);
    }

    if args.highlight {
        let highlighter = Highlighter::new(&parser);
        for h in highlighter.highlights.iter() {
            println!("{}", h);
        }
    }

    if args.metrics {
        let loc = Loc::new(&parser);

        println!("loc: {}", loc.lines.len());
        println!("commentloc: {}", loc.commentlines.len());

        let code_blocks = CodeBlocks::new(&parser);
        println!("macros: {}", code_blocks.macros);
        println!("statements: {}", code_blocks.statements);
        println!("elements: {}", code_blocks.elements);
    }
}
