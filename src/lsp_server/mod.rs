use std::sync::Arc;
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result as JsonRpcResult;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

mod document_store;
pub mod handlers;

use document_store::DocumentStore;

pub struct ProtobufLanguageServer {
    client: Client,
    documents: Arc<RwLock<DocumentStore>>,
}

impl ProtobufLanguageServer {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(RwLock::new(DocumentStore::new())),
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for ProtobufLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> JsonRpcResult<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions {
                        identifier: Some("protobuf-edition-lsp".to_string()),
                        inter_file_dependencies: false,
                        workspace_diagnostics: false,
                        work_done_progress_options: WorkDoneProgressOptions::default(),
                    },
                )),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![
                        ".".to_string(),
                        " ".to_string(),
                        "=".to_string(),
                    ]),
                    work_done_progress_options: WorkDoneProgressOptions::default(),
                    all_commit_characters: None,
                    resolve_provider: None,
                    completion_item: None,
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Protobuf LSP server initialized")
            .await;
    }

    async fn shutdown(&self) -> JsonRpcResult<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let mut store = self.documents.write().await;
        store.open_document(
            params.text_document.uri.clone(),
            params.text_document.text.clone(),
            params.text_document.version,
        );

        // Trigger diagnostics
        let diagnostics = handlers::compute_diagnostics(&params.text_document.text);
        self.client
            .publish_diagnostics(params.text_document.uri, diagnostics, None)
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let mut store = self.documents.write().await;

        // We use full text sync, so there should be exactly one change
        if let Some(change) = params.content_changes.into_iter().next() {
            store.update_document(
                params.text_document.uri.clone(),
                change.text.clone(),
                params.text_document.version,
            );

            // Trigger diagnostics
            let diagnostics = handlers::compute_diagnostics(&change.text);
            self.client
                .publish_diagnostics(params.text_document.uri, diagnostics, None)
                .await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let mut store = self.documents.write().await;
        store.close_document(&params.text_document.uri);
    }

    async fn completion(
        &self,
        params: CompletionParams,
    ) -> JsonRpcResult<Option<CompletionResponse>> {
        let store = self.documents.read().await;
        let document = store.get_document(&params.text_document_position.text_document.uri);

        if let Some(doc) = document {
            let completions =
                handlers::compute_completions(&doc.content, params.text_document_position.position);
            Ok(Some(CompletionResponse::Array(completions)))
        } else {
            Ok(None)
        }
    }

    async fn hover(&self, params: HoverParams) -> JsonRpcResult<Option<Hover>> {
        let store = self.documents.read().await;
        let document = store.get_document(&params.text_document_position_params.text_document.uri);

        if let Some(doc) = document {
            Ok(handlers::compute_hover(
                &doc.content,
                params.text_document_position_params.position,
            ))
        } else {
            Ok(None)
        }
    }

    async fn diagnostic(
        &self,
        params: DocumentDiagnosticParams,
    ) -> JsonRpcResult<DocumentDiagnosticReportResult> {
        let store = self.documents.read().await;
        let document = store.get_document(&params.text_document.uri);

        if let Some(doc) = document {
            let diagnostics = handlers::compute_diagnostics(&doc.content);
            Ok(DocumentDiagnosticReportResult::Report(
                DocumentDiagnosticReport::Full(RelatedFullDocumentDiagnosticReport {
                    related_documents: None,
                    full_document_diagnostic_report: FullDocumentDiagnosticReport {
                        result_id: None,
                        items: diagnostics,
                    },
                }),
            ))
        } else {
            Ok(DocumentDiagnosticReportResult::Report(
                DocumentDiagnosticReport::Full(RelatedFullDocumentDiagnosticReport {
                    related_documents: None,
                    full_document_diagnostic_report: FullDocumentDiagnosticReport {
                        result_id: None,
                        items: vec![],
                    },
                }),
            ))
        }
    }
}
