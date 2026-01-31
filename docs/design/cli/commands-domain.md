# ドメインコマンド

← [CLI 設計書](./)

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

    /// ドメインイベント雛形を含める
    #[arg(long)]
    pub with_events: bool,

    /// リポジトリ trait/interface を含める（デフォルト: true）
    #[arg(long, default_value = "true")]
    pub with_repository: bool,

    /// 初期バージョン
    #[arg(long, default_value = "0.1.0")]
    pub version: String,
}
```

### ドメインタイプ

| タイプ | テンプレートパス | 出力先 | 言語 |
|--------|----------------|-------|------|
| `backend-rust` | `CLI/templates/backend-rust/domain` | `domain/backend/rust/{name}` | rust |
| `backend-go` | `CLI/templates/backend-go/domain` | `domain/backend/go/{name}` | go |
| `backend-csharp` | `CLI/templates/backend-csharp/domain` | `domain/backend/csharp/{name}` | csharp |
| `backend-python` | `CLI/templates/backend-python/domain` | `domain/backend/python/{name}` | python |
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
| `domain_name` | ドメイン名（kebab-case） | `manufacturing` |
| `domain_name_snake` | snake_case 変換 | `manufacturing` |
| `domain_name_pascal` | PascalCase 変換 | `Production` |
| `language` | 言語 | `rust` |
| `service_type` | タイプ | `backend` |
| `k1s0_version` | k1s0 バージョン | `0.1.0` |
| `with_events` | イベント有効 | `true` |
| `with_repository` | リポジトリ有効 | `true` |
| `version` | 初期バージョン | `0.1.0` |

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
    "service_name": "manufacturing",
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
k1s0 new-domain --type backend-rust --name manufacturing

# カスタム出力先
k1s0 new-domain --type backend-rust --name manufacturing --output ./my-domains

# 上書き
k1s0 new-domain --type backend-rust --name manufacturing --force
```

---

## domain 管理コマンド

### 目的

domain の管理（一覧表示、バージョン管理、依存関係分析）を行う。

### サブコマンド

#### domain-list

```bash
k1s0 domain-list

# 出力例
Domains:
  manufacturing          0.1.0    domain/backend/rust/manufacturing
  inventory           1.2.0    domain/backend/rust/inventory
  user-management     2.0.0    domain/backend/go/user-management
```

#### domain-version

```bash
# バージョン確認
k1s0 domain-version --name manufacturing

# バージョン更新
k1s0 domain-version --name manufacturing --bump patch
k1s0 domain-version --name manufacturing --bump minor
k1s0 domain-version --name manufacturing --bump major

# 直接指定
k1s0 domain-version --name manufacturing --set 2.0.0

# 破壊的変更を記録
k1s0 domain-version --name manufacturing --bump major \
  --message "WorkOrder.quantity の型を変更"
```

#### domain-dependents

```bash
k1s0 domain-dependents --name manufacturing

# 出力例
Features depending on 'manufacturing':
  work-order-api          ^1.2.0    feature/backend/rust/work-order-api
  work-order-dashboard    ^1.5.0    feature/frontend/react/work-order-dashboard
  manufacturing-report       ^1.0.0    feature/backend/rust/manufacturing-report
```

#### domain-impact

```bash
k1s0 domain-impact --name manufacturing --from 1.5.0 --to 2.0.0

# 出力例
Domain: manufacturing
Version change: 1.5.0 -> 2.0.0 (MAJOR)

Breaking changes:
  - 2.0.0: WorkOrder.quantity の型を u32 から Quantity 値オブジェクトに変更

Affected features (3):
  - work-order-api (constraint: ^1.2.0) - INCOMPATIBLE
  - work-order-dashboard (constraint: ^1.5.0) - INCOMPATIBLE
  - manufacturing-report (constraint: ^1.0.0) - INCOMPATIBLE
```

---

## domain-catalog コマンド

### 目的

ドメインの一覧をカタログ形式で表示する。依存関係の状況も含む。

### 引数

```rust
pub struct DomainCatalogArgs {
    /// 言語でフィルタ（rust, go, typescript, dart）
    #[arg(long)]
    pub language: Option<String>,

    /// 非推奨ドメインも含める
    #[arg(long)]
    pub include_deprecated: bool,

    /// JSON 形式で出力
    #[arg(long)]
    pub json: bool,
}
```

---

## domain-graph コマンド

### 目的

ドメイン間の依存関係をグラフとして可視化する。

### 引数

```rust
pub struct DomainGraphArgs {
    /// 出力フォーマット
    #[arg(long, value_enum, default_value = "mermaid")]
    pub format: GraphFormat,

    /// ルートドメイン（指定した場合そのドメインを起点にしたサブグラフを出力）
    #[arg(long)]
    pub root: Option<String>,

    /// 循環依存を検出する
    #[arg(long)]
    pub detect_cycles: bool,
}

#[derive(Clone, ValueEnum)]
pub enum GraphFormat {
    /// Mermaid 形式
    Mermaid,
    /// Graphviz DOT 形式
    Dot,
}
```
