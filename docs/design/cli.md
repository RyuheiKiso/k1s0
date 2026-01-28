# CLI 設計書

## 概要

k1s0 CLI は、サービスの雛形生成、規約チェック、テンプレート更新支援を行う開発支援ツールです。

## Crate 構成

```
CLI/crates/
├── k1s0-cli/           # CLI メインプログラム
│   └── src/
│       ├── main.rs     # エントリーポイント
│       ├── lib.rs      # CLI 定義（clap）
│       ├── error.rs    # エラー型
│       ├── output.rs   # 出力制御
│       └── commands/   # サブコマンド実装
│           ├── init.rs
│           ├── new_feature.rs
│           ├── new_screen.rs
│           ├── lint.rs
│           ├── upgrade.rs
│           └── completions.rs
│
└── k1s0-generator/     # テンプレートエンジン（別設計書参照）
```

## コマンド一覧

| コマンド | 説明 | 主要オプション |
|---------|------|---------------|
| `init` | リポジトリ初期化 | `--force`, `--template-source` |
| `new-feature` | サービス雛形生成 | `-t/--type`, `-n/--name`, `--with-grpc`, `--with-rest`, `--with-db` |
| `new-screen` | 画面雛形生成 | `-t/--type`, `-s/--screen-id`, `-T/--title`, `-f/--feature-dir` |
| `lint` | 規約違反検査 | `--rules`, `--exclude-rules`, `--strict`, `--fix` |
| `upgrade` | テンプレート更新 | `--check`, `-y/--yes`, `--managed-only` |
| `completions` | シェル補完生成 | `--shell` |

## グローバルオプション

```rust
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// 詳細な出力を有効にする
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// カラー出力を無効にする
    #[arg(long, global = true)]
    pub no_color: bool,

    /// JSON 形式で出力する
    #[arg(long, global = true)]
    pub json: bool,
}
```

## バージョン管理

k1s0 のバージョンは `k1s0-version.txt` ファイルで一元管理されます。

```rust
static VERSION_STRING: Lazy<String> = Lazy::new(|| {
    include_str!("../../../../k1s0-version.txt").trim().to_string()
});
```

---

## init コマンド

### 目的

リポジトリを初期化し、`.k1s0/` ディレクトリと `config.json` を作成する。

### 引数

```rust
pub struct InitArgs {
    /// 初期化するディレクトリ（デフォルト: カレントディレクトリ）
    #[arg(default_value = ".")]
    pub path: String,

    /// 既存の .k1s0/ を上書きする
    #[arg(short, long)]
    pub force: bool,

    /// テンプレートソース（local または registry URL）
    #[arg(long, default_value = "local")]
    pub template_source: String,
}
```

### 処理フロー

```
1. パスの正規化
2. 既存の .k1s0/ 確認
   └─ 存在する場合
      ├─ --force: 削除して続行
      └─ なし: エラー
3. .k1s0/ ディレクトリ作成
4. config.json 作成
5. 完了メッセージ表示
```

### 生成ファイル

`.k1s0/config.json`:

```json
{
  "schema_version": "1.0.0",
  "k1s0_version": "0.1.0",
  "template_source": "local",
  "initialized_at": "2026-01-27T10:00:00Z",
  "project": {
    "default_language": "rust",
    "default_service_type": "backend"
  }
}
```

---

## new-feature コマンド

### 目的

新規サービスの雛形を Tera テンプレートから生成する。

### 引数

```rust
pub struct NewFeatureArgs {
    /// サービスタイプ
    #[arg(short = 't', long = "type", value_enum)]
    pub service_type: ServiceType,

    /// サービス名（kebab-case）
    #[arg(short, long)]
    pub name: String,

    /// 生成先ディレクトリ
    #[arg(short, long)]
    pub output: Option<String>,

    /// 既存のディレクトリを上書きする
    #[arg(short, long)]
    pub force: bool,

    /// gRPC API を含める
    #[arg(long)]
    pub with_grpc: bool,

    /// REST API を含める
    #[arg(long)]
    pub with_rest: bool,

    /// DB マイグレーションを含める
    #[arg(long)]
    pub with_db: bool,
}
```

### サービスタイプ

