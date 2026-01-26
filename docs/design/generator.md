# Generator 設計書

## 概要

k1s0-generator は、テンプレート展開・差分適用を行うライブラリです。CLI の `new-feature`、`lint`、`upgrade` コマンドの基盤となります。

## モジュール構成

```
CLI/crates/k1s0-generator/src/
├── lib.rs          # エラー型定義、モジュール公開
├── manifest.rs     # manifest.json の読み書き・バリデーション
├── template.rs     # Tera テンプレートレンダラー
├── fingerprint.rs  # テンプレートの SHA256 ハッシュ計算
├── diff.rs         # ファイル差分の計算・表示
├── fs.rs           # ファイル操作ユーティリティ
├── walker.rs       # ディレクトリ再帰走査
├── upgrade.rs      # テンプレート更新ロジック
└── lint/           # 規約検査エンジン（別設計書参照）
```

---

## エラー型

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// IO エラー
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON パースエラー
    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),

    /// YAML パースエラー
    #[error("YAML parse error: {0}")]
    YamlParse(#[from] serde_yaml::Error),

    /// テンプレートエラー
    #[error("Template error: {0}")]
    Template(#[from] tera::Error),

    /// manifest が見つからない
    #[error("Manifest not found: {0}")]
    ManifestNotFound(String),

    /// manifest のバリデーションエラー
    #[error("Manifest validation error: {0}")]
    ManifestValidation(String),

    /// テンプレートが見つからない
    #[error("Template not found: {0}")]
    TemplateNotFound(String),

    /// ファイルの衝突
    #[error("File conflict: {0}")]
    FileConflict(String),

    /// その他のエラー
    #[error("{0}")]
    Other(String),
}
```

---

## manifest モジュール

### 目的

`.k1s0/manifest.json` の読み込み・書き込み・バリデーションを提供する。

### Manifest 構造

```rust
pub struct Manifest {
    /// スキーマバージョン
    pub schema_version: String,  // "1.0.0"

    /// k1s0 バージョン
    pub k1s0_version: String,

    /// テンプレート情報
    pub template: TemplateInfo,

    /// サービス情報
    pub service: ServiceInfo,

    /// 生成日時（RFC 3339）
    pub generated_at: String,

    /// CLI が管理するパス
    pub managed_paths: Vec<String>,

    /// CLI が変更しないパス
    pub protected_paths: Vec<String>,

    /// パス別の更新ポリシー
    pub update_policy: HashMap<String, UpdatePolicy>,

    /// ファイルのチェックサム
    pub checksums: HashMap<String, String>,

    /// framework crate への依存情報
    pub dependencies: Option<Dependencies>,
}
```

### TemplateInfo

```rust
pub struct TemplateInfo {
    /// テンプレート名（例: "backend-rust"）
    pub name: String,

    /// テンプレートバージョン
    pub version: String,

    /// ソース（"local" / registry URL）
    pub source: String,

    /// テンプレートのパス
    pub path: String,

    /// Git リビジョン（省略可能）
    pub revision: Option<String>,

    /// fingerprint（SHA256 ハッシュ）
    pub fingerprint: String,
}
```

### ServiceInfo

```rust
pub struct ServiceInfo {
    /// サービス名（kebab-case）
    pub service_name: String,

    /// 言語（rust, go, typescript, dart）
    pub language: String,

    /// タイプ（backend, frontend, bff）
    pub service_type: String,

    /// フレームワーク（省略可能）
    pub framework: Option<String>,
}
```

### UpdatePolicy

```rust
pub enum UpdatePolicy {
    /// 自動更新
    Auto,
    /// 差分提示のみ
    SuggestOnly,
    /// 変更しない
    Protected,
}
```

### API

```rust
impl Manifest {
    /// manifest.json を読み込む
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self>;

    /// manifest.json を書き込む
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()>;

    /// バリデーションを実行する
    pub fn validate(&self) -> Result<()>;
}
```

### manifest.json 例

```json
{
  "schema_version": "1.0.0",
  "k1s0_version": "0.1.0",
  "template": {
    "name": "backend-rust",
    "version": "0.1.0",
    "source": "local",
    "path": "CLI/templates/backend-rust/feature",
    "fingerprint": "abcd1234..."
  },
  "service": {
    "service_name": "user-management",
    "language": "rust",
    "type": "backend"
  },
  "generated_at": "2026-01-27T10:00:00Z",
  "managed_paths": ["deploy/", "buf.yaml", "buf.gen.yaml"],
  "protected_paths": ["src/domain/", "src/application/", "README.md"],
  "update_policy": {
    "deploy/": "auto",
    "src/domain/": "protected",
    "README.md": "suggest_only"
  },
  "checksums": {}
}
```

---

## template モジュール

### 目的

Tera テンプレートエンジンを使用したテンプレート展開を提供する。

### TemplateRenderer

```rust
pub struct TemplateRenderer {
    /// Tera テンプレートエンジン
    tera: Tera,
    /// テンプレートディレクトリ
    template_dir: PathBuf,
}
```

### RenderResult

```rust
pub struct RenderResult {
    /// 生成されたファイル
    pub created_files: Vec<String>,
    /// スキップされたファイル（既に同一内容）
    pub skipped_files: Vec<String>,
    /// 上書きされたファイル
    pub overwritten_files: Vec<String>,
}
```

### API

```rust
impl TemplateRenderer {
    /// 新しいレンダラーを作成する
    pub fn new<P: AsRef<Path>>(template_dir: P) -> Result<Self>;

    /// テンプレートをレンダリングする
    pub fn render(&self, template_name: &str, context: &Context) -> Result<String>;

    /// テンプレートディレクトリを展開する
    pub fn render_directory<P: AsRef<Path>>(
        &self,
        output_dir: P,
        context: &Context,
    ) -> Result<RenderResult>;

    /// 利用可能なテンプレート一覧を取得
    pub fn list_templates(&self) -> Vec<String>;
}
```

### 処理ルール

1. **`.tera` 拡張子のファイル**: Tera でレンダリング後、拡張子を除去して出力
2. **その他のファイル**: そのままコピー
3. **ディレクトリ構造**: 維持

---

## fingerprint モジュール

### 目的

テンプレートディレクトリから決定論的な SHA-256 ハッシュを算出する。詳細は ADR-0003 を参照。

### API

```rust
/// ディレクトリの fingerprint を算出する
pub fn calculate_fingerprint<P: AsRef<Path>>(dir: P) -> Result<String>;

/// ファイルの checksum を算出する
pub fn calculate_file_checksum<P: AsRef<Path>>(path: P) -> Result<String>;
```

### アルゴリズム

1. ディレクトリを再帰的に走査
2. 除外パターンに一致するファイルをスキップ
3. ファイルを相対パスでソート（決定論的な順序）
4. パス区切りを `/` に正規化（OS 非依存）
5. 各ファイルについて:
   - パスをハッシュに追加
   - ファイル内容をハッシュに追加
6. 最終的な SHA-256 ハッシュを hex エンコード

### 除外パターン

```rust
// ディレクトリ/ファイル名パターン
const EXCLUDE_PATTERNS: &[&str] = &[
    ".git", ".svn", ".hg",           // バージョン管理
    "target", "node_modules", "dist", "build", "__pycache__", ".dart_tool",  // ビルド成果物
    ".DS_Store", "Thumbs.db", "Desktop.ini",  // OS メタデータ
    ".idea", ".vscode",              // IDE/エディタ
    ".k1s0",                         // k1s0 メタデータ
];

// 除外する拡張子
const EXCLUDE_EXTENSIONS: &[&str] = &[".pyc", ".pyo", ".log", ".tmp", ".bak", ".swp", ".swo"];

// 除外するファイル名（完全一致）
const EXCLUDE_FILES: &[&str] = &[".env", ".env.local"];
```

---

## upgrade モジュール

### 目的

テンプレートの更新チェックと適用を提供する。

### VersionChange

```rust
pub enum VersionChange {
    /// メジャーバージョン更新 (1.x.x -> 2.x.x)
    Major,
    /// マイナーバージョン更新 (x.1.x -> x.2.x)
    Minor,
    /// パッチバージョン更新 (x.x.1 -> x.x.2)
    Patch,
    /// 変更なし
    None,
}
```

### UpgradeCheckResult

```rust
pub struct UpgradeCheckResult {
    /// 現在のテンプレートバージョン
    pub current_version: String,
    /// 新しいテンプレートバージョン
    pub new_version: String,
    /// バージョン変更の種類
    pub version_change: VersionChange,
    /// 現在の fingerprint
    pub current_fingerprint: String,
    /// 新しい fingerprint
    pub new_fingerprint: String,
    /// 差分結果
    pub diff: DiffResult,
    /// managed 領域の差分
    pub managed_diff: DiffResult,
    /// protected 領域の差分
    pub protected_diff: DiffResult,
    /// MAJOR 変更時の ADR ファイル存在
    pub has_upgrade_adr: bool,
    /// UPGRADE.md の存在
    pub has_upgrade_md: bool,
    /// 更新が必要かどうか
    pub needs_upgrade: bool,
    /// 衝突があるかどうか
    pub has_conflicts: bool,
}
```

### UpgradeApplyResult

```rust
pub struct UpgradeApplyResult {
    /// 適用されたファイル
    pub applied: Vec<String>,
    /// スキップされたファイル（protected）
    pub skipped: Vec<String>,
    /// バックアップされたファイル
    pub backed_up: Vec<String>,
    /// 衝突したファイル
    pub conflicts: Vec<String>,
}
```

### API

```rust
/// アップグレードチェックを実行する
pub fn check_upgrade<P: AsRef<Path>>(
    service_path: P,
    template_path: Option<&Path>,
) -> Result<UpgradeCheckResult>;

/// アップグレードを適用する
pub fn apply_upgrade<P: AsRef<Path>>(
    service_path: P,
    check_result: &UpgradeCheckResult,
    managed_only: bool,
    create_backup: bool,
) -> Result<UpgradeApplyResult>;

/// 保留中のマイグレーションを一覧表示
pub fn list_pending_migrations<P: AsRef<Path>>(
    service_path: P,
) -> Result<Vec<MigrationFile>>;

/// マイグレーションを適用する
pub fn apply_migrations<P: AsRef<Path>>(
    service_path: P,
    env: &str,
    dry_run: bool,
) -> Result<MigrationResult>;
```

### 処理フロー

#### check_upgrade

```
1. manifest.json 読み込み
2. テンプレートパス決定
3. 新 fingerprint 計算
4. fingerprint 比較
   └─ 同一の場合: 更新不要
5. 差分計算（衝突検知含む）
6. managed/protected に分類
7. ADR/UPGRADE.md 確認
8. UpgradeCheckResult 返却
```

#### apply_upgrade

```
1. 衝突がある場合: エラー
2. manifest 読み込み
3. managed 領域の変更を適用
   ├─ バックアップ作成（オプション）
   └─ ファイルコピー/削除
4. manifest.json 更新
   ├─ fingerprint 更新
   ├─ version 更新
   └─ checksums 更新
5. UpgradeApplyResult 返却
```

---

## diff モジュール

### 目的

ファイル差分の計算と表示を提供する。

### DiffKind

```rust
pub enum DiffKind {
    /// 追加
    Added,
    /// 削除
    Removed,
    /// 変更
    Modified,
    /// 変更なし
    Unchanged,
}
```

### FileDiff

```rust
pub struct FileDiff {
    /// ファイルパス（相対）
    pub path: String,
    /// 差分の種類
    pub kind: DiffKind,
    /// 期待されるチェックサム（衝突検知用）
    pub expected_checksum: Option<String>,
    /// 実際のチェックサム
    pub actual_checksum: Option<String>,
}
```

### DiffResult

```rust
pub struct DiffResult {
    /// 追加されたファイル
    pub added: Vec<FileDiff>,
    /// 削除されたファイル
    pub removed: Vec<FileDiff>,
    /// 変更されたファイル
    pub modified: Vec<FileDiff>,
    /// 衝突したファイル
    pub conflicts: Vec<FileDiff>,
    /// 変更なしのファイル
    pub unchanged: Vec<FileDiff>,
}

impl DiffResult {
    /// 変更があるかどうか
    pub fn has_changes(&self) -> bool;

    /// 衝突があるかどうか
    pub fn has_conflicts(&self) -> bool;

    /// すべての変更を取得
    pub fn all_changes(&self) -> Vec<&FileDiff>;

    /// サマリーを取得
    pub fn summary(&self) -> String;
}
```

---

## fs モジュール

### 目的

ファイル操作のユーティリティを提供する。

### WriteResult

```rust
pub enum WriteResult {
    /// 新規作成
    Created,
    /// スキップ（同一内容）
    Skipped,
    /// 上書き
    Overwritten,
}
```

### API

```rust
/// ファイルを書き込む（親ディレクトリを自動作成）
pub fn write_file<P: AsRef<Path>>(path: P, content: &str) -> Result<WriteResult>;

/// ファイルを読み込む
pub fn read_file<P: AsRef<Path>>(path: P) -> Result<String>;

/// ディレクトリを作成
pub fn create_dir_all<P: AsRef<Path>>(path: P) -> Result<()>;
```

---

## 依存ライブラリ

| ライブラリ | バージョン | 用途 |
|-----------|----------|------|
| tera | 1.19 | テンプレートエンジン |
| serde | 1.0 | シリアライゼーション |
| serde_json | 1.0 | JSON 処理 |
| serde_yaml | 0.9 | YAML 処理 |
| sha2 | 0.10 | SHA-256 ハッシュ |
| walkdir | 2.5 | ディレクトリ走査 |
| glob | 0.3 | ファイルパターンマッチング |
| chrono | 0.4 | 日時操作 |
| thiserror | 2.0 | エラー型定義 |
