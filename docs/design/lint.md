# Lint 機能設計書

## 概要

k1s0 の Lint 機能は、開発規約に対する違反を検査し、一部は自動修正を提供します。

## モジュール構成

```
CLI/crates/k1s0-generator/src/lint/
├── mod.rs           # モジュール公開
├── types.rs         # 型定義（RuleId, Severity, Violation, LintResult, LintConfig）
├── linter.rs        # Linter 本体（manifest/必須ファイル検査）
├── required_files.rs# 必須ディレクトリ/ファイル定義
├── env_vars.rs      # K020: 環境変数参照検査
├── secret_config.rs # K021: 機密直書き検査
├── dependency.rs    # K022: 依存方向違反検査
├── retry.rs         # K030-K032: gRPC リトライ設定検査
├── fixer.rs         # 自動修正ロジック
├── utils.rs         # ユーティリティ関数
└── tests.rs         # テスト
```

---

## ルール一覧

| ID | 重要度 | 説明 | 自動修正 |
|----|--------|------|---------|
| K001 | Error | manifest.json が存在しない | - |
| K002 | Error | manifest.json の必須キーが不足 | - |
| K003 | Error | manifest.json の値が不正 | - |
| K010 | Error | 必須ディレクトリが存在しない | ✓ |
| K011 | Error | 必須ファイルが存在しない | ✓ |
| K020 | Error | 環境変数参照の禁止 | - |
| K021 | Error | config YAML への機密直書き禁止 | - |
| K022 | Error | Clean Architecture 依存方向違反 | - |
| K030 | Warning | gRPC リトライ設定の検出（可視化） | - |
| K031 | Warning | gRPC リトライ設定に ADR 参照がない | - |
| K032 | Warning | gRPC リトライ設定が不完全 | - |
| K040 | Error | 層間依存の基本違反 | - |
| K041 | Error | domain が見つからない | - |
| K042 | Error | domain バージョン制約不整合 | - |
| K043 | Error | 循環依存の検出 | - |
| K044 | Warning | 非推奨 domain の使用 | - |
| K045 | Warning | min_framework_version 違反 | - |
| K046 | Warning | breaking_changes の影響 | - |
| K047 | Error | domain 層の version 未設定 | - |

---

## 型定義

### RuleId

```rust
pub enum RuleId {
    ManifestNotFound,        // K001
    ManifestMissingKey,      // K002
    ManifestInvalidValue,    // K003
    RequiredDirMissing,      // K010
    RequiredFileMissing,     // K011
    EnvVarUsage,             // K020
    SecretInConfig,          // K021
    DependencyDirection,     // K022
    RetryUsageDetected,      // K030
    RetryWithoutAdr,         // K031
    RetryConfigIncomplete,   // K032
    // 層間依存関係ルール（K040-K047）
    LayerDependencyViolation,    // K040
    DomainNotFound,              // K041
    DomainVersionMismatch,       // K042
    CircularDependency,          // K043
    DeprecatedDomainUsage,       // K044
    MinFrameworkVersionViolation,// K045
    BreakingChangeImpact,        // K046
    DomainVersionMissing,        // K047
}

impl RuleId {
    /// ルール ID を文字列として取得
    pub fn as_str(&self) -> &'static str;

    /// ルールの説明
    pub fn description(&self) -> &'static str;
}
```

### Severity

```rust
pub enum Severity {
    /// エラー（lint 失敗）
    Error,
    /// 警告（lint 成功だが注意）
    Warning,
}
```

### Violation

```rust
pub struct Violation {
    /// ルール ID
    pub rule: RuleId,
    /// 重要度
    pub severity: Severity,
    /// メッセージ
    pub message: String,
    /// 対象パス
    pub path: Option<String>,
    /// 行番号
    pub line: Option<usize>,
    /// ヒント
    pub hint: Option<String>,
}
```

### LintResult