| タイプ | テンプレートパス | 出力先 | 言語 |
|--------|----------------|-------|------|
| `backend-rust` | `CLI/templates/backend-rust/feature` | `feature/backend/rust/{name}` | rust |
| `backend-go` | `CLI/templates/backend-go/feature` | `feature/backend/go/{name}` | go |
| `frontend-react` | `CLI/templates/frontend-react/feature` | `feature/frontend/react/{name}` | typescript |
| `frontend-flutter` | `CLI/templates/frontend-flutter/feature` | `feature/frontend/flutter/{name}` | dart |

### 処理フロー

```
1. サービス名のバリデーション（kebab-case）
2. 出力パスの決定
3. 既存衝突検査
   └─ 存在する場合
      ├─ --force: 削除して続行
      └─ なし: エラー
4. テンプレートディレクトリの検索
5. fingerprint の算出
6. Tera コンテキストの作成
7. テンプレートの展開
8. manifest.json の作成
9. 完了メッセージ表示
```

### テンプレート変数

| 変数名 | 説明 | 例 |
|--------|------|-----|
| `feature_name` | 機能名（kebab-case） | `user-management` |
| `service_name` | サービス名 | `user-management` |
| `feature_name_snake` | snake_case 変換 | `user_management` |
| `feature_name_pascal` | PascalCase 変換 | `UserManagement` |
| `language` | 言語 | `rust` |
| `service_type` | タイプ | `backend` |
| `k1s0_version` | k1s0 バージョン | `0.1.0` |
| `with_grpc` | gRPC 有効 | `true` |
| `with_rest` | REST 有効 | `false` |
| `with_db` | DB 有効 | `true` |

### サービス名のバリデーション

```rust
fn is_valid_kebab_case(s: &str) -> bool {
    // 1. 空でない
    // 2. 先頭は小文字
    // 3. 末尾はハイフンでない
    // 4. 連続するハイフンがない
    // 5. 許可される文字: 小文字、数字、ハイフン
}
```

有効な例: `user-management`, `order`, `auth-service`, `api2`
無効な例: `UserManagement`, `user_management`, `-user`, `user-`, `user--management`

---

## new-screen コマンド

### 目的

既存の feature ディレクトリに React/Flutter 画面の雛形を生成する。
画面追加の手順を「雛形生成 → 画面実装 → config 追記」に統一する。

### 引数

```rust
pub struct NewScreenArgs {
    /// フロントエンドタイプ
    #[arg(short = 't', long = "type", value_enum, default_value = "react")]
    pub frontend_type: FrontendType,

    /// 画面ID（ドット区切り、例: users.list, users.detail）
    #[arg(short, long)]
    pub screen_id: String,

    /// 画面タイトル
    #[arg(short = 'T', long)]
    pub title: String,

    /// 対象の feature ディレクトリ
    #[arg(short, long)]
    pub feature_dir: String,

    /// メニューに追加する（メニュー設定スニペットを出力）
    #[arg(long)]
    pub with_menu: bool,

    /// URL パス（指定しない場合は screen_id から自動生成）
    #[arg(short, long)]
    pub path: Option<String>,

    /// 必要な権限（カンマ区切り）
    #[arg(long)]
    pub permissions: Option<String>,

    /// 必要な feature flag（カンマ区切り）
    #[arg(long)]
    pub flags: Option<String>,

    /// 既存のファイルを上書きする
    #[arg(short = 'F', long)]
    pub force: bool,
}
```

### フロントエンドタイプ

| タイプ | テンプレートパス | 出力ファイル |
|--------|----------------|-------------|
| `react` | `CLI/templates/frontend-react/screen` | `src/pages/{ComponentName}.tsx` |
| `flutter` | `CLI/templates/frontend-flutter/screen` | `lib/src/presentation/pages/{snake_case}_page.dart` |

### 処理フロー

```
1. screen_id のバリデーション
2. feature_dir の存在確認
3. URL パスの生成（指定がない場合）
4. コンポーネント名・ファイル名の生成
5. テンプレートディレクトリの検索
6. 既存ファイルの確認
   └─ 存在する場合
      ├─ --force: 上書き
      └─ なし: エラー
7. Tera コンテキストの作成
8. テンプレートのレンダリング
9. ファイルの書き込み
10. 設定スニペットの出力
    ├─ React: screens.ts, route.yaml
    └─ Flutter: route.yaml
11. --with-menu: menu.yaml スニペット出力
12. 完了メッセージ表示
```

### テンプレート変数

