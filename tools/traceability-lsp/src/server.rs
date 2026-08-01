//! JSON-RPC loop and method dispatch for the LSP server.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use lsp_server::{Connection, Message, Notification, Request, Response};
use lsp_types::{
    request::GotoImplementationParams, CodeLensOptions, CodeLensParams, GotoDefinitionParams,
    HoverParams, InitializeParams, OneOf, PublishDiagnosticsParams, ReferenceParams,
    ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind,
    WorkspaceFoldersServerCapabilities, WorkspaceServerCapabilities,
};

use crate::diagnostics;
use crate::lsp_util::path_to_uri;
use crate::navigation;
use traceability_lsp::TraceIndex;

pub fn run() -> Result<(), String> {
    let (connection, _io_threads) = Connection::stdio();
    let (id, params) = connection
        .initialize_start()
        .map_err(|e| format!("initialize failed: {e}"))?;
    let _params: InitializeParams =
        serde_json::from_value(params).map_err(|e| format!("bad initialize params: {e}"))?;

    let caps = server_capabilities();
    connection
        .initialize_finish(id, serde_json::to_value(caps).unwrap())
        .map_err(|e| format!("initialize_finish failed: {e}"))?;

    let Connection { sender, receiver } = connection;
    let mut server = Server::new(sender);

    loop {
        match receiver.recv() {
            Ok(Message::Request(req)) => server.on_request(req)?,
            Ok(Message::Notification(not)) => {
                if server.on_notification(not)? {
                    break;
                }
            }
            Ok(Message::Response(_)) => {}
            Err(_) => break,
        }
    }

    Ok(())
}

fn server_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(
            TextDocumentSyncKind::FULL,
        )),
        hover_provider: Some(lsp_types::HoverProviderCapability::Simple(true)),
        definition_provider: Some(OneOf::Left(true)),
        references_provider: Some(OneOf::Left(true)),
        implementation_provider: Some(
            lsp_types::ImplementationProviderCapability::Simple(true),
        ),
        code_lens_provider: Some(CodeLensOptions {
            resolve_provider: Some(false),
        }),
        workspace: Some(WorkspaceServerCapabilities {
            workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                supported: Some(true),
                change_notifications: Some(OneOf::Left(true)),
            }),
            file_operations: None,
        }),
        ..Default::default()
    }
}

struct Server {
    sender: crossbeam_channel::Sender<Message>,
    index: TraceIndex,
    overlays: HashMap<PathBuf, String>,
    open_docs: Vec<PathBuf>,
    root: PathBuf,
}

impl Server {
    fn new(sender: crossbeam_channel::Sender<Message>) -> Self {
        let root = traceability_lsp::workspace_root();
        let overlays = HashMap::new();
        let index = TraceIndex::build(&root, &overlays);
        Server {
            sender,
            index,
            overlays,
            open_docs: Vec::new(),
            root,
        }
    }

    fn rebuild(&mut self) {
        self.index = TraceIndex::build(&self.root, &self.overlays);
    }

    fn on_request(&mut self, req: Request) -> Result<(), String> {
        let method = req.method.clone();
        let id = req.id;
        match method.as_str() {
            "shutdown" => self.respond_ok(id, serde_json::Value::Null),
            "textDocument/hover" => {
                let Some(params) = self.parse_or_err::<HoverParams>(&id, req.params) else {
                    return Ok(());
                };
                let result = navigation::hover(&self.index, &params);
                self.respond_ok(id, serde_json::to_value(result).unwrap())
            }
            "textDocument/definition" => {
                let Some(params) = self.parse_or_err::<GotoDefinitionParams>(&id, req.params) else {
                    return Ok(());
                };
                let result = navigation::definition(&self.index, &params);
                self.respond_ok(id, serde_json::to_value(result).unwrap())
            }
            "textDocument/references" => {
                let Some(params) = self.parse_or_err::<ReferenceParams>(&id, req.params) else {
                    return Ok(());
                };
                let result = navigation::references(&self.index, &params);
                self.respond_ok(id, serde_json::to_value(result).unwrap())
            }
            "textDocument/implementation" => {
                let Some(params) = self.parse_or_err::<GotoImplementationParams>(&id, req.params)
                else {
                    return Ok(());
                };
                let result = navigation::implementation(&self.index, &params);
                self.respond_ok(id, serde_json::to_value(result).unwrap())
            }
            "textDocument/codeLens" => {
                let Some(params) = self.parse_or_err::<CodeLensParams>(&id, req.params) else {
                    return Ok(());
                };
                let result = navigation::code_lens(&self.index, &params);
                self.respond_ok(id, serde_json::to_value(result).unwrap())
            }
            _ => {
                let err = lsp_server::ResponseError {
                    code: lsp_server::ErrorCode::MethodNotFound as i32,
                    message: format!("method not found: {method}"),
                    data: None,
                };
                self.send(Message::Response(Response::new_err(id, err.code, err.message)))
            }
        }
    }