```rust
pub struct LintResult {
    /// 検査したパス
    pub path: PathBuf,
    /// 違反リスト
    pub violations: Vec<Violation>,
}

impl LintResult {
    /// エラーの数
    pub fn error_count(&self) -> usize;

    /// 警告の数
    pub fn warning_count(&self) -> usize;

    /// 成功かどうか（エラーがないか）
    pub fn is_success(&self) -> bool;
}
```

### LintConfig

```rust
pub struct LintConfig {
    /// 実行するルール（None の場合は全て）
    pub rules: Option<Vec<String>>,
    /// 除外するルール
    pub exclude_rules: Vec<String>,
    /// 警告をエラーとして扱う
    pub strict: bool,
    /// 環境変数参照を許可するファイルパス（glob パターン）
    pub env_var_allowlist: Vec<String>,
    /// 自動修正を試みる
    pub fix: bool,
}
```

---

## Linter 本体

### API

```rust
impl Linter {
    /// 新しい linter を作成
    pub fn new(config: LintConfig) -> Self;

    /// デフォルト設定で作成
    pub fn default_linter() -> Self;

    /// ディレクトリを検査する
    pub fn lint<P: AsRef<Path>>(&self, path: P) -> LintResult;
}
```

### 処理フロー

```
1. manifest の検査
   ├─ K001: manifest.json の存在確認
   ├─ K002: 必須キーの検査
   └─ K003: 値の妥当性検査
2. 必須ファイルの検査
   ├─ K010: 必須ディレクトリの検査
   └─ K011: 必須ファイルの検査
3. K020: 環境変数参照の検査
4. K021: config YAML への機密直書き検査
5. K022: Clean Architecture 依存方向検査
6. K030/K031/K032: gRPC リトライ設定検査
7. strict モードの場合: 警告をエラーに昇格
```

---

## K001-K003: manifest 検査

### K001: manifest.json の存在確認

```
対象: .k1s0/manifest.json
重要度: Error
ヒント: k1s0 new-feature で生成したプロジェクトか確認してください
```

### K002: 必須キーの検査

**必須キー（Error）:**
- `k1s0_version`
- `template.name`
- `template.version`
- `template.fingerprint`
- `service.service_name`
- `service.language`

**必須キー（Warning）:**
- `managed_paths`
- `protected_paths`

### K003: 値の妥当性検査

**service.language:**
```rust
const VALID_LANGUAGES: &[&str] = &["rust", "go", "typescript", "dart"];
```

**service.service_type:**
```rust
const VALID_TYPES: &[&str] = &["backend", "frontend", "bff"];
```

**template.name:**
```rust
const VALID_TEMPLATES: &[&str] = &[
    "backend-rust",
    "backend-go",
    "frontend-react",
    "frontend-flutter",
];
```

---

## K010-K011: 必須ファイル検査

### RequiredFiles

```rust
pub struct RequiredFiles {
    /// 必須ディレクトリ
    pub directories: Vec<&'static str>,
    /// 必須ファイル
    pub files: Vec<&'static str>,
}

impl RequiredFiles {
    /// テンプレート名から必須ファイルを取得
    pub fn from_template_name(name: &str) -> Option<Self>;
}
```

### backend-rust の必須ファイル

```rust
RequiredFiles {
    directories: vec![
        "src",
        "src/domain",
        "src/application",
        "src/presentation",
        "src/infrastructure",
        "config",
        "deploy",
    ],
    files: vec![
        "Cargo.toml",
        "src/main.rs",
        "config/default.yaml",
        ".k1s0/manifest.json",
    ],
}
```

### backend-go の必須ファイル

```rust
RequiredFiles {
    directories: vec![
        "cmd",
        "internal/domain",
        "internal/application",
        "internal/presentation",
        "internal/infrastructure",
        "config",
        "deploy",
    ],
    files: vec![
        "go.mod",
        "config/default.yaml",
        ".k1s0/manifest.json",
    ],
}
```

