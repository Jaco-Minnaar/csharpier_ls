use std::sync::Arc;

use anyhow::anyhow;
use tokio::sync::RwLock;
use tower_lsp::{
    jsonrpc::Result,
    lsp_types::{
        DidChangeTextDocumentParams, DidOpenTextDocumentParams, DocumentFormattingOptions,
        InitializeParams, InitializeResult, InitializedParams, MessageType, OneOf, Position, Range,
        ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind,
        TextDocumentSyncOptions, TextEdit, WillSaveTextDocumentParams, WorkDoneProgressOptions,
    },
    Client, LanguageServer,
};

use crate::{buffer::Buffers, processes::ProcessProvider};

pub struct CSharpierLanguageServer {
    server: Arc<RwLock<Backend>>,
}

impl CSharpierLanguageServer {
    pub fn new(client: Client) -> Self {
        Self {
            server: Arc::new(RwLock::new(Backend::new(client))),
        }
    }
}

#[derive(Debug)]
pub struct Backend {
    pub client: Client,
    process_provider: ProcessProvider,
    buffers: Buffers,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            process_provider: ProcessProvider::new(),
            buffers: Buffers::new(),
        }
    }

    async fn did_open(&mut self, params: DidOpenTextDocumentParams) -> anyhow::Result<()> {
        self.buffers.create_buffer(
            params.text_document.uri.path().to_string(),
            params.text_document.text,
        );

        self.process_provider
            .get_process(params.text_document.uri.path())
            .await?;

        Ok(())
    }

    async fn did_change(&mut self, params: DidChangeTextDocumentParams) -> anyhow::Result<()> {
        self.buffers
            .update_buffer(params.text_document.uri.path(), &params.content_changes)?;

        Ok(())
    }

    async fn will_save_wait_until(
        &mut self,
        params: WillSaveTextDocumentParams,
    ) -> anyhow::Result<Option<Vec<TextEdit>>> {
        let path = params.text_document.uri.path();
        let current_content = self
            .buffers
            .get_buffer(path)
            .ok_or(anyhow!("Could not find buffer {}", path))?;

        let process = self.process_provider.get_process(path).await?;

        let Some(new_content) = process.format_file(current_content, path).await? else {
            log::debug!("No changes to apply");
            return Ok(None);
        };

        log::debug!(
            "Sending new changes to apply. Content length: {}",
            new_content.len()
        );
        Ok(Some(vec![TextEdit::new(
            Range::new(Position::new(0, 0), Position::new(u32::MAX, u32::MAX)),
            new_content,
        )]))
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for CSharpierLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        log::info!("initialize");

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::FULL),
                        will_save_wait_until: Some(true),
                        ..Default::default()
                    },
                )),
                document_formatting_provider: Some(OneOf::Right(DocumentFormattingOptions {
                    work_done_progress_options: WorkDoneProgressOptions {
                        work_done_progress: Some(true),
                    },
                })),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.server
            .read()
            .await
            .client
            .log_message(MessageType::INFO, "server initialized!")
            .await;

        log::info!("initialized");
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let path = params.text_document.uri.path().to_string();
        if let Err(err) = self.server.write().await.did_open(params).await {
            log::error!(
                "Error while performing did_open action on file {}: {}",
                path,
                err
            );
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.server
            .write()
            .await
            .did_change(params)
            .await
            .expect("err while performing did_change action");
    }

    async fn will_save_wait_until(
        &self,
        params: WillSaveTextDocumentParams,
    ) -> Result<Option<Vec<TextEdit>>> {
        Ok(self
            .server
            .write()
            .await
            .will_save_wait_until(params)
            .await
            .expect("err while performing will_save_wait_until action"))
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}
