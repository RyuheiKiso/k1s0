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
| `new-screen` | 画面雛形生成 | `-t/--type`, `-n/--name` |
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

## 今後の拡張予定

1. **registry サポート**: リモートテンプレートレジストリからのテンプレート取得
2. **プラグインシステム**: カスタムコマンドの追加
3. **設定ファイル**: `.k1s0/settings.yaml` によるデフォルト設定
4. **watch モード**: ファイル変更時の自動 lint