### frontend-react の必須ファイル

```rust
RequiredFiles {
    directories: vec![
        "src",
        "src/domain",
        "src/application",
        "src/presentation",
        "public",
    ],
    files: vec![
        "package.json",
        "tsconfig.json",
        ".k1s0/manifest.json",
    ],
}
```

### frontend-flutter の必須ファイル

```rust
RequiredFiles {
    directories: vec![
        "lib",
        "lib/src/domain",
        "lib/src/application",
        "lib/src/presentation",
    ],
    files: vec![
        "pubspec.yaml",
        ".k1s0/manifest.json",
    ],
}
```

---

## K020: 環境変数参照の禁止

### 目的

環境変数の直接参照を禁止し、設定ファイル経由での設定読み込みを強制する。

### 検査対象パターン

**Rust:**
```rust
const ENV_VAR_PATTERNS: &[&str] = &[
    "std::env::var",
    "std::env::var_os",
    "env::var",
    "env::var_os",
    "env!(",
    "option_env!(",
];
```

**Go:**
```rust
const ENV_VAR_PATTERNS: &[&str] = &[
    "os.Getenv",
    "os.LookupEnv",
    "os.ExpandEnv",
];
```

**TypeScript:**
```rust
const ENV_VAR_PATTERNS: &[&str] = &[
    "process.env",
];
```

**Dart:**
```rust
const ENV_VAR_PATTERNS: &[&str] = &[
    "Platform.environment",
];
```

### 除外パターン

- テストファイル（`*_test.rs`, `*_test.go`, `*.test.ts`, `*_test.dart`）
- `--env-var-allowlist` で指定されたファイル

### 違反例

```rust
// K020 違反: 環境変数の直接参照
let db_url = std::env::var("DATABASE_URL").unwrap();
```

### 正しい実装

```rust
// k1s0-config を使用
let config = ConfigLoader::new(options)?;
let db_config: DbConfig = config.load()?;
```

---

## K021: config YAML への機密直書き禁止

### 目的

機密情報を YAML ファイルに直接記述することを禁止する。

### 検査対象キー

```rust
const SECRET_KEY_PATTERNS: &[&str] = &[
    "password",
    "secret",
    "api_key",
    "apikey",
    "api-key",
    "token",
    "credential",
    "private_key",
    "privatekey",
    "private-key",
];
```

### 許可されるパターン

```rust
// OK: _file サフィックスで外部ファイルを参照
password_file: /var/run/secrets/db-password

// OK: 空値
password:

// OK: プレースホルダ
password: ${DB_PASSWORD}  # 環境変数展開は別の仕組みで
```

### 違反例

```yaml
# K021 違反: 機密情報の直接記述
database:
  password: my-secret-password
```

### 正しい実装

```yaml
# _file サフィックスで外部ファイルを参照
database:
  password_file: /var/run/secrets/k1s0/db-password
```

---

## K022: Clean Architecture 依存方向違反

### 目的

Clean Architecture の依存方向ルールを強制する。

### 依存ルール

```
外側 → 内側 のみ許可

presentation → application → domain ← infrastructure

禁止される依存:
- domain → application
- domain → presentation
- domain → infrastructure
- application → presentation
```

### 検査方法

ファイルパスとインポート文を解析して依存方向を検査する。

```rust
// domain 層のファイルで application をインポート
// K022 違反
mod domain {
    use crate::application::services::UserService;  // NG
}
```

### 層の判定

```rust
fn get_layer(path: &str) -> Option<Layer> {
    if path.contains("/domain/") { Some(Layer::Domain) }
    else if path.contains("/application/") { Some(Layer::Application) }
    else if path.contains("/presentation/") { Some(Layer::Presentation) }
    else if path.contains("/infrastructure/") { Some(Layer::Infrastructure) }
    else { None }
}
```

---

## K030-K032: gRPC リトライ設定検査

### K030: リトライ設定の検出（可視化）

