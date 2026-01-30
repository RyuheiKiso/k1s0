# k1s0-lsp

← [CLI 設計書](./)

## 概要

k1s0-lsp は、manifest.json ファイル用の Language Server Protocol (LSP) サーバーです。VSCode やその他の LSP 対応エディタで、インテリジェントな編集支援を提供します。

## Crate 構成

```
CLI/crates/k1s0-lsp/
└── src/
    ├── lib.rs          # LSP サーバー本体
    ├── main.rs         # エントリーポイント
    ├── schema.rs       # スキーマ定義
    ├── completion.rs   # 補完機能
    ├── hover.rs        # ホバー情報
    ├── definition.rs   # 定義ジャンプ
    ├── references.rs   # 参照検索
    └── symbols.rs      # シンボル機能
```

## 機能一覧

| 機能 | 説明 | 状態 |
|------|------|:----:|
| 補完（Completion） | キー/値の自動補完、スニペット | ✅ |
| ホバー（Hover） | キーの説明、値の型情報 | ✅ |
| 診断（Diagnostics） | JSON 構文エラー、スキーマバリデーション | ✅ |
| 定義ジャンプ（Go to Definition） | テンプレート/crate への移動 | ✅ |
| 参照検索（Find References） | 値の使用箇所を検索 | ✅ |
| ドキュメントシンボル（Document Symbol） | ファイル内のシンボル一覧 | ✅ |
| ワークスペースシンボル（Workspace Symbol） | プロジェクト全体のシンボル検索 | ✅ |

## 補完機能

manifest.json のコンテキストに応じた補完候補を提供します。

### 対応するコンテキスト

| コンテキスト | 補完内容 |
|-------------|---------|
| トップレベルキー | `schema_version`, `template`, `service`, `dependencies` |
| `template.name` | テンプレート名一覧 |
| `template.version` | セマンティックバージョン形式 |
| `dependencies.framework_crates[].name` | Framework crate 名一覧 |

### 使用例

```json
{
  "template": {
    "name": "|"  // ← ここで補完: backend-rust, backend-go, frontend-react, frontend-flutter
  }
}
```

## ホバー機能

カーソル位置のキーや値に関する詳細情報を表示します。

### 対応する情報

| 対象 | 表示内容 |
|------|---------|
| `schema_version` | スキーマバージョンの説明、有効な値 |
| `template.name` | テンプレートの説明、使用可能なオプション |
| `dependencies.framework_crates` | crate の説明、依存関係 |

## 定義ジャンプ機能

manifest.json 内の参照から、定義元へジャンプします。

### 対応するジャンプ先

| 参照元 | ジャンプ先 |
|--------|----------|
| `template.path` | テンプレートディレクトリ |
| `template.name` | `CLI/templates/{name}/` |
| `dependencies.framework_crates[].name` | `framework/backend/rust/crates/{name}/Cargo.toml` |

### 実装

```rust
pub fn find_definition(
    content: &str,
    position: Position,
    workspace_root: Option<&PathBuf>,
) -> Option<GotoDefinitionResponse> {
    // 1. カーソル位置のキー/値を抽出
    // 2. コンテキストを判定（template, framework_crates 等）
    // 3. 対応するファイル/ディレクトリを検索
    // 4. Location を返す
}
```

## 参照検索機能

manifest.json 内の値が他のファイルで参照されている箇所を検索します。

### 対応する検索対象

- テンプレート名の参照
- Framework crate 名の参照
- サービス名の参照

### 実装

```rust
pub fn find_references(
    uri: &Url,
    content: &str,
    position: Position,
    workspace_root: Option<&PathBuf>,
    include_declaration: bool,
) -> Vec<Location> {
    // 1. カーソル位置のキー/値を抽出
    // 2. ワークスペース内の manifest.json を検索
    // 3. 同じ値を持つ箇所を収集
    // 4. Location のリストを返す
}
```