| 変数名 | 説明 | 例 |
|--------|------|-----|
| `screen_id` | 画面ID | `users.list` |
| `title` | 画面タイトル | `ユーザー一覧` |
| `path` | URL パス | `/users/list` |
| `component_name` | コンポーネント名（PascalCase + Page） | `UsersListPage` |
| `file_name` | ファイル名 | `UsersListPage` (React), `users_list` (Flutter) |
| `permissions` | 必要な権限リスト | `["user.read"]` |
| `flags` | 必要なフラグリスト | `["feature_users"]` |
| `with_menu` | メニュー追加フラグ | `true` |

### 画面ID のバリデーション

```rust
fn is_valid_screen_id(s: &str) -> bool {
    // 1. 空でない
    // 2. 先頭・末尾がドットでない
    // 3. 連続するドットがない
    // 4. 許可される文字: 小文字、数字、ドット、アンダースコア
}
```

有効な例: `home`, `users.list`, `settings.profile.edit`
無効な例: `Users`, `.users`, `users.`, `users..list`

### 生成例

#### React

```bash
k1s0 new-screen -t react -s users.list -T "ユーザー一覧" -f ./my-feature --with-menu
```

生成されるファイル:
- `my-feature/src/pages/UsersListPage.tsx`

出力される設定スニペット:
- `src/config/screens.ts` への追加コード
- `config/default.yaml` の `ui.navigation.routes` への追加
- `config/default.yaml` の `ui.navigation.menu.items` への追加（--with-menu 時）

#### Flutter

```bash
k1s0 new-screen -t flutter -s settings.profile -T "プロフィール設定" -f ./my-feature
```

生成されるファイル:
- `my-feature/lib/src/presentation/pages/settings_profile_page.dart`

出力される設定スニペット:
- `config/default.yaml` の `ui.navigation.routes` への追加

---

## lint コマンド

### 目的

k1s0 の開発規約に対する違反を検査する。

### 引数

```rust
pub struct LintArgs {
    /// 検査するディレクトリ（デフォルト: カレントディレクトリ）
    #[arg(default_value = ".")]
    pub path: String,

    /// 特定のルールのみ実行（カンマ区切り）
    #[arg(long)]
    pub rules: Option<String>,

    /// 特定のルールを除外（カンマ区切り）
    #[arg(long)]
    pub exclude_rules: Option<String>,

    /// 警告をエラーとして扱う
    #[arg(long)]
    pub strict: bool,

    /// 自動修正を試みる
    #[arg(long)]
    pub fix: bool,

    /// 環境変数参照を許可するファイルパス（カンマ区切り、glob パターン対応）
    #[arg(long)]
    pub env_var_allowlist: Option<String>,
}
```

### 処理フロー

```
1. パスの存在確認
2. LintConfig の構築
3. Linter 実行
4. --fix 指定時: 自動修正実行
   └─ 修正後に再検査
5. 結果出力
   ├─ --json: JSON 形式
   └─ なし: 人間向け形式
6. 終了コード決定
```

### 詳細

Lint 機能の詳細は [lint.md](./lint.md) を参照。

---

## upgrade コマンド

### 目的

テンプレートの更新を確認・適用する。

### 引数

```rust
pub struct UpgradeArgs {
    /// 更新するサービスのディレクトリ（デフォルト: カレントディレクトリ）
    #[arg(default_value = ".")]
    pub path: String,

    /// 差分のみ表示し、実際には適用しない
    #[arg(long)]
    pub check: bool,

    /// 対話的な確認なしで適用する
    #[arg(short = 'y', long)]
    pub yes: bool,

    /// managed 領域のみ更新（protected 領域の差分は提示のみ）
    #[arg(long)]
    pub managed_only: bool,

    /// 特定のバージョンにアップグレード
    #[arg(long)]
    pub to_version: Option<String>,

    /// 衝突時にバックアップを作成
    #[arg(long, default_value = "true")]
    pub backup: bool,

    /// DB マイグレーションを自動適用（dev 環境のみ）
    #[arg(long)]
    pub apply_migrations: bool,
}
```

### 処理フロー（--check モード）

```
1. manifest.json の存在確認
2. check_upgrade() 実行
   ├─ manifest 読み込み
   ├─ テンプレートパス決定
   ├─ 新 fingerprint 計算
   ├─ 差分計算
   └─ ADR/UPGRADE.md 確認
3. 差分表示
4. 次のアクション提示
```

### 処理フロー（適用モード）

