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
//! - `textDocument/completion`: manifest.json の入力補完
//! - `textDocument/hover`: manifest.json キーのホバー情報
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

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use k1s0_generator::lint::{LayerDependencyRules, LintConfig, Linter, Severity, Violation};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, RwLock};
use tower_lsp::jsonrpc::Result as LspResult;
use tower_lsp::lsp_types::{
    CodeActionKind, CodeActionOptions, CodeActionOrCommand, CodeActionParams,
    CodeActionProviderCapability, CompletionOptions, CompletionParams, CompletionResponse,
    Diagnostic, DiagnosticOptions, DiagnosticRelatedInformation, DiagnosticServerCapabilities,
    DiagnosticSeverity, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, DidSaveTextDocumentParams, DocumentSymbolParams,
    DocumentSymbolResponse, GotoDefinitionParams, GotoDefinitionResponse, Hover, HoverParams,
    HoverProviderCapability, InitializeParams, InitializeResult, InitializedParams, Location,
    MessageType, NumberOrString, Position, Range, ReferenceParams, SaveOptions,
    ServerCapabilities, ServerInfo, SymbolInformation, TextDocumentSyncCapability,
    TextDocumentSyncKind, TextDocumentSyncOptions, TextDocumentSyncSaveOptions, Url,
    WorkspaceSymbolParams,
};
use tower_lsp::{Client, LanguageServer, LspService, Server};

pub mod code_action;
pub mod completion;
pub mod definition;
pub mod hover;
pub mod references;
pub mod schema;
pub mod symbols;

use schema::ManifestSchema;

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
    /// デバウンス用: 最後の変更時刻
    pending_lints: Arc<RwLock<HashMap<Url, Instant>>>,
    /// デバウンス用: lint トリガー送信チャネル
    lint_trigger: mpsc::Sender<(Url, u64)>,
    /// manifest.json スキーマ
    schema: Arc<ManifestSchema>,
    /// 診断を publish 済みの URI セット（stale diagnostic クリア用）
    published_diagnostics_uris: Arc<RwLock<HashSet<Url>>>,
}

/// LSP Position をバイトオフセットに変換
///
/// LSP では Position の character は UTF-16 code unit として解釈される。
/// この関数は行番号と UTF-16 オフセットからバイトオフセットを計算する。
fn position_to_byte_offset(text: &str, position: Position) -> usize {
    let mut byte_offset = 0;

    for (current_line, line) in text.lines().enumerate() {
        if current_line as u32 == position.line {
            // この行内で UTF-16 オフセットからバイトオフセットを計算
            let mut utf16_offset = 0u32;
            for (char_byte_offset, ch) in line.char_indices() {
                if utf16_offset >= position.character {
                    return byte_offset + char_byte_offset;
                }
                utf16_offset += ch.len_utf16() as u32;
            }
            // 行末まで到達
            return byte_offset + line.len();
        }
        // 次の行へ（改行文字を含む）
        byte_offset += line.len() + 1; // +1 for newline
    }

    // 位置がテキストの末尾を超えている場合
    byte_offset.min(text.len())
}

/// 増分変更をテキストに適用
fn apply_incremental_change(text: &str, range: Range, new_text: &str) -> String {
    let start_offset = position_to_byte_offset(text, range.start);
    let end_offset = position_to_byte_offset(text, range.end);

    // 安全なスライス境界を確保
    let start = start_offset.min(text.len());
    let end = end_offset.min(text.len()).max(start);

    let mut result = String::with_capacity(start + new_text.len() + (text.len() - end));
    result.push_str(&text[..start]);
    result.push_str(new_text);
    result.push_str(&text[end..]);
    result
}