## シンボル機能

### ドキュメントシンボル

manifest.json 内のシンボル（キー）をツリー構造で表示します。

```rust
pub fn extract_document_symbols(content: &str) -> Vec<DocumentSymbol> {
    // JSON をパースしてシンボルツリーを構築
    // 各キーを SymbolKind に応じて分類
    // - Object → OBJECT
    // - Array → ARRAY
    // - String → STRING
    // - Number → NUMBER
    // - Boolean → BOOLEAN
}
```

### ワークスペースシンボル

プロジェクト全体の manifest.json からシンボルを検索します。

```rust
pub fn search_workspace_symbols(
    query: &str,
    manifest_files: &[(Url, String)],
) -> Vec<SymbolInformation> {
    // 1. すべての manifest.json を走査
    // 2. クエリにマッチするキーを収集
    // 3. SymbolInformation のリストを返す
}
```

## LSP サーバー設定

### サーバーケイパビリティ

```rust
ServerCapabilities {
    text_document_sync: Some(TextDocumentSyncCapability::Kind(
        TextDocumentSyncKind::FULL,
    )),
    completion_provider: Some(CompletionOptions {
        trigger_characters: Some(vec!["\"".to_string(), ":".to_string()]),
        ..Default::default()
    }),
    hover_provider: Some(HoverProviderCapability::Simple(true)),
    definition_provider: Some(OneOf::Left(true)),
    references_provider: Some(OneOf::Left(true)),
    document_symbol_provider: Some(OneOf::Left(true)),
    workspace_symbol_provider: Some(OneOf::Left(true)),
    ..Default::default()
}
```

### 起動方法

```bash
# stdio モードで起動
k1s0-lsp

# VSCode 拡張機能から自動起動
```

## VSCode 拡張機能との統合

VSCode 拡張機能 `k1s0-vscode` は、k1s0-lsp を内蔵し、以下の機能を提供します:

1. manifest.json の補完・ホバー・診断
2. テンプレートへのジャンプ
3. Framework crate へのジャンプ
4. 参照検索
5. シンボル一覧（Outline）
6. ワークスペースシンボル検索（Ctrl+T）

## テスト

k1s0-lsp は包括的なユニットテストを備えています。177個以上のテストが各モジュールに実装されており、高いコードカバレッジを達成しています。

### テストの実行方法

```bash
# CLI ディレクトリからすべてのテストを実行
cd CLI
cargo test --all

# k1s0-lsp のみテスト
cargo test -p k1s0-lsp

# 特定のモジュールのテスト
cargo test -p k1s0-lsp completion::
cargo test -p k1s0-lsp hover::
```

### テスト内容

| モジュール | テスト数 | 主なテスト内容 |
|-----------|---------|---------------|
| lib.rs | ~27 | `position_to_byte_offset`、`apply_incremental_change`、`LspConfig` |
| completion.rs | ~27 | `analyze_context`、`extract_json_path`、`get_completions` |
| hover.rs | ~27 | `find_key_in_line`、`build_key_path`、`get_hover_info` |
| definition.rs | ~22 | `extract_key_value_at_position`、`find_definition`、セクション判定 |
| references.rs | ~25 | `extract_target_at_position`、`find_value_references`、manifest検索 |
| symbols.rs | ~25 | `extract_document_symbols`、`search_workspace_symbols` |
| schema.rs | ~32 | スキーマキー検索、補完アイテム生成、値の型判定 |

### テストの特徴

- **UTF-16/UTF-8 変換**: LSP の Position（UTF-16 code unit）とバイトオフセットの変換を検証
- **日本語・絵文字対応**: マルチバイト文字を含むテキストの処理を検証
- **エッジケース**: 空ドキュメント、範囲外アクセス、不正な JSON などの境界条件を網羅
- **ファイルシステム操作**: `tempfile` クレートを使用した一時ディレクトリでの実際のファイル操作テスト