```
1. manifest.json の存在確認
2. check_upgrade() 実行
3. 更新が不要な場合: 終了
4. 衝突がある場合: エラー
5. MAJOR 変更の場合: 警告
6. 差分表示
7. 確認（--yes でスキップ）
8. apply_upgrade() 実行
   ├─ managed 領域の変更適用
   ├─ バックアップ作成
   ├─ manifest.json 更新
   └─ checksums 更新
9. 結果表示
10. --apply-migrations: マイグレーション適用
```

---

## エラーハンドリング

### エラー型

```rust
pub struct CliError {
    /// エラーの種類
    pub kind: CliErrorKind,
    /// エラーメッセージ
    pub message: String,
    /// 対象（ファイルパス等）
    pub target: Option<String>,
    /// ヒント
    pub hint: Option<String>,
}

pub enum CliErrorKind {
    /// IO エラー
    Io,
    /// 衝突（ファイル/ディレクトリが既に存在）
    Conflict,
    /// バリデーションエラー
    Validation,
    /// manifest が見つからない
    ManifestNotFound,
    /// テンプレートが見つからない
    TemplateNotFound,
    /// 内部エラー
    Internal,
}
```

### 終了コード

```rust
pub enum ExitCode {
    /// 成功
    Success = 0,
    /// 一般的なエラー
    Error = 1,
    /// バリデーションエラー（lint 失敗等）
    ValidationError = 2,
}
```

---

## 出力制御

### OutputConfig

```rust
pub struct OutputConfig {
    /// 出力モード
    pub mode: OutputMode,
    /// カラー出力
    pub color: bool,
    /// 詳細出力
    pub verbose: bool,
}

pub enum OutputMode {
    /// 人間向け出力
    Human,
    /// JSON 出力
    Json,
}
```

### Output トレイト

```rust
impl Output {
    pub fn header(&self, text: &str);
    pub fn info(&self, text: &str);
    pub fn success(&self, text: &str);
    pub fn warning(&self, text: &str);
    pub fn error(&self, err: &CliError);
    pub fn list_item(&self, key: &str, value: &str);
    pub fn file_added(&self, path: &str);
    pub fn hint(&self, text: &str);
    pub fn newline(&self);
    pub fn print_json<T: Serialize>(&self, value: &T);
}
```

---

## 依存ライブラリ

| ライブラリ | バージョン | 用途 |
|-----------|----------|------|
| clap | 4.5 | CLI パーサー |
| clap_complete | 4.5 | シェル補完 |
| serde | 1.0 | シリアライゼーション |
| serde_json | 1.0 | JSON 処理 |
| chrono | 0.4 | 日時操作 |
| console | 0.15 | コンソール出力 |
| indicatif | 0.17 | プログレスバー |
| tokio | 1.0 | 非同期ランタイム |
| once_cell | 1.19 | 遅延初期化 |

---

## k1s0-lsp

### 概要

k1s0-lsp は、manifest.json ファイル用の Language Server Protocol (LSP) サーバーです。VSCode やその他の LSP 対応エディタで、インテリジェントな編集支援を提供します。

### Crate 構成

```
CLI/crates/k1s0-lsp/
└── src/
    ├── lib.rs          # LSP サーバー本体
    ├── completion.rs   # 補完機能
    ├── hover.rs        # ホバー情報
    ├── diagnostics.rs  # 診断機能
    ├── definition.rs   # 定義ジャンプ
    ├── references.rs   # 参照検索
    └── symbols.rs      # シンボル機能
```

### 機能一覧

| 機能 | 説明 | 状態 |
|------|------|:----:|
| 補完（Completion） | キー/値の自動補完、スニペット | ✅ |
| ホバー（Hover） | キーの説明、値の型情報 | ✅ |
| 診断（Diagnostics） | JSON 構文エラー、スキーマバリデーション | ✅ |
| 定義ジャンプ（Go to Definition） | テンプレート/crate への移動 | ✅ |
| 参照検索（Find References） | 値の使用箇所を検索 | ✅ |
| ドキュメントシンボル（Document Symbol） | ファイル内のシンボル一覧 | ✅ |
| ワークスペースシンボル（Workspace Symbol） | プロジェクト全体のシンボル検索 | ✅ |

### 補完機能

manifest.json のコンテキストに応じた補完候補を提供します。

#### 対応するコンテキスト