    /// Parse request params; on failure, reply with an `InvalidParams` error
    /// response and return `None` so the server keeps running.
    fn parse_or_err<T: serde::de::DeserializeOwned>(
        &self,
        id: &lsp_server::RequestId,
        params: serde_json::Value,
    ) -> Option<T> {
        match serde_json::from_value(params) {
            Ok(t) => Some(t),
            Err(e) => {
                let err = lsp_server::ResponseError {
                    code: lsp_server::ErrorCode::InvalidParams as i32,
                    message: format!("bad params: {e}"),
                    data: None,
                };
                let _ = self.send(Message::Response(Response::new_err(id.clone(), err.code, err.message)));
                None
            }
        }
    }

    /// Returns `Ok(true)` when the loop should exit.
    fn on_notification(&mut self, not: Notification) -> Result<bool, String> {
        match not.method.as_str() {
            "exit" => Ok(true),
            "initialized" => {
                self.publish_all()?;
                Ok(false)
            }
            "textDocument/didOpen" => {
                let params: lsp_types::DidOpenTextDocumentParams = from_params(not.params)?;
                if let Some(path) = crate::lsp_util::uri_to_path(&params.text_document.uri) {
                    self.overlays.insert(path.clone(), params.text_document.text);
                    if !self.open_docs.contains(&path) {
                        self.open_docs.push(path.clone());
                    }
                    self.rebuild();
                    self.publish(&path)?;
                }
                Ok(false)
            }
            "textDocument/didChange" => {
                let params: lsp_types::DidChangeTextDocumentParams = from_params(not.params)?;
                if let Some(path) = crate::lsp_util::uri_to_path(&params.text_document.uri) {
                    if let Some(change) = params.content_changes.last() {
                        self.overlays.insert(path.clone(), change.text.clone());
                    }
                    self.rebuild();
                    self.publish(&path)?;
                }
                Ok(false)
            }
            "textDocument/didSave" => {
                let params: lsp_types::DidSaveTextDocumentParams = from_params(not.params)?;
                if let Some(path) = crate::lsp_util::uri_to_path(&params.text_document.uri) {
                    if let Some(text) = &params.text {
                        self.overlays.insert(path.clone(), text.clone());
                    }
                    self.rebuild();
                    self.publish(&path)?;
                }
                Ok(false)
            }
            "textDocument/didClose" => {
                let params: lsp_types::DidCloseTextDocumentParams = from_params(not.params)?;
                if let Some(path) = crate::lsp_util::uri_to_path(&params.text_document.uri) {
                    self.overlays.remove(&path);
                    self.open_docs.retain(|p| p != &path);
                    self.rebuild();
                }
                Ok(false)
            }
            "workspace/didChangeWatchedFiles" => {
                self.rebuild();
                self.publish_all()?;
                Ok(false)
            }
            _ => Ok(false),
        }
    }

    fn respond_ok(&self, id: lsp_server::RequestId, result: serde_json::Value) -> Result<(), String> {
        self.send(Message::Response(Response::new_ok(id, result)))
    }

    fn send(&self, msg: Message) -> Result<(), String> {
        self.sender
            .send(msg)
            .map_err(|e| format!("failed to send message: {e}"))
    }

    fn publish(&self, path: &Path) -> Result<(), String> {
        let diags = diagnostics::diagnostics_for(&self.index, path);
        let params = PublishDiagnosticsParams {
            uri: path_to_uri(path),
            diagnostics: diags,
            version: None,
        };
        self.send(Message::Notification(Notification {
            method: "textDocument/publishDiagnostics".into(),
            params: serde_json::to_value(params).unwrap(),
        }))
    }

    fn publish_all(&self) -> Result<(), String> {
        for path in &self.open_docs {
            self.publish(path)?;
        }
        Ok(())
    }
}

fn from_params<T: serde::de::DeserializeOwned>(params: serde_json::Value) -> Result<T, String> {
    serde_json::from_value(params).map_err(|e| format!("bad params: {e}"))
}
