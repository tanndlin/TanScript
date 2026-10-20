use std::{collections::HashMap, panic, path::PathBuf, sync::Mutex};

use tanscript::{compile, diagnostic::underline_len, error::CompilerError, source_map::SourceMap};
use tower_lsp::{
    Client, LanguageServer, LspService, Server,
    jsonrpc::Result,
    lsp_types::{
        Diagnostic, DiagnosticSeverity, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
        DidOpenTextDocumentParams, InitializeParams, InitializeResult, Position, Range,
        ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind, Url,
    },
};

struct Backend {
    client: Client,
    error_files: Mutex<HashMap<Url, Url>>,
}

impl Backend {
    async fn check(&self, uri: Url, text: String) {
        let found = tokio::task::spawn_blocking({
            let uri = uri.clone();
            move || find_error(&uri, text)
        })
        .await
        .ok()
        .flatten();

        let previous = self.error_files.lock().unwrap().remove(&uri);
        match found {
            Some((error_uri, diagnostic)) => {
                if let Some(previous) = previous.filter(|p| *p != error_uri) {
                    self.clear(previous).await;
                }
                self.error_files
                    .lock()
                    .unwrap()
                    .insert(uri, error_uri.clone());
                self.client
                    .publish_diagnostics(error_uri, vec![diagnostic], None)
                    .await;
            }
            None => {
                if let Some(previous) = previous {
                    self.clear(previous).await;
                }
            }
        }
    }

    async fn clear(&self, uri: Url) {
        self.client.publish_diagnostics(uri, vec![], None).await;
    }
}

fn find_error(uri: &Url, text: String) -> Option<(Url, Diagnostic)> {
    let path = uri
        .to_file_path()
        .unwrap_or_else(|()| PathBuf::from(uri.path()));
    let mut sources = SourceMap::default();
    let file = sources.add(path, text);

    match panic::catch_unwind(panic::AssertUnwindSafe(|| {
        compile(file, &mut sources, None)
    })) {
        Ok(Ok(_)) => None,
        Ok(Err(error)) => Some(to_diagnostic(&error, file, uri, &sources)),
        Err(_) => Some((
            uri.clone(),
            error_diagnostic(
                Range::default(),
                "internal compiler error: the compiler panicked".to_string(),
                "tanscript",
            ),
        )),
    }
}

fn to_diagnostic(
    error: &CompilerError,
    file: u32,
    uri: &Url,
    sources: &SourceMap,
) -> (Url, Diagnostic) {
    let source = sources.get(error.span.file);
    let error_uri = if error.span.file == file {
        uri.clone()
    } else {
        Url::from_file_path(&source.path).unwrap_or_else(|()| uri.clone())
    };

    let line = source
        .text
        .lines()
        .nth(error.span.line as usize)
        .unwrap_or("");
    let len = u32::try_from(underline_len(line, error.span.col as usize)).unwrap_or(1);
    let range = Range::new(
        Position::new(error.span.line, error.span.col),
        Position::new(error.span.line, error.span.col + len),
    );

    (
        error_uri,
        error_diagnostic(range, error.kind.to_string(), error.kind.stage()),
    )
}

fn error_diagnostic(range: Range, message: String, stage: &str) -> Diagnostic {
    Diagnostic {
        range,
        severity: Some(DiagnosticSeverity::ERROR),
        source: Some(format!("tanscript {stage}")),
        message,
        ..Diagnostic::default()
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                ..ServerCapabilities::default()
            },
            ..InitializeResult::default()
        })
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.check(params.text_document.uri, params.text_document.text)
            .await;
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.pop() {
            self.check(params.text_document.uri, change.text).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        let previous = self.error_files.lock().unwrap().remove(&uri);
        if let Some(previous) = previous {
            self.clear(previous).await;
        }
        self.clear(uri).await;
    }
}

#[tokio::main]
async fn main() {
    let (service, socket) = LspService::new(|client| Backend {
        client,
        error_files: Mutex::default(),
    });
    Server::new(tokio::io::stdin(), tokio::io::stdout(), socket)
        .serve(service)
        .await;
}