| コンテキスト | 補完内容 |
|-------------|---------|
| トップレベルキー | `schema_version`, `template`, `service`, `dependencies` |
| `template.name` | テンプレート名一覧 |
| `template.version` | セマンティックバージョン形式 |
| `dependencies.framework_crates[].name` | Framework crate 名一覧 |

#### 使用例

```json
{
  "template": {
    "name": "|"  // ← ここで補完: backend-rust, backend-go, frontend-react, frontend-flutter
  }
}
```

### ホバー機能

カーソル位置のキーや値に関する詳細情報を表示します。

#### 対応する情報

| 対象 | 表示内容 |
|------|---------|
| `schema_version` | スキーマバージョンの説明、有効な値 |
| `template.name` | テンプレートの説明、使用可能なオプション |
| `dependencies.framework_crates` | crate の説明、依存関係 |

### 定義ジャンプ機能

manifest.json 内の参照から、定義元へジャンプします。

#### 対応するジャンプ先

| 参照元 | ジャンプ先 |
|--------|----------|
| `template.path` | テンプレートディレクトリ |
| `template.name` | `CLI/templates/{name}/` |
| `dependencies.framework_crates[].name` | `framework/backend/rust/crates/{name}/Cargo.toml` |

#### 実装

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

### 参照検索機能

manifest.json 内の値が他のファイルで参照されている箇所を検索します。

#### 対応する検索対象

- テンプレート名の参照
- Framework crate 名の参照
- サービス名の参照

#### 実装

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

### シンボル機能

#### ドキュメントシンボル

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

#### ワークスペースシンボル

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

### LSP サーバー設定

#### サーバーケイパビリティ

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

#### 起動方法

```bash
# stdio モードで起動
k1s0-lsp

# VSCode 拡張機能から自動起動
```

### VSCode 拡張機能との統合

VSCode 拡張機能 `k1s0-vscode` は、k1s0-lsp を内蔵し、以下の機能を提供します:

1. manifest.json の補完・ホバー・診断
2. テンプレートへのジャンプ
3. Framework crate へのジャンプ
4. 参照検索
5. シンボル一覧（Outline）
6. ワークスペースシンボル検索（Ctrl+T）

### テスト

k1s0-lsp は包括的なユニットテストを備えています。177個以上のテストが各モジュールに実装されており、高いコードカバレッジを達成しています。

#### テストの実行方法

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

#### テスト内容

| モジュール | テスト数 | 主なテスト内容 |
|-----------|---------|---------------|
| lib.rs | ~27 | `position_to_byte_offset`、`apply_incremental_change`、`LspConfig` |
| completion.rs | ~27 | `analyze_context`、`extract_json_path`、`get_completions` |
| hover.rs | ~27 | `find_key_in_line`、`build_key_path`、`get_hover_info` |
| definition.rs | ~22 | `extract_key_value_at_position`、`find_definition`、セクション判定 |
| references.rs | ~25 | `extract_target_at_position`、`find_value_references`、manifest検索 |
| symbols.rs | ~25 | `extract_document_symbols`、`search_workspace_symbols` |
| schema.rs | ~32 | スキーマキー検索、補完アイテム生成、値の型判定 |

#### テストの特徴

- **UTF-16/UTF-8 変換**: LSP の Position（UTF-16 code unit）とバイトオフセットの変換を検証
- **日本語・絵文字対応**: マルチバイト文字を含むテキストの処理を検証
- **エッジケース**: 空ドキュメント、範囲外アクセス、不正な JSON などの境界条件を網羅
- **ファイルシステム操作**: `tempfile` クレートを使用した一時ディレクトリでの実際のファイル操作テスト

---

---

## new-domain コマンド

### 目的

domain 層の雛形を生成する。domain 層は、複数の feature で共有されるビジネスロジックを管理する中間層です。

### 引数

```rust
pub struct NewDomainArgs {
    /// ドメインタイプ
    #[arg(short = 't', long = "type", value_enum)]
    pub domain_type: DomainType,

    /// ドメイン名（kebab-case）
    #[arg(short, long)]
    pub name: String,

    /// 生成先ディレクトリ
    #[arg(short, long)]
    pub output: Option<String>,

    /// 既存のディレクトリを上書きする
    #[arg(short, long)]
    pub force: bool,
}
```

### ドメインタイプ

