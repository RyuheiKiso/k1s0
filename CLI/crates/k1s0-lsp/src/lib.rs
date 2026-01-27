//! k1s0 Language Server Protocol 実装
//!
//! k1s0 lint の結果を LSP 経由でエディタに提供する。
//!
//! # 機能
//!
//! - `textDocument/publishDiagnostics`: lint 結果を診断情報として送信
//! - `textDocument/didOpen`: ファイルを開いたときに lint 実行
//! - `textDocument/didSave`: ファイルを保存したときに lint 実行
//! - `textDocument/didChange`: ファイルを変更したときに lint 実行（デバウンス付き）
//!
//! # 使用方法
//!
//! ```bash
//! # stdio モードで起動
//! k1s0-lsp --stdio
//!
//! # TCP モードで起動
//! k1s0-lsp --tcp --port 9257
//! ```

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use k1s0_generator::lint::{LintConfig, LintResult, Linter, Severity, Violation};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result as LspResult;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

/// LSP サーバー設定
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LspConfig {
    /// lint 設定
    pub lint: LintConfig,
    /// ファイル変更時の lint を有効にするか
    pub lint_on_change: bool,
    /// デバウンス間隔（ミリ秒）
    pub debounce_ms: u64,
}

impl Default for LspConfig {
    fn default() -> Self {
        Self {
            lint: LintConfig::default(),
            debounce_ms: 500,
            lint_on_change: true,
        }
    }
}

/// k1s0 Language Server
pub struct K1s0LanguageServer {
    client: Client,
    config: Arc<RwLock<LspConfig>>,
    /// ワークスペースルート
    workspace_root: Arc<RwLock<Option<PathBuf>>>,
    /// 開いているドキュメント
    documents: Arc<RwLock<HashMap<Url, String>>>,
}

impl K1s0LanguageServer {
    /// 新しい Language Server を作成
    pub fn new(client: Client) -> Self {
        Self {
            client,
            config: Arc::new(RwLock::new(LspConfig::default())),
            workspace_root: Arc::new(RwLock::new(None)),
            documents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// lint を実行して診断情報を発行
    async fn lint_and_publish(&self, uri: &Url) {
        let workspace_root = self.workspace_root.read().await;
        let config = self.config.read().await;

        let path = match workspace_root.as_ref() {
            Some(root) => root.clone(),
            None => {
                if let Ok(path) = uri.to_file_path() {
                    path.parent().map(|p| p.to_path_buf()).unwrap_or_default()
                } else {
                    return;
                }
            }
        };

        // lint 実行
        let linter = Linter::new(config.lint.clone());
        let result = linter.lint(&path);

        // 診断情報に変換
        let diagnostics = self.violations_to_diagnostics(&result, uri);

        // 発行
        self.client
            .publish_diagnostics(uri.clone(), diagnostics, None)
            .await;
    }

    /// Violation を Diagnostic に変換
    fn violations_to_diagnostics(&self, result: &LintResult, uri: &Url) -> Vec<Diagnostic> {
        result
            .violations
            .iter()
            .filter(|v| {
                // URI に関連する違反のみをフィルタ
                if let Some(vpath) = &v.path {
                    if let Ok(file_path) = uri.to_file_path() {
                        return file_path.to_string_lossy().contains(vpath);
                    }
                }
                true // パスがない場合は全て含める
            })
            .map(|v| self.violation_to_diagnostic(v))
            .collect()
    }

    /// Violation を Diagnostic に変換
    fn violation_to_diagnostic(&self, violation: &Violation) -> Diagnostic {
        let severity = match violation.severity {
            Severity::Error => DiagnosticSeverity::ERROR,
            Severity::Warning => DiagnosticSeverity::WARNING,
        };

        let line = violation.line.unwrap_or(1).saturating_sub(1) as u32;

        Diagnostic {
            range: Range {
                start: Position { line, character: 0 },
                end: Position {
                    line,
                    character: 1000,
                },
            },
            severity: Some(severity),
            code: Some(NumberOrString::String(violation.rule.as_str().to_string())),
            source: Some("k1s0".to_string()),
            message: violation.message.clone(),
            related_information: violation.hint.as_ref().map(|hint| {
                vec![DiagnosticRelatedInformation {
                    location: Location {
                        uri: Url::parse("file:///").unwrap(),
                        range: Range::default(),
                    },
                    message: hint.clone(),
                }]
            }),
            ..Default::default()
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for K1s0LanguageServer {
    async fn initialize(&self, params: InitializeParams) -> LspResult<InitializeResult> {
        // ワークスペースルートを設定
        if let Some(root_uri) = params.root_uri {
            if let Ok(root_path) = root_uri.to_file_path() {
                *self.workspace_root.write().await = Some(root_path);
            }
        }

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::INCREMENTAL),
                        save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                            include_text: Some(true),
                        })),
                        ..Default::default()
                    },
                )),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions {
                        identifier: Some("k1s0".to_string()),
                        inter_file_dependencies: true,
                        workspace_diagnostics: true,
                        ..Default::default()
                    },
                )),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "k1s0-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "k1s0 Language Server initialized")
            .await;
    }

    async fn shutdown(&self) -> LspResult<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();

        // ドキュメントを保存
        self.documents
            .write()
            .await
            .insert(uri.clone(), params.text_document.text);

        // lint 実行
        self.lint_and_publish(&uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();

        // ドキュメントを更新
        let mut documents = self.documents.write().await;
        if let Some(doc) = documents.get_mut(&uri) {
            for change in params.content_changes {
                if let Some(range) = change.range {
                    // 増分更新
                    let _ = range; // TODO: 正しく増分更新を実装
                    *doc = change.text;
                } else {
                    // 全置換
                    *doc = change.text;
                }
            }
        }

        // lint 実行（デバウンス付き）
        let config = self.config.read().await;
        if config.lint_on_change {
            // TODO: デバウンス実装
            drop(config);
            self.lint_and_publish(&uri).await;
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri;
        self.lint_and_publish(&uri).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;

        // ドキュメントを削除
        self.documents.write().await.remove(&uri);

        // 診断情報をクリア
        self.client.publish_diagnostics(uri, vec![], None).await;
    }
}

/// LSP サーバーを起動（stdio モード）
pub async fn run_stdio() -> anyhow::Result<()> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(K1s0LanguageServer::new);
    Server::new(stdin, stdout, socket).serve(service).await;

    Ok(())
}

/// LSP サーバーを起動（TCP モード）
pub async fn run_tcp(port: u16) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await?;

    let (stream, _) = listener.accept().await?;
    let (read, write) = tokio::io::split(stream);

    let (service, socket) = LspService::new(K1s0LanguageServer::new);
    Server::new(read, write, socket).serve(service).await;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lsp_config_default() {
        let config = LspConfig::default();

        assert!(config.lint_on_change);
        assert_eq!(config.debounce_ms, 500);
    }
}
