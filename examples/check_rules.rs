use clap::Parser;
use madxls::parser::{self};
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity};

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

        println!("And now the rules");
        println!("-----------------");

        let collect_labels = madxls::rules::collect_labels::CollectLabels::new(&parser);

        println!("Collected {} labels", collect_labels.get_labels().len());
        for l in collect_labels.get_labels().iter() {
            println!("{}", String::from_utf8_lossy(l));
        }

        let missing_callee = madxls::rules::undefined_exec_call::UndefinedExecCall::new(
            &parser,
            collect_labels.get_labels(),
        );

        println!("Found {} problems", missing_callee.get_problems().len());
        for p in missing_callee.get_problems().iter() {
            let range = parser.lexer.cursor_range_to_text_range(p);
            let severity = Some(DiagnosticSeverity::ERROR);
            let code = None;
            //let source = "madx";
            let message = format!("Undefined exec call: {}", parser.get_element_str(&p));

            let diagnostic = Diagnostic {
                range,
                severity,
                code,
                code_description: None,
                message,
                source: None,
                related_information: None,
                tags: None,
                data: None,
            };

            println!("{:?}", diagnostic);
        }
    }
}
