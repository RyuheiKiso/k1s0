# Lint 機能設計書

## 概要

k1s0 の Lint 機能は、開発規約に対する違反を検査し、一部は自動修正を提供します。

## ドキュメント構成

| ファイル | 内容 |
|----------|------|
| [ast-analysis.md](./ast-analysis.md) | AST ベース解析の設計（tree-sitter 統合） |
| [rules-manifest.md](./rules-manifest.md) | K001-K003 manifest 検査、K010-K011 必須ファイル検査 |
| [rules-code-quality.md](./rules-code-quality.md) | K020-K029 コード品質検査 |
| [rules-grpc-retry.md](./rules-grpc-retry.md) | K030-K032 gRPC リトライ設定検査 |
| [rules-layer-deps.md](./rules-layer-deps.md) | K040-K047 層間依存関係検査 |
| [rules-security.md](./rules-security.md) | K050-K053 セキュリティ検査 |
| [rules-infrastructure.md](./rules-infrastructure.md) | K060 インフラ検査 |
| [fixer.md](./fixer.md) | 自動修正 Fixer |
| [integration.md](./integration.md) | CLI 統合・LSP 統合 |
| [future.md](./future.md) | 今後の拡張予定 |

## モジュール構成

```
CLI/crates/k1s0-generator/src/lint/
├── mod.rs              # モジュール公開
├── types.rs            # 型定義（RuleId, Severity, Violation, LintResult, LintConfig）
├── linter.rs           # Linter 本体（manifest/必須ファイル検査）
├── required_files.rs   # 必須ディレクトリ/ファイル定義
├── ast/                # AST ベース解析（tree-sitter）
│   ├── mod.rs          # 公開 API
│   ├── parser.rs       # ParserPool（言語検出・パーサー管理）
│   ├── query.rs        # QueryCache（Query コンパイル・キャッシュ）
│   ├── context.rs      # AstContext（is_non_code, is_in_test, query_matches）
│   └── languages/      # 言語固有クエリ定義
│       ├── rust.rs
│       ├── go.rs
│       ├── typescript.rs
│       ├── python.rs
│       ├── csharp.rs
│       └── kotlin.rs
├── env_vars.rs         # K020: 環境変数参照検査（AST 対応）
├── secret_config.rs    # K021: 機密直書き検査
├── dependency.rs       # K022: 依存方向違反検査（AST 対応）
├── layer_dependency.rs # K040-K047: 層間依存関係検査
├── retry.rs            # K030-K032: gRPC リトライ設定検査
├── fixer.rs            # 自動修正ロジック
├── config_naming.rs    # K025: 設定ファイル命名規約検査
├── protocol_dependency.rs # K026: Domain 層プロトコル依存検査（AST 対応）
├── unused_domain.rs    # K028: 未使用 domain 依存検査
├── panic_detection.rs  # K029: 本番コードパニック検出（AST 対応）
├── sql_injection.rs    # K050: SQL インジェクションリスク検査（AST 対応）
├── sensitive_logging.rs # K053: 機密情報ログ出力検査（AST 対応）
├── dockerfile_lint.rs  # K060: Dockerfile ベースイメージ検査
├── diff.rs             # Git diff フィルタリング
├── watch.rs            # ファイル監視モード
├── utils.rs            # ユーティリティ関数
└── tests/              # テスト
```

**注記:** K020, K022, K026, K029, K050, K053 は AST ベース解析に対応しています。詳細は [ast-analysis.md](./ast-analysis.md) を参照してください。

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
| K025 | Error | 設定ファイルの命名規約違反 | - |
| K026 | Error | Domain 層でのプロトコル型使用禁止 | - |
| K028 | Warning | 未使用 domain 依存の検出 | - |
| K029 | Error | 本番コードでのパニック使用禁止 | - |
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
| K050 | Error | SQL インジェクションリスク検出 | - |
| K053 | Warning | ログへの機密情報出力検出 | - |
| K060 | Warning | Dockerfile ベースイメージ未固定 | - |

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
    ConfigFileNaming,        // K025
    ProtocolDependencyInDomain, // K026
    UnusedDomainDependency,  // K028
    PanicInProductionCode,   // K029
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
    // セキュリティルール（K050-K053）
    SqlInjectionRisk,            // K050
    LoggingSensitiveData,        // K053
    // インフラルール（K060）
    DockerfileBaseImageUnpinned, // K060
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
6. K025: 設定ファイル命名規約検査
7. K026: Domain 層プロトコル依存検査
8. K028: 未使用 domain 依存検査
9. K029: 本番コードパニック検出
10. K030/K031/K032: gRPC リトライ設定検査
11. K050: SQL インジェクションリスク検査
12. K053: 機密情報ログ出力検査
13. K060: Dockerfile ベースイメージ検査
14. strict モードの場合: 警告をエラーに昇格
```