```
重要度: Warning
目的: リトライ設定が存在することを開発者に認識させる
```

### K031: ADR 参照がない

```
重要度: Warning
目的: リトライ設定に関する設計決定が文書化されていることを確認
検査: コメントに ADR-XXXX への参照があるか
```

### K032: 設定が不完全

```
重要度: Warning
目的: リトライ設定の必須項目が揃っているか確認
```

**必須項目:**
- `max_attempts`: 最大リトライ回数
- `initial_backoff`: 初期バックオフ
- `max_backoff`: 最大バックオフ
- `backoff_multiplier`: バックオフ乗数
- `retryable_status_codes`: リトライ対象ステータスコード

### 検査例

```yaml
# gRPC リトライ設定
grpc:
  client:
    retry:
      # ADR-0005 参照  ← K031 OK
      max_attempts: 3
      initial_backoff: 100ms
      max_backoff: 1s
      backoff_multiplier: 2.0
      retryable_status_codes:
        - UNAVAILABLE
        - DEADLINE_EXCEEDED
```

---

## 自動修正（Fixer）

### API

```rust
impl Fixer {
    /// 新しい Fixer を作成
    pub fn new(base_path: &Path) -> Self;

    /// 違反を修正する
    pub fn fix(&self, violation: &Violation) -> Option<FixResult>;

    /// ルールが自動修正可能かどうか
    pub fn is_fixable(rule: RuleId) -> bool;
}
```

### 修正可能なルール

| ルール | 修正内容 |
|--------|---------|
| K010 | ディレクトリを作成 |
| K011 | 空ファイルを作成 |

### FixResult

```rust
pub struct FixResult {
    /// 修正したファイルパス
    pub path: PathBuf,
    /// 修正の説明
    pub description: String,
    /// 成功したかどうか
    pub success: bool,
    /// エラーメッセージ（失敗時）
    pub error: Option<String>,
}
```

---

## CLI 統合

### 使用例

```bash
# 基本的な lint 実行
k1s0 lint

# 特定のルールのみ実行
k1s0 lint --rules K001,K002,K003

# 特定のルールを除外
k1s0 lint --exclude-rules K030,K031,K032

# 警告をエラーとして扱う（CI 向け）
k1s0 lint --strict

# 自動修正を試みる
k1s0 lint --fix

# JSON 出力
k1s0 lint --json

# 環境変数参照を許可するファイルを指定
k1s0 lint --env-var-allowlist "tests/**/*,scripts/**/*"
```

### JSON 出力形式

```json
{
  "error": true,
  "path": "feature/backend/rust/user-service",
  "violation_count": 2,
  "warning_count": 1,
  "violations": [
    {
      "rule": "K001",
      "severity": "error",
      "message": "manifest.json が見つかりません",
      "path": ".k1s0/manifest.json",
      "line": null
    },
    {
      "rule": "K030",
      "severity": "warning",
      "message": "gRPC リトライ設定が検出されました",
      "path": "config/default.yaml",
      "line": 42
    }
  ]
}
```

---

## LSP 統合

### 概要

k1s0-lsp は Language Server Protocol を実装し、エディタ/IDE と連携して lint 結果をリアルタイムで提供します。

### モジュール構成

```
CLI/crates/k1s0-lsp/
├── Cargo.toml
├── src/
│   ├── lib.rs           # LSP サーバ本体
│   ├── main.rs          # エントリポイント
│   ├── schema.rs        # manifest.json スキーマ定義
│   ├── completion.rs    # 補完機能
│   └── hover.rs         # ホバー情報機能
```

### サポート機能

| 機能 | 説明 |
|------|------|
| `textDocument/publishDiagnostics` | lint 結果を診断情報として送信 |
| `textDocument/didOpen` | ファイル開時に lint 実行 |
| `textDocument/didSave` | ファイル保存時に lint 実行 |
| `textDocument/didChange` | ファイル変更時に lint 実行（デバウンス付き） |
| `textDocument/completion` | manifest.json の入力補完 |
| `textDocument/hover` | manifest.json キーのホバー情報 |