impl K1s0LanguageServer {
    /// 新しい Language Server を作成
    pub fn new(client: Client) -> Self {
        // デバウンス用チャネルを作成（ダミー）
        let (tx, _rx) = mpsc::channel(100);
        Self {
            client,
            config: Arc::new(RwLock::new(LspConfig::default())),
            workspace_root: Arc::new(RwLock::new(None)),
            documents: Arc::new(RwLock::new(HashMap::new())),
            pending_lints: Arc::new(RwLock::new(HashMap::new())),
            lint_trigger: tx,
            schema: Arc::new(ManifestSchema::new()),
            published_diagnostics_uris: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// デバウンスワーカー付きで Language Server を作成
    pub fn new_with_debounce(client: Client) -> Self {
        let (tx, rx) = mpsc::channel::<(Url, u64)>(100);

        let server = Self {
            client,
            config: Arc::new(RwLock::new(LspConfig::default())),
            workspace_root: Arc::new(RwLock::new(None)),
            documents: Arc::new(RwLock::new(HashMap::new())),
            pending_lints: Arc::new(RwLock::new(HashMap::new())),
            lint_trigger: tx,
            schema: Arc::new(ManifestSchema::new()),
            published_diagnostics_uris: Arc::new(RwLock::new(HashSet::new())),
        };

        // デバウンスワーカーを起動
        server.spawn_debounce_worker(rx);

        server
    }

    /// デバウンスワーカーを起動
    fn spawn_debounce_worker(&self, mut rx: mpsc::Receiver<(Url, u64)>) {
        let pending_lints = self.pending_lints.clone();
        let client = self.client.clone();
        let workspace_root = self.workspace_root.clone();
        let config = self.config.clone();
        let published_uris = self.published_diagnostics_uris.clone();

        tokio::spawn(async move {
            // URI ごとのタスクハンドルを管理
            let mut tasks: HashMap<Url, tokio::task::JoinHandle<()>> = HashMap::new();

            while let Some((uri, debounce_ms)) = rx.recv().await {
                // 既存のタスクをキャンセル
                if let Some(handle) = tasks.remove(&uri) {
                    handle.abort();
                }

                let uri_clone = uri.clone();
                let pending_lints = pending_lints.clone();
                let client = client.clone();
                let workspace_root = workspace_root.clone();
                let config = config.clone();
                let published_uris = published_uris.clone();

                // 新しいデバウンスタスクを起動
                let handle = tokio::spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_millis(debounce_ms)).await;

                    // デバウンス期間後に lint を実行
                    let workspace_root = workspace_root.read().await;
                    let config = config.read().await;

                    let path = match workspace_root.as_ref() {
                        Some(root) => root.clone(),
                        None => {
                            if let Ok(path) = uri_clone.to_file_path() {
                                path.parent().map(|p| p.to_path_buf()).unwrap_or_default()
                            } else {
                                return;
                            }
                        }
                    };

                    let linter = Linter::new(config.lint.clone());
                    let result = linter.lint(&path);

                    // K040-K047: レイヤー依存チェック
                    let mut all_violations = result.violations;
                    let manifest_path = path.join(".k1s0").join("manifest.json");
                    if manifest_path.exists() {
                        let mut layer_rules =
                            k1s0_generator::lint::LayerDependencyRules::new(&path);
                        let layer_violations = layer_rules.check(&manifest_path);
                        all_violations.extend(layer_violations);
                    }

                    let service_root = &result.path;

                    // violation を URI ごとにグループ化
                    let mut diagnostics_by_uri: HashMap<Url, Vec<Diagnostic>> = HashMap::new();

                    for v in &all_violations {
                        let severity = match v.severity {
                            Severity::Error => DiagnosticSeverity::ERROR,
                            Severity::Warning => DiagnosticSeverity::WARNING,
                        };
                        let line = v.line.unwrap_or(1).saturating_sub(1) as u32;
                        let diag = Diagnostic {
                            range: Range {
                                start: Position { line, character: 0 },
                                end: Position {
                                    line,
                                    character: 1000,
                                },
                            },
                            severity: Some(severity),
                            code: Some(NumberOrString::String(v.rule.as_str().to_string())),
                            source: Some("k1s0".to_string()),
                            message: v.message.clone(),
                            related_information: v.hint.as_ref().map(|hint| {
                                vec![DiagnosticRelatedInformation {
                                    location: Location {
                                        uri: Url::parse("file:///").unwrap(),
                                        range: Range::default(),
                                    },
                                    message: hint.clone(),
                                }]
                            }),
                            ..Default::default()
                        };

                        let target_uri = if let Some(vpath) = &v.path {
                            let abs_path = service_root.join(vpath);
                            Url::from_file_path(&abs_path)
                                .unwrap_or_else(|()| uri_clone.clone())
                        } else {
                            uri_clone.clone()
                        };
                        diagnostics_by_uri.entry(target_uri).or_default().push(diag);
                    }

                    // stale URI をクリア
                    let prev_uris = published_uris.read().await.clone();
                    for stale_uri in &prev_uris {
                        if !diagnostics_by_uri.contains_key(stale_uri) {
                            client
                                .publish_diagnostics(stale_uri.clone(), vec![], None)
                                .await;
                        }
                    }

                    // publish
                    let mut new_uris = HashSet::new();
                    for (target_uri, diags) in &diagnostics_by_uri {
                        client
                            .publish_diagnostics(target_uri.clone(), diags.clone(), None)
                            .await;
                        new_uris.insert(target_uri.clone());
                    }

                    *published_uris.write().await = new_uris;
                    pending_lints.write().await.remove(&uri_clone);
                });

                tasks.insert(uri, handle);
            }
        });
    }

    /// lint をスケジュール（デバウンス付き）
    async fn schedule_lint(&self, uri: &Url) {
        let config = self.config.read().await;
        let debounce_ms = config.debounce_ms;
        drop(config);

        // 最後のリクエスト時刻を更新
        self.pending_lints
            .write()
            .await
            .insert(uri.clone(), Instant::now());

        // トリガーを送信
        let _ = self.lint_trigger.send((uri.clone(), debounce_ms)).await;
    }

    /// lint を実行して診断情報を発行
    ///
    /// violation の `path` フィールドを使い、該当するソースファイルの URI に直接
    /// diagnostic を publish する。`path` がない violation は呼び出し元 URI に publish する。
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

        // K040-K047: レイヤー依存チェック
        let mut all_violations = result.violations;
        let manifest_path = path.join(".k1s0").join("manifest.json");
        if manifest_path.exists() {
            let mut layer_rules = LayerDependencyRules::new(&path);
            let layer_violations = layer_rules.check(&manifest_path);
            all_violations.extend(layer_violations);
        }

        let service_root = &result.path;

        // violation を URI ごとにグループ化
        let mut diagnostics_by_uri: HashMap<Url, Vec<Diagnostic>> = HashMap::new();

        for v in &all_violations {
            let target_uri = if let Some(vpath) = &v.path {
                let abs_path = service_root.join(vpath);
                Url::from_file_path(&abs_path).unwrap_or_else(|()| uri.clone())
            } else {
                uri.clone()
            };
            diagnostics_by_uri
                .entry(target_uri)
                .or_default()
                .push(self.violation_to_diagnostic(v));
        }

        // 前回 publish した URI のうち今回 violation がないものをクリア
        let prev_uris = self.published_diagnostics_uris.read().await.clone();
        for stale_uri in &prev_uris {
            if !diagnostics_by_uri.contains_key(stale_uri) {
                self.client
                    .publish_diagnostics(stale_uri.clone(), vec![], None)
                    .await;
            }
        }

        // 今回の diagnostic を publish
        let mut new_uris = HashSet::new();
        for (target_uri, diags) in &diagnostics_by_uri {
            self.client
                .publish_diagnostics(target_uri.clone(), diags.clone(), None)
                .await;
            new_uris.insert(target_uri.clone());
        }

        // publish 済み URI セットを更新
        *self.published_diagnostics_uris.write().await = new_uris;
    }

    /// manifest.json ファイルかどうかを判定
    fn is_manifest_file(&self, uri: &Url) -> bool {
        if let Ok(path) = uri.to_file_path() {
            if let Some(file_name) = path.file_name() {
                return file_name == "manifest.json";
            }
        }
        false
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
                // 補完機能
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![
                        "\"".to_string(),
                        ":".to_string(),
                        "{".to_string(),
                        ",".to_string(),
                    ]),
                    resolve_provider: Some(false),
                    ..Default::default()
                }),
                // ホバー機能
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                // 定義へジャンプ
                definition_provider: Some(tower_lsp::lsp_types::OneOf::Left(true)),
                // 参照検索
                references_provider: Some(tower_lsp::lsp_types::OneOf::Left(true)),
                // ドキュメントシンボル
                document_symbol_provider: Some(tower_lsp::lsp_types::OneOf::Left(true)),
                // ワークスペースシンボル
                workspace_symbol_provider: Some(tower_lsp::lsp_types::OneOf::Left(true)),
                // Code Actions
                code_action_provider: Some(CodeActionProviderCapability::Options(
                    CodeActionOptions {
                        code_action_kinds: Some(vec![CodeActionKind::QUICKFIX]),
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
                    // 増分更新: UTF-16 オフセットを考慮して変更を適用
                    *doc = apply_incremental_change(doc, range, &change.text);
                } else {
                    // 全置換
                    *doc = change.text;
                }
            }
        }
        drop(documents);

        // lint 実行（デバウンス付き）
        let config = self.config.read().await;
        if config.lint_on_change {
            drop(config);
            self.schedule_lint(&uri).await;
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

        // 全 publish 済み URI の診断情報をクリア
        let prev_uris = self.published_diagnostics_uris.write().await.drain().collect::<Vec<_>>();
        for stale_uri in prev_uris {
            self.client
                .publish_diagnostics(stale_uri, vec![], None)
                .await;
        }
        self.client.publish_diagnostics(uri, vec![], None).await;
    }

    async fn completion(&self, params: CompletionParams) -> LspResult<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        // manifest.json のみ補完を提供
        if !self.is_manifest_file(uri) {
            return Ok(None);
        }

        // ドキュメントの内容を取得
        let documents = self.documents.read().await;
        let document = match documents.get(uri) {
            Some(doc) => doc.clone(),
            None => return Ok(None),
        };
        drop(documents);

        // ワークスペースルートを取得
        let workspace_root = self.workspace_root.read().await;

        // 補完候補を取得
        let items =
            completion::get_completions(&document, position, &self.schema, workspace_root.as_ref());

        if items.is_empty() {
            Ok(None)
        } else {
            Ok(Some(CompletionResponse::Array(items)))
        }
    }

    async fn hover(&self, params: HoverParams) -> LspResult<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        // manifest.json のみホバー情報を提供
        if !self.is_manifest_file(uri) {
            return Ok(None);
        }

        // ドキュメントの内容を取得
        let documents = self.documents.read().await;
        let document = match documents.get(uri) {
            Some(doc) => doc.clone(),
            None => return Ok(None),
        };
        drop(documents);

        // ホバー情報を取得
        Ok(hover::get_hover_info(&document, position, &self.schema))
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> LspResult<Option<GotoDefinitionResponse>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        // manifest.json のみ定義ジャンプを提供
        if !self.is_manifest_file(uri) {
            return Ok(None);
        }

        // ドキュメントの内容を取得
        let documents = self.documents.read().await;
        let document = match documents.get(uri) {
            Some(doc) => doc.clone(),
            None => return Ok(None),
        };
        drop(documents);

        // ワークスペースルートを取得
        let workspace_root = self.workspace_root.read().await;
        let root_ref = workspace_root.as_ref();

        // 定義を検索
        Ok(definition::find_definition(&document, position, root_ref))
    }

    async fn references(&self, params: ReferenceParams) -> LspResult<Option<Vec<Location>>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        // manifest.json のみ参照検索を提供
        if !self.is_manifest_file(uri) {
            return Ok(None);
        }

        // ドキュメントの内容を取得
        let documents = self.documents.read().await;
        let document = match documents.get(uri) {
            Some(doc) => doc.clone(),
            None => return Ok(None),
        };
        drop(documents);

        // ワークスペースルートを取得
        let workspace_root = self.workspace_root.read().await;
        let root_ref = workspace_root.as_ref();

        // 参照を検索
        let refs = references::find_references(
            uri,
            &document,
            position,
            root_ref,
            params.context.include_declaration,
        );

        if refs.is_empty() {
            Ok(None)
        } else {
            Ok(Some(refs))
        }
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> LspResult<Option<DocumentSymbolResponse>> {
        let uri = &params.text_document.uri;

        // manifest.json のみシンボルを提供
        if !self.is_manifest_file(uri) {
            return Ok(None);
        }

        // ドキュメントの内容を取得
        let documents = self.documents.read().await;
        let document = match documents.get(uri) {
            Some(doc) => doc.clone(),
            None => return Ok(None),
        };
        drop(documents);

        // ドキュメントシンボルを抽出
        let symbols = symbols::extract_document_symbols(&document);

        if symbols.is_empty() {
            Ok(None)
        } else {
            Ok(Some(DocumentSymbolResponse::Nested(symbols)))
        }
    }

    async fn symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> LspResult<Option<Vec<SymbolInformation>>> {
        let query = &params.query;

        // 開いているドキュメントからシンボルを検索
        let documents = self.documents.read().await;
        let manifest_files: Vec<(Url, String)> = documents
            .iter()
            .filter(|(uri, _)| self.is_manifest_file(uri))
            .map(|(uri, content)| (uri.clone(), content.clone()))
            .collect();
        drop(documents);

        // ワークスペースシンボルを検索
        let symbols = symbols::search_workspace_symbols(query, &manifest_files);

        if symbols.is_empty() {
            Ok(None)
        } else {
            Ok(Some(symbols))
        }
    }

    async fn code_action(
        &self,
        params: CodeActionParams,
    ) -> LspResult<Option<Vec<CodeActionOrCommand>>> {
        let uri = &params.text_document.uri;
        let workspace_root = self.workspace_root.read().await;
        let actions = code_action::get_code_actions(
            uri,
            &params.context.diagnostics,
            workspace_root.as_ref(),
        );
        if actions.is_empty() {
            Ok(None)
        } else {
            Ok(Some(actions))
        }
    }
}

