use std::fs::File;
use std::io::BufWriter;
use std::io::Write;

use log::LevelFilter;
use log4rs::Config;
use log4rs::config::Appender;
use log4rs::config::Root;
use log4rs::encode::pattern::PatternEncoder;
use parser::LEGEND_TYPE;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use dashmap::DashMap;
use log4rs::append::file::FileAppender;
use clap::Parser;

pub mod lexer;
pub mod parser;
pub mod document;
pub mod semantic_tokens;
pub mod error;

pub mod debug;


#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(long)]
    pub debug_file: Option<String>,
}


#[tokio::main]
async fn main() {
    //debug::debug_parser();
    //
    let args = Args::parse();

    if let Some(file) = args.debug_file {
        debug::print_ast(file);
    }

    //return;
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
        .build("/home/awegsche/logs/logfile.log").unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Debug)).unwrap();

    log4rs::init_config(config).unwrap();
    run_server().await;
}

#[derive(Debug)]
struct Backend {
    documents: DashMap<Url, document::Document>,
    client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult{
            server_info: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string(), ",".to_string()]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                semantic_tokens_provider: Some(
                                              SemanticTokensServerCapabilities::SemanticTokensRegistrationOptions(
                                                  SemanticTokensRegistrationOptions {
                                                      text_document_registration_options: {
                                                          TextDocumentRegistrationOptions {
                                                              document_selector: Some(vec![DocumentFilter {
                                                                  language: Some("madx".to_string()),
                                                                  scheme: Some("file".to_string()),
                                                                  pattern: None,
                                                              }]),
                                                          }
                                                      },
                                                      semantic_tokens_options: SemanticTokensOptions {
                                                          work_done_progress_options: WorkDoneProgressOptions::default(),
                                                          legend: SemanticTokensLegend {
                                                              token_types: LEGEND_TYPE.into(),
                                                              token_modifiers: vec![],
                                                          },
                                                          range: Some(true),
                                                          full: Some(SemanticTokensFullOptions::Bool(true)),
                                                      },
                                                      static_registration_options: StaticRegistrationOptions::default(),
                                                  },
                                                  ),
                                                  ),
                                                  ..ServerCapabilities::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, p: InitializedParams) {
        log::info!("initialized");
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        log::info!("completion");
        let uri = params.text_document_position.text_document.uri;
        if let Some(document) = self.documents.get(&uri) {
            document.get_completion(params.text_document_position.position)
        }
        else {
            Ok(Some(CompletionResponse::Array(
                        vec![
                        CompletionItem::new_simple(
                            "hello".to_string(), "details".to_string()
                            )
                        ]
                        )))
        }
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        log::info!("did open");
        self.client
            .log_message(MessageType::INFO, "did open!")
            .await;
        let uri = &params.text_document.uri;
        if !self.documents.contains_key(uri) {
            let mut document = document::Document::new(params.text_document.text.as_bytes());
            self.documents.insert(
                uri.clone(),
                document,
            );
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        log::info!("did change");
        if let Some(mut document) = self.documents.get_mut(&params.text_document.uri) {
            document.reload(params.content_changes[0].text.as_bytes());
        }
    }


    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        log::info!("hover");
        if let Some(parser) = self.documents.get(&params.text_document_position_params.text_document.uri) {
            return Ok(Some(Hover {
                contents: HoverContents::Scalar(
                              MarkedString::String("You're hovering!".to_string())
                              ),
                              range: None
            }));

        }
        Ok(None)
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        log::info!("semantic tokens full");
        if let Some(document) = self.documents.get(&params.text_document.uri) {
            return document.get_semantic_tokens();
        }
        Ok(None)
    }



    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

async fn run_server() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend { client, documents: DashMap::new() });
    Server::new(stdin, stdout, socket).serve(service).await;
}

