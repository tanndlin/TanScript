use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs, mem, panic,
    path::{Path, PathBuf},
    sync::Mutex,
};

use tanscript::{compile, diagnostic::underline_len, error::CompilerError, source_map::SourceMap};
use tower_lsp::{
    Client, LanguageServer, LspService, Server,
    jsonrpc::Result,
    lsp_types::{
        Diagnostic, DiagnosticSeverity, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
        DidOpenTextDocumentParams, DidSaveTextDocumentParams, InitializeParams, InitializeResult,
        Position, Range, ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind,
        TextDocumentSyncOptions, TextDocumentSyncSaveOptions, Url,
    },
};

struct Backend {
    client: Client,
    documents: Mutex<HashMap<Url, String>>,
    reported: Mutex<HashSet<Url>>,
}

impl Backend {
    async fn check_all(&self) {
        let documents = self.documents.lock().unwrap().clone();
        let found = tokio::task::spawn_blocking(move || find_errors(&documents))
            .await
            .unwrap_or_default();

        let previous = mem::replace(
            &mut *self.reported.lock().unwrap(),
            found.keys().cloned().collect(),
        );
        for uri in previous {
            if !found.contains_key(&uri) {
                self.client.publish_diagnostics(uri, vec![], None).await;
            }
        }
        for (uri, diagnostics) in found {
            self.client
                .publish_diagnostics(uri, diagnostics, None)
                .await;
        }
    }
}

struct OpenFile<'a> {
    uri: &'a Url,
    text: &'a String,
}

fn path_of(uri: &Url) -> PathBuf {
    uri.to_file_path()
        .unwrap_or_else(|()| PathBuf::from(uri.path()))
}

fn canonical(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn find_errors(documents: &HashMap<Url, String>) -> HashMap<Url, Vec<Diagnostic>> {
    let open: HashMap<PathBuf, OpenFile> = documents
        .iter()
        .map(|(uri, text)| (canonical(&path_of(uri)), OpenFile { uri, text }))
        .collect();

    let mut entries: BTreeMap<PathBuf, PathBuf> = BTreeMap::new();
    for (canonical_path, file) in &open {
        let path = path_of(file.uri);
        entries.insert(canonical_path.clone(), path.clone());
        let Some(Ok(siblings)) = path.parent().map(fs::read_dir) else {
            continue;
        };
        for sibling in siblings.flatten() {
            let sibling = sibling.path();
            if sibling.extension().is_some_and(|e| e == "tan") {
                entries.entry(canonical(&sibling)).or_insert(sibling);
            }
        }
    }

    let mut found: HashMap<Url, Vec<Diagnostic>> = HashMap::new();
    for (canonical_path, path) in entries {
        let Some((uri, diagnostic)) = check_entry(&canonical_path, path, &open) else {
            continue;
        };
        let diagnostics = found.entry(uri).or_default();
        if !diagnostics.contains(&diagnostic) {
            diagnostics.push(diagnostic);
        }
    }
    found
}

fn check_entry(
    canonical_path: &Path,
    path: PathBuf,
    open: &HashMap<PathBuf, OpenFile>,
) -> Option<(Url, Diagnostic)> {
    let entry_uri = match open.get(canonical_path) {
        Some(file) => file.uri.clone(),
        None => Url::from_file_path(&path).ok()?,
    };
    let text = match open.get(canonical_path) {
        Some(file) => file.text.clone(),
        None => fs::read_to_string(&path).ok()?,
    };

    let mut sources = SourceMap::default();
    let file = sources.add(path, text);

    match panic::catch_unwind(panic::AssertUnwindSafe(|| {
        compile(file, &mut sources, None)
    })) {
        Ok(Ok(_)) => None,
        Ok(Err(error)) => to_diagnostic(&error, file, &entry_uri, &sources, open),
        Err(_) => Some((
            entry_uri,
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
    entry_uri: &Url,
    sources: &SourceMap,
    open: &HashMap<PathBuf, OpenFile>,
) -> Option<(Url, Diagnostic)> {
    let source = sources.get(error.span.file);
    let error_uri = if error.span.file == file {
        entry_uri.clone()
    } else if open.contains_key(&canonical(&source.path)) {
        return None;
    } else {
        Url::from_file_path(&source.path).unwrap_or_else(|()| entry_uri.clone())
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

    Some((
        error_uri,
        error_diagnostic(range, error.kind.to_string(), error.kind.stage()),
    ))
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
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::FULL),
                        save: Some(TextDocumentSyncSaveOptions::Supported(true)),
                        ..TextDocumentSyncOptions::default()
                    },
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
        self.documents
            .lock()
            .unwrap()
            .insert(params.text_document.uri, params.text_document.text);
        self.check_all().await;
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.pop() {
            self.documents
                .lock()
                .unwrap()
                .insert(params.text_document.uri, change.text);
            self.check_all().await;
        }
    }

    async fn did_save(&self, _: DidSaveTextDocumentParams) {
        self.check_all().await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.documents
            .lock()
            .unwrap()
            .remove(&params.text_document.uri);
        self.check_all().await;
    }
}

#[tokio::main]
async fn main() {
    let (service, socket) = LspService::new(|client| Backend {
        client,
        documents: Mutex::default(),
        reported: Mutex::default(),
    });
    Server::new(tokio::io::stdin(), tokio::io::stdout(), socket)
        .serve(service)
        .await;
}