| タイプ | テンプレートパス | 出力先 | 言語 |
|--------|----------------|-------|------|
| `backend-rust` | `CLI/templates/backend-rust/domain` | `domain/backend/rust/{name}` | rust |
| `backend-go` | `CLI/templates/backend-go/domain` | `domain/backend/go/{name}` | go |
| `frontend-react` | `CLI/templates/frontend-react/domain` | `domain/frontend/react/{name}` | typescript |
| `frontend-flutter` | `CLI/templates/frontend-flutter/domain` | `domain/frontend/flutter/{name}` | dart |

### 処理フロー

```
1. ドメイン名のバリデーション（kebab-case）
2. 予約語チェック（framework, feature, domain, k1s0, common, shared）
3. 出力パスの決定
4. 既存衝突検査
   └─ 存在する場合
      ├─ --force: 削除して続行
      └─ なし: エラー
5. テンプレートディレクトリの検索
6. fingerprint の算出
7. Tera コンテキストの作成
8. テンプレートの展開
9. manifest.json の作成（layer: domain, version: 0.1.0）
10. 完了メッセージ表示
```

### テンプレート変数

| 変数名 | 説明 | 例 |
|--------|------|-----|
| `domain_name` | ドメイン名（kebab-case） | `production` |
| `domain_name_snake` | snake_case 変換 | `production` |
| `domain_name_pascal` | PascalCase 変換 | `Production` |
| `language` | 言語 | `rust` |
| `service_type` | タイプ | `backend` |
| `k1s0_version` | k1s0 バージョン | `0.1.0` |

### 生成される manifest.json

```json
{
  "schema_version": "1.0.0",
  "k1s0_version": "0.1.0",
  "template": {
    "name": "backend-rust",
    "version": "0.1.0",
    "source": "local",
    "path": "CLI/templates/backend-rust/domain",
    "fingerprint": "abc123..."
  },
  "service": {
    "service_name": "production",
    "language": "rust",
    "type": "backend"
  },
  "layer": "domain",
  "version": "0.1.0",
  "min_framework_version": "0.1.0",
  "dependencies": {
    "framework": ["k1s0-error", "k1s0-config"]
  }
}
```

### 使用例

```bash
# 基本的な使用法
k1s0 new-domain --type backend-rust --name production

# カスタム出力先
k1s0 new-domain --type backend-rust --name production --output ./my-domains

# 上書き
k1s0 new-domain --type backend-rust --name production --force
```

---

## domain コマンド

### 目的

domain の管理（一覧表示、バージョン管理、依存関係分析）を行う。

### サブコマンド

#### domain list

```bash
k1s0 domain list

# 出力例
Domains:
  production          0.1.0    domain/backend/rust/production
  inventory           1.2.0    domain/backend/rust/inventory
  user-management     2.0.0    domain/backend/go/user-management
```

#### domain version

```bash
# バージョン確認
k1s0 domain version --name production

# バージョン更新
k1s0 domain version --name production --bump patch
k1s0 domain version --name production --bump minor
k1s0 domain version --name production --bump major

# 直接指定
k1s0 domain version --name production --set 2.0.0

# 破壊的変更を記録
k1s0 domain version --name production --bump major \
  --message "WorkOrder.quantity の型を変更"
```

#### domain dependents

```bash
k1s0 domain dependents --name production

# 出力例
Features depending on 'production':
  work-order-api          ^1.2.0    feature/backend/rust/work-order-api
  work-order-dashboard    ^1.5.0    feature/frontend/react/work-order-dashboard
  production-report       ^1.0.0    feature/backend/rust/production-report
```

#### domain impact

```bash
k1s0 domain impact --name production --from 1.5.0 --to 2.0.0

# 出力例
Domain: production
Version change: 1.5.0 -> 2.0.0 (MAJOR)

Breaking changes:
  - 2.0.0: WorkOrder.quantity の型を u32 から Quantity 値オブジェクトに変更

Affected features (3):
  - work-order-api (constraint: ^1.2.0) - INCOMPATIBLE
  - work-order-dashboard (constraint: ^1.5.0) - INCOMPATIBLE
  - production-report (constraint: ^1.0.0) - INCOMPATIBLE
```

---

## 今後の拡張予定

1. **registry サポート**: リモートテンプレートレジストリからのテンプレート取得
2. **プラグインシステム**: カスタムコマンドの追加
3. **設定ファイル**: `.k1s0/settings.yaml` によるデフォルト設定
4. **watch モード**: ファイル変更時の自動 lint
5. **LSP 拡張**: コードアクション、リネーム、フォーマット対応