/// LSP サーバーを起動（stdio モード）
pub async fn run_stdio() -> anyhow::Result<()> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(K1s0LanguageServer::new_with_debounce);
    Server::new(stdin, stdout, socket).serve(service).await;

    Ok(())
}

/// LSP サーバーを起動（TCP モード）
pub async fn run_tcp(port: u16) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await?;

    let (stream, _) = listener.accept().await?;
    let (read, write) = tokio::io::split(stream);

    let (service, socket) = LspService::new(K1s0LanguageServer::new_with_debounce);
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

    #[test]
    fn test_position_to_byte_offset_ascii() {
        let text = "hello\nworld\n";

        // 1行目、先頭
        assert_eq!(
            position_to_byte_offset(text, Position { line: 0, character: 0 }),
            0
        );

        // 1行目、3文字目
        assert_eq!(
            position_to_byte_offset(text, Position { line: 0, character: 3 }),
            3
        );

        // 2行目、先頭
        assert_eq!(
            position_to_byte_offset(text, Position { line: 1, character: 0 }),
            6
        );

        // 2行目、2文字目
        assert_eq!(
            position_to_byte_offset(text, Position { line: 1, character: 2 }),
            8
        );
    }

    #[test]
    fn test_position_to_byte_offset_utf16() {
        // 絵文字は UTF-16 で 2 code units、UTF-8 で 4 bytes
        let text = "a😀b\nc";

        // 'a' の位置 (0)
        assert_eq!(
            position_to_byte_offset(text, Position { line: 0, character: 0 }),
            0
        );

        // 😀 の位置 (1) - バイトオフセット 1
        assert_eq!(
            position_to_byte_offset(text, Position { line: 0, character: 1 }),
            1
        );

        // 'b' の位置 - UTF-16 では character 3（絵文字が2 code units）、バイトオフセット 5
        assert_eq!(
            position_to_byte_offset(text, Position { line: 0, character: 3 }),
            5
        );

        // 2行目の 'c'
        assert_eq!(
            position_to_byte_offset(text, Position { line: 1, character: 0 }),
            7
        );
    }

    #[test]
    fn test_apply_incremental_change_insert() {
        let text = "hello world";
        let range = Range {
            start: Position { line: 0, character: 5 },
            end: Position { line: 0, character: 5 },
        };
        let new_text = " beautiful";

        let result = apply_incremental_change(text, range, new_text);
        assert_eq!(result, "hello beautiful world");
    }

    #[test]
    fn test_apply_incremental_change_delete() {
        let text = "hello beautiful world";
        let range = Range {
            start: Position { line: 0, character: 5 },
            end: Position { line: 0, character: 15 },
        };
        let new_text = "";

        let result = apply_incremental_change(text, range, new_text);
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_apply_incremental_change_replace() {
        let text = "hello world";
        let range = Range {
            start: Position { line: 0, character: 6 },
            end: Position { line: 0, character: 11 },
        };
        let new_text = "rust";

        let result = apply_incremental_change(text, range, new_text);
        assert_eq!(result, "hello rust");
    }

    #[test]
    fn test_apply_incremental_change_multiline() {
        let text = "line1\nline2\nline3";
        let range = Range {
            start: Position { line: 1, character: 0 },
            end: Position { line: 1, character: 5 },
        };
        let new_text = "replaced";

        let result = apply_incremental_change(text, range, new_text);
        assert_eq!(result, "line1\nreplaced\nline3");
    }

    #[test]
    fn test_apply_incremental_change_with_emoji() {
        let text = "hello 😀 world";
        // 😀 は UTF-16 で 2 code units、UTF-8 で 4 bytes
        // "hello " = 6 chars, 😀 starts at character 6, ends at character 8 (UTF-16)
        let range = Range {
            start: Position { line: 0, character: 6 },
            end: Position { line: 0, character: 8 },
        };
        let new_text = "🎉";

        let result = apply_incremental_change(text, range, new_text);
        assert_eq!(result, "hello 🎉 world");
    }

    #[test]
    fn test_position_to_byte_offset_empty_text() {
        let text = "";
        let result = position_to_byte_offset(text, Position { line: 0, character: 0 });
        assert_eq!(result, 0);
    }

    #[test]
    fn test_position_to_byte_offset_beyond_text() {
        let text = "abc";
        // 行がテキストの末尾を超えている場合
        let result = position_to_byte_offset(text, Position { line: 10, character: 0 });
        assert_eq!(result, 3); // テキストの長さに制限される
    }

    #[test]
    fn test_position_to_byte_offset_beyond_line() {
        let text = "abc\ndef";
        // 文字位置が行の末尾を超えている場合
        let result = position_to_byte_offset(text, Position { line: 0, character: 100 });
        assert_eq!(result, 3); // 行末まで
    }

    #[test]
    fn test_apply_incremental_change_at_start() {
        let text = "hello";
        let range = Range {
            start: Position { line: 0, character: 0 },
            end: Position { line: 0, character: 0 },
        };
        let new_text = "prefix ";

        let result = apply_incremental_change(text, range, new_text);
        assert_eq!(result, "prefix hello");
    }

    #[test]
    fn test_apply_incremental_change_at_end() {
        let text = "hello";
        let range = Range {
            start: Position { line: 0, character: 5 },
            end: Position { line: 0, character: 5 },
        };
        let new_text = " suffix";

        let result = apply_incremental_change(text, range, new_text);
        assert_eq!(result, "hello suffix");
    }

    #[test]
    fn test_apply_incremental_change_cross_line() {
        let text = "line1\nline2\nline3";
        let range = Range {
            start: Position { line: 0, character: 3 },
            end: Position { line: 2, character: 2 },
        };
        let new_text = "REPLACED";

        let result = apply_incremental_change(text, range, new_text);
        assert_eq!(result, "linREPLACEDne3");
    }

    #[test]
    fn test_apply_incremental_change_empty_result() {
        let text = "hello";
        let range = Range {
            start: Position { line: 0, character: 0 },
            end: Position { line: 0, character: 5 },
        };
        let new_text = "";

        let result = apply_incremental_change(text, range, new_text);
        assert_eq!(result, "");
    }

    #[test]
    fn test_lsp_config_serialization() {
        let config = LspConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("debounce_ms"));
        assert!(json.contains("lint_on_change"));

        let parsed: LspConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.debounce_ms, config.debounce_ms);
        assert_eq!(parsed.lint_on_change, config.lint_on_change);
    }

    #[test]
    fn test_lsp_config_deserialization_with_custom_values() {
        let json = r#"{"debounce_ms": 1000, "lint_on_change": false}"#;
        let config: LspConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.debounce_ms, 1000);
        assert!(!config.lint_on_change);
    }

    #[test]
    fn test_position_to_byte_offset_japanese() {
        // 日本語文字は UTF-16 で 1 code unit、UTF-8 で 3 bytes
        let text = "あいう";

        // 先頭
        assert_eq!(
            position_to_byte_offset(text, Position { line: 0, character: 0 }),
            0
        );

        // 'い' の位置（UTF-16 character 1）
        assert_eq!(
            position_to_byte_offset(text, Position { line: 0, character: 1 }),
            3
        );

        // 'う' の位置（UTF-16 character 2）
        assert_eq!(
            position_to_byte_offset(text, Position { line: 0, character: 2 }),
            6
        );
    }

    #[test]
    fn test_position_to_byte_offset_mixed_chars() {
        // ASCII と日本語の混合
        let text = "aあb";

        // 'a' の位置
        assert_eq!(
            position_to_byte_offset(text, Position { line: 0, character: 0 }),
            0
        );

        // 'あ' の位置（UTF-16 character 1）
        assert_eq!(
            position_to_byte_offset(text, Position { line: 0, character: 1 }),
            1
        );

        // 'b' の位置（UTF-16 character 2）
        assert_eq!(
            position_to_byte_offset(text, Position { line: 0, character: 2 }),
            4
        );
    }
}
