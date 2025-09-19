use std::sync::Arc;

use clap::Parser;
use dashmap::DashMap;
use log::LevelFilter;
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
    Config,
};
use madxls::document;
use madxls::parser::{Problem, LEGEND_TYPE};
use tower_lsp::jsonrpc::Result;
use tower_lsp::{
    lsp_types::{
        CompletionItem, CompletionOptions, CompletionParams, CompletionResponse,
        DidChangeTextDocumentParams, DidOpenTextDocumentParams, DocumentFilter, DocumentHighlight,
        DocumentHighlightOptions, DocumentHighlightParams, Hover, HoverParams,
        HoverProviderCapability, InitializeParams, InitializeResult, InitializedParams,
        MessageType, OneOf, Position, SemanticTokensFullOptions, SemanticTokensLegend,
        SemanticTokensOptions, SemanticTokensParams, SemanticTokensRegistrationOptions,
        SemanticTokensResult, SemanticTokensServerCapabilities, ServerCapabilities,
        StaticRegistrationOptions, TextDocumentRegistrationOptions, TextDocumentSyncCapability,
        TextDocumentSyncKind, Url, WillSaveTextDocumentParams, WorkDoneProgressOptions,
    },
    Client, LanguageServer, LspService, Server,
};

#[tokio::main]
async fn main() {
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
        .build("logfile.log")
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(
            Root::builder()
                .appender("logfile")
                .build(LevelFilter::Debug),
        )
        .unwrap();

    log4rs::init_config(config).unwrap();
    run_server().await;
}

pub enum Message {
    DiagnosticsCompleted(Vec<Problem>),
}

#[derive(Debug)]
struct Backend {
    documents: Arc<DashMap<Url, document::Document>>,
    client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
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
                document_highlight_provider: Some(OneOf::Right(DocumentHighlightOptions {
                    work_done_progress_options: WorkDoneProgressOptions {
                        work_done_progress: Some(false),
                    },
                })),
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

    async fn initialized(&self, _: InitializedParams) {
        log::info!("initialized");
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        log::info!("completion");
        let uri = params.text_document_position.text_document.uri;
        let mut items = Vec::new();

        get_completions(
            &mut items,
            Some(params.text_document_position.position),
            &uri,
            &self.documents,
        );
        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn document_highlight(
        &self,
        params: DocumentHighlightParams,
    ) -> Result<Option<Vec<DocumentHighlight>>> {
        log::debug!("highlights triggered");
        if let Some(doc) = self
            .documents
            .get(&params.text_document_position_params.text_document.uri)
        {}
        Ok(None)
    }

    async fn will_save(&self, params: WillSaveTextDocumentParams) {
        self.resubmit_diagnostics(&params.text_document.uri).await;
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "did open!")
            .await;
        let uri = &params.text_document.uri;
        if !self.documents.contains_key(uri) {
            let document =
                document::Document::new(Some(uri.clone()), params.text_document.text.as_bytes());

            // check the includes
            /*
            let includes = document.parser.includes.clone();
            let docs = self.documents.clone();
            tokio::spawn(async move {
                for incl in includes.into_iter() {
                    reload_includes(incl, &docs);
                }
            });
            */

            self.documents.insert(uri.clone(), document);
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        log::info!("did change");
        if let Some(mut document) = self.documents.get_mut(&params.text_document.uri) {
            document.reload(params.content_changes[0].text.as_bytes());
            //self.client.publish_diagnostics(params.text_document.uri.clone(), document.get_diagnostics(), None).await;
            // check the includes
            /*
            let includes = document.parser.includes.clone();
            let docs = self.documents.clone();
            tokio::spawn(async move {
                for incl in includes.into_iter() {
                    reload_includes(incl, &docs);
                }
            });
            */
        }
        self.resubmit_diagnostics(&params.text_document.uri).await;
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        log::info!("hover");
        if let Some(doc) = self
            .documents
            .get(&params.text_document_position_params.text_document.uri)
        {
            //self.resubmit_diagnostics(&params.text_document_position_params.text_document.uri)
            //    .await;
            //let labels = doc.get_labels_under_cursor(params.text_document_position_params.position);
            //log::debug!("check hover for: {:?}", labels);
            //let mut items = Vec::new();
            //doc.get_hover(&labels, &mut items, None);

            //log::debug!("docs loaded: {}", self.documents.len());

            //return Ok(Some(Hover {
            //    contents: HoverContents::Array(items),
            //    range: None,
            //}));
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

fn recheck_problems(
    uri: &Url,
    documents: &Arc<DashMap<Url, document::Document>>,
    problems: &mut Vec<Problem>,
) {
    if let Some(doc) = documents.get(uri) {
        // do something
    }
}

fn reload_includes(uri: Url, documents: &Arc<DashMap<Url, document::Document>>) {
    /*
    log::debug!("reloading includes for {}", uri.path());
    if !documents.contains_key(&uri) {
        if let Ok(doc) = document::Document::open(uri.path()) {
            log::debug!("opened doc {}", uri.path());
            for incl in doc.parser.includes.iter().cloned() {
                reload_includes(incl, documents);
            }

            documents.insert(uri.clone(), doc);
        }
    }
    */
}

impl Backend {
    async fn resubmit_diagnostics(&self, uri: &Url) {
        log::debug!("try resubmit");

        if let Some(doc) = self.documents.get(uri) {
            ////let problems = doc.get_diagnostics();

            //self.client
            //    .publish_diagnostics(
            //        uri.clone(),
            //        diagnostics_from_problems(&problems, &doc.parser),
            //        None,
            //    )
            //    .await;
        }
    }
}

fn get_completions(
    items: &mut Vec<CompletionItem>,
    pos: Option<Position>,
    url: &Url,
    documents: &Arc<DashMap<Url, document::Document>>,
) {
    let _ = items;
    if let Some(doc) = documents.get(url) {}
}

async fn run_server() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        documents: Arc::new(DashMap::new()),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