### 起動方法

```bash
# stdio モードで起動
k1s0-lsp --stdio

# TCP モードで起動（デバッグ用）
k1s0-lsp --tcp --port 9257
```

### 補完機能

manifest.json 編集時に以下の補完を提供：

**キー補完:**
- ルートレベルのキー（`schema_version`, `template`, `service` 等）
- ネストされたキー（`template.name`, `service.language` 等）

**値補完:**
- enum 型の値（`rust`, `go`, `backend`, `frontend` 等）
- 例に基づく値の提案

### ホバー情報

manifest.json のキーにカーソルを合わせると以下の情報を表示：

- キーの説明
- 必須/オプションの区別
- 型情報（enum の場合は有効な値一覧）
- 使用例

### 設定

```rust
pub struct LspConfig {
    /// lint 設定
    pub lint: LintConfig,
    /// ファイル変更時の lint を有効にするか
    pub lint_on_change: bool,
    /// デバウンス間隔（ミリ秒）
    pub debounce_ms: u64,  // デフォルト: 500
}
```

### VS Code 統合例

```json
// .vscode/settings.json
{
  "k1s0.lsp.path": "/path/to/k1s0-lsp",
  "k1s0.lsp.args": ["--stdio"]
}
```

---

## K040-K047: 層間依存関係検査

### 概要

3層アーキテクチャ（framework -> domain -> feature）における層間の依存関係を検証します。

### K040: 層間依存の基本違反

```
重要度: Error
対象: 全ての manifest.json
```

層間依存の基本ルール:
- feature は domain または framework に依存可能
- domain は framework に依存可能
- framework は何にも依存しない（最下層）

違反例:
- domain 層の manifest.json に `domain` フィールドが設定されている
- framework 層の manifest.json に `domain` フィールドが設定されている

### K041: domain が見つからない

```
重要度: Error
対象: feature 層の manifest.json
```

feature が依存する domain が存在するか確認します。

検査対象:
- manifest.json の `domain` フィールド
- manifest.json の `dependencies.domain` オブジェクト

### K042: domain バージョン制約不整合

```
重要度: Error
対象: feature 層の manifest.json
```

feature が指定するバージョン制約と、domain の実際のバージョンが一致するか確認します。

例:
```json
// feature の manifest.json
{
  "domain": "production",
  "domain_version": "^1.2.0"
}
```

```json
// domain の manifest.json
{
  "version": "0.9.0"  // ^1.2.0 を満たさない -> K042 違反
}
```

### K043: 循環依存の検出

```
重要度: Error
対象: domain 層の manifest.json
```

domain 間の循環依存を検出します。

例:
- domain-a が domain-b に依存
- domain-b が domain-a に依存
-> 循環依存

### K044: 非推奨 domain の使用

```
重要度: Warning
対象: feature 層の manifest.json
```

非推奨（deprecated）の domain を使用している場合に警告します。

### K045: min_framework_version 違反

```
重要度: Warning
対象: domain 層、feature 層の manifest.json
```

domain の `min_framework_version` と k1s0 のバージョンを比較し、
要件を満たしていない場合に警告します。

### K046: breaking_changes の影響

```
重要度: Warning
対象: feature 層の manifest.json
```

依存する domain に破壊的変更がある場合に警告します。

### K047: domain 層の version 未設定

```
重要度: Error
対象: domain 層の manifest.json
```

domain 層には `version` フィールドが必須です。

---

## 今後の拡張予定

1. **カスタムルール**: ユーザー定義ルールのサポート
2. **プラグインシステム**: 言語固有の lint ルール
3. **差分 lint**: 変更されたファイルのみ検査
4. **watch モード**: ファイル変更時の自動 lint
5. **LSP codeAction**: 自動修正の提案
