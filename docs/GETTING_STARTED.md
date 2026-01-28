# k1s0 入門ガイド

新規参入者向けの k1s0 プロジェクト使用方法ガイドです。

---

## 目次

1. [k1s0 とは](#k1s0-とは)
2. [環境セットアップ](#環境セットアップ)
3. [CLI の基本操作](#cli-の基本操作)
4. [新規サービスの作成](#新規サービスの作成)
5. [規約チェック（lint）](#規約チェックlint)
6. [守るべき重要ルール](#守るべき重要ルール)
7. [プロジェクト構造の理解](#プロジェクト構造の理解)
8. [よくある質問](#よくある質問)

---

## k1s0 とは

k1s0 は「高速な開発サイクルを実現する」ための統合開発基盤です。

### 3 つの主要コンポーネント

| コンポーネント | 役割 |
|--------------|------|
| **CLI** | サービス生成・規約チェック・アップグレード支援 |
| **Framework** | 共通ライブラリ・共通サービス |
| **Templates** | 新規サービス用の雛形 |

### k1s0 でできること

- **サービスの雛形を自動生成** → 手作業による構成ミスを防止
- **規約の自動チェック** → コードレビュー前に問題を検出
- **安全なアップグレード** → テンプレート更新時の衝突を管理

---

## 環境セットアップ

### 必要なツール

```
Rust        1.85 以上
pnpm        9.15.4 以上（フロントエンド開発時）
PostgreSQL  （バックエンド開発時）
```

### CLI のビルド

```bash
# リポジトリをクローン
git clone <repository-url>
cd k1s0

# CLI をビルド
cd CLI
cargo build --release

# パスを通す（任意）
# Windows: CLI\target\release\k1s0.exe
# Linux/Mac: CLI/target/release/k1s0
```

### 動作確認

```bash
k1s0 --version
# => k1s0 0.1.0
```

---

## CLI の基本操作

### コマンド一覧

| コマンド | 説明 |
|---------|------|
| `k1s0 init` | リポジトリを初期化 |
| `k1s0 new-feature` | 新規サービスの雛形を生成 |
| `k1s0 new-screen` | 画面（フロントエンド）の雛形を生成 |
| `k1s0 lint` | 規約違反を検査 |
| `k1s0 upgrade` | テンプレートの更新を確認・適用 |
| `k1s0 completions` | シェル補完スクリプトを生成 |

### 共通オプション

```bash
-v, --verbose    # 詳細出力
--no-color       # カラー出力を無効化
--json           # JSON 形式で出力
```

### ヘルプの見方

```bash
# 全体のヘルプ
k1s0 --help

# コマンド別のヘルプ
k1s0 new-feature --help
```

---

## 新規サービスの作成

### Step 1: サービスタイプを選ぶ

| タイプ | 説明 | 出力先 |
|-------|------|--------|
| `backend-rust` | Rust バックエンド | `feature/backend/rust/{name}/` |
| `backend-go` | Go バックエンド | `feature/backend/go/{name}/` |
| `frontend-react` | React フロントエンド | `feature/frontend/react/{name}/` |
| `frontend-flutter` | Flutter フロントエンド | `feature/frontend/flutter/{name}/` |

### Step 2: コマンドを実行

```bash
# 例: user-management という Rust バックエンドサービスを作成
k1s0 new-feature --type backend-rust --name user-management
```

### オプション

```bash
--with-grpc    # gRPC API を含める
--with-rest    # REST API を含める
--with-db      # DB マイグレーションを含める
--force        # 既存ディレクトリを上書き
--output DIR   # 出力先を指定
```

### 実行例

```bash
$ k1s0 new-feature --type backend-rust --name order-service --with-grpc --with-db

k1s0 new-feature

• type: backend-rust
• name: order-service
• output: feature/backend/rust/order-service
• with_grpc: true
• with_rest: false
• with_db: true

テンプレート: CLI/templates/backend-rust/feature
• fingerprint: a1b2c3d4e5f6...

テンプレートを展開中...

+ feature/backend/rust/order-service/Cargo.toml
+ feature/backend/rust/order-service/src/main.rs
+ feature/backend/rust/order-service/src/domain/mod.rs
+ ...
+ .k1s0/manifest.json

✓ サービス 'order-service' を生成しました

次のステップ:
→ cd feature/backend/rust/order-service
→ k1s0 lint でサービスの規約準拠を確認
```

### Step 3: 生成後の確認

```bash
# 生成されたディレクトリに移動
cd feature/backend/rust/order-service

# 規約チェック
k1s0 lint
```

### Step 4: 生成されたファイルの理解

生成直後のディレクトリ構造を確認します。

```
order-service/
├── .k1s0/
│   └── manifest.json          # k1s0 管理メタデータ（編集禁止）
├── Cargo.toml                 # 依存関係定義
├── config/
│   ├── default.yaml           # 共通設定
│   ├── dev.yaml               # 開発環境設定
│   └── prod.yaml              # 本番環境設定
├── deploy/
│   ├── kubernetes/            # K8s マニフェスト
│   └── docker/                # Dockerfile
├── migrations/                # DB マイグレーション（--with-db 時）
├── proto/                     # Protocol Buffers（--with-grpc 時）
└── src/
    ├── main.rs                # エントリーポイント
    ├── domain/                # ビジネスロジック層
    │   ├── mod.rs
    │   ├── entities/          # エンティティ定義
    │   ├── value_objects/     # 値オブジェクト
    │   ├── repositories/      # リポジトリトレイト
    │   └── services/          # ドメインサービス
    ├── application/           # アプリケーション層
    │   ├── mod.rs
    │   ├── usecases/          # ユースケース実装
    │   ├── services/          # アプリケーションサービス
    │   └── dtos/              # データ転送オブジェクト
    ├── infrastructure/        # インフラストラクチャ層
    │   ├── mod.rs
    │   ├── repositories/      # リポジトリ実装
    │   └── external/          # 外部サービス連携
    └── presentation/          # プレゼンテーション層
        ├── mod.rs
        ├── grpc/              # gRPC ハンドラ（--with-grpc 時）
        └── rest/              # REST ハンドラ（--with-rest 時）
```

### Step 5: 最初の機能を実装する

**1. ドメインエンティティを定義:**

```rust
// src/domain/entities/order.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: OrderId,
    pub customer_id: CustomerId,
    pub items: Vec<OrderItem>,
    pub status: OrderStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    Pending,
    Confirmed,
    Shipped,
    Delivered,
    Cancelled,
}
```

**2. リポジトリトレイトを定義:**

```rust
// src/domain/repositories/order_repository.rs
use async_trait::async_trait;
use crate::domain::entities::Order;

#[async_trait]
pub trait OrderRepository: Send + Sync {
    async fn find_by_id(&self, id: &OrderId) -> Result<Option<Order>, DomainError>;
    async fn save(&self, order: &Order) -> Result<Order, DomainError>;
    async fn delete(&self, id: &OrderId) -> Result<bool, DomainError>;
}
```

**3. ユースケースを実装:**

```rust
// src/application/usecases/create_order.rs
use crate::domain::{Order, OrderRepository};

pub struct CreateOrderUseCase {
    order_repository: Arc<dyn OrderRepository>,
}

impl CreateOrderUseCase {
    pub async fn execute(&self, input: CreateOrderInput) -> Result<Order, AppError> {
        let order = Order::new(input.customer_id, input.items)?;
        self.order_repository.save(&order).await
    }
}
```

**4. ビルドして確認:**

```bash
# ビルド
cargo build

# テスト
cargo test

# 規約チェック
k1s0 lint
```

### Step 6: 設定ファイルをカスタマイズ

```yaml
# config/dev.yaml
server:
  host: "0.0.0.0"
  port: 8080

database:
  host: "localhost"
  port: 5432
  database: "order_service_dev"
  username: "dev_user"
  password_file: "/run/secrets/db_password"  # 機密情報は _file サフィックス

grpc:
  port: 50051

observability:
  log_level: "debug"
  tracing_enabled: true
```

### Step 7: ローカルで実行

```bash
# 環境を明示的に指定して実行
cargo run -- --env dev

# または設定ファイルを直接指定
cargo run -- --config config/dev.yaml
```

### サービス名のルール

サービス名は **kebab-case** で指定します。

```
✓ 有効な例
  - user-management
  - order-service
  - auth-api

✗ 無効な例
  - UserManagement  （CamelCase は不可）
  - user_management （snake_case は不可）
  - -user-service   （先頭ハイフン不可）
  - user-           （末尾ハイフン不可）
```

---

## 規約チェック（lint）

### 基本の使い方

```bash
# カレントディレクトリを検査
k1s0 lint

# 特定のディレクトリを検査
k1s0 lint feature/backend/rust/order-service
```

### 検出されるルール

| ルール ID | 説明 | 深刻度 |
|-----------|------|--------|
| K001 | manifest.json が存在しない | Error |
| K002 | manifest.json の必須キーが不足 | Error |
| K003 | manifest.json の値が不正 | Error |
| K010 | 必須ディレクトリが存在しない | Error |
| K011 | 必須ファイルが存在しない | Error |
| K020 | 環境変数参照の禁止 | Error |
| K021 | config YAML への機密直書き禁止 | Error |
| K022 | Clean Architecture 依存方向違反 | Error |
| K030 | gRPC リトライ設定の検出 | Warning |
| K031 | gRPC リトライ設定に ADR 参照がない | Warning |
| K032 | gRPC リトライ設定が不完全 | Warning |

### オプション

```bash
--rules K001,K002      # 特定のルールのみ実行
--exclude-rules K030   # 特定のルールを除外
--strict               # 警告もエラーとして扱う
--fix                  # 自動修正を試みる
--json                 # JSON 形式で出力
```

### 自動修正

一部のルールは `--fix` オプションで自動修正できます。

```bash
k1s0 lint --fix
```

修正可能なルール: K001, K002, K010, K011

### 出力例

```bash
$ k1s0 lint

k1s0 lint

• path: feature/backend/rust/order-service

違反:

[K020] error: 環境変数参照が検出されました
  対象: src/config.rs:15
  ヒント: 環境変数ではなく設定ファイル（config/*.yaml）を使用してください

[K030] warn: gRPC リトライ設定が検出されました
  対象: src/infrastructure/grpc_client.rs:42
  ヒント: docs/adr/ADR-xxx.md に設定理由を記録してください

エラー: 1, 警告: 1
検査失敗
```

---

## 守るべき重要ルール

### 1. 環境変数は使わない

```rust
// ✗ 禁止
let db_url = std::env::var("DATABASE_URL")?;

// ✓ 正しい方法
let config = Config::load("config/app.yaml")?;
let db_url = config.database.url;
```

**理由**: 環境変数は追跡が困難。設定ファイルで一元管理する。

### 2. 機密情報は直書きしない

```yaml
# ✗ 禁止（config/app.yaml）
database:
  password: "actual-password-here"

# ✓ 正しい方法
database:
  password_file: "/run/secrets/db_password"
```

**理由**: Git にコミットされるリスクを防ぐ。

### 3. 依存方向を守る

```
許可される依存方向:
  feature → framework     ✓
  presentation → application → domain ✓

禁止される依存方向:
  framework → feature     ✗
  domain → infrastructure ✗
```

### 4. Clean Architecture に従う

```
src/
├── domain/         # ビジネスロジック（外部依存なし）
├── application/    # ユースケース（domain を使う）
├── presentation/   # API/UI 層（application を使う）
└── infrastructure/ # 外部接続（domain の trait を実装）
```

---

## プロジェクト構造の理解

### 全体構造

```
k1s0/
├── CLI/                    # あなたが使う CLI ツール
│   ├── crates/             # CLI のソースコード
│   └── templates/          # サービス生成用テンプレート
│
├── framework/              # 共通部品（import して使う）
│   ├── backend/rust/crates/    # Rust 共通ライブラリ
│   ├── backend/rust/services/  # 共通マイクロサービス
│   └── frontend/react/packages/ # React 共通パッケージ
│
├── feature/                # ★ あなたが開発する場所
│   ├── backend/rust/       # Rust サービス群
│   ├── backend/go/         # Go サービス群
│   ├── frontend/react/     # React アプリ群
│   └── database/           # テーブル定義
│
├── bff/                    # Backend For Frontend（任意）
│
└── docs/                   # ドキュメント
    ├── adr/                # アーキテクチャ決定記録
    └── conventions/        # 規約
```

### よく使うパス

| 目的 | パス |
|-----|------|
| 新規サービスを作る | `feature/backend/rust/` または `feature/frontend/react/` |
| 共通ライブラリを使う | `framework/backend/rust/crates/` |
| 規約を確認する | `docs/conventions/` |
| アーキテクチャ決定を読む | `docs/adr/` |

### 共通ライブラリ（Rust）

| ライブラリ | 用途 |
|-----------|------|
| `k1s0-config` | 設定読み込み |
| `k1s0-error` | エラー型の統一 |
| `k1s0-observability` | ログ/トレース/メトリクス |
| `k1s0-grpc-client` | gRPC クライアント |
| `k1s0-grpc-server` | gRPC サーバー |
| `k1s0-db` | データベース接続 |
| `k1s0-auth` | 認証 |
| `k1s0-health` | ヘルスチェック |
| `k1s0-resilience` | リトライ・サーキットブレーカー |
| `k1s0-validation` | バリデーション |

### 共通パッケージ（React）

| パッケージ | 用途 |
|-----------|------|
| `@k1s0/api-client` | API クライアント |
| `@k1s0/navigation` | ルーティング |
| `@k1s0/ui` | UI コンポーネント（MUI ベース） |
| `@k1s0/config` | 設定管理 |
| `@k1s0/shell` | シェル基盤 |

---

## よくある質問

### Q: どこからコマンドを実行すればいい？

**A**: k1s0 リポジトリのルートディレクトリから実行してください。CLI は親ディレクトリを辿ってテンプレートを探しますが、ルートから実行するのが確実です。

```bash
cd /path/to/k1s0
k1s0 new-feature --type backend-rust --name my-service
```

### Q: lint エラーが出たらどうすればいい？

**A**: まず `--fix` オプションを試してください。自動修正できないものは、エラーメッセージの「ヒント」に従って手動で修正します。

```bash
k1s0 lint --fix
```

### Q: 既存のサービスを上書きしたい

**A**: `--force` オプションを使います。ただし、既存のコードは完全に削除されるので注意してください。

```bash
k1s0 new-feature --type backend-rust --name my-service --force
```

### Q: テンプレートが更新されたら？

**A**: `k1s0 upgrade` コマンドで更新を確認・適用できます。

```bash
# 更新の確認のみ
k1s0 upgrade --check

# 更新を適用
k1s0 upgrade
```

### Q: manifest.json とは？

**A**: 各サービスの `.k1s0/manifest.json` は、CLI が管理するメタデータファイルです。テンプレートバージョン、フィンガープリント、更新ポリシーなどが記録されています。**手動で編集しないでください**。

### Q: 規約ドキュメントはどこ？

**A**: `docs/conventions/` にあります。

| ドキュメント | 内容 |
|-------------|------|
| service-structure.md | サービスの構成規則 |
| config-and-secrets.md | 設定と機密情報の扱い |
| api-contracts.md | API 契約管理 |
| observability.md | ログ/トレース/メトリクス |
| error-handling.md | エラーハンドリング |
| versioning.md | バージョニング |

---

## 次のステップ

1. **環境をセットアップする** → [環境セットアップ](#環境セットアップ)
2. **最初のサービスを作る** → [新規サービスの作成](#新規サービスの作成)
3. **規約を読む** → `docs/conventions/` のドキュメント
4. **共通ライブラリを理解する** → `framework/` のソースコード

---

## 困ったときは

- **規約の詳細** → `docs/conventions/` を確認
- **アーキテクチャの決定理由** → `docs/adr/` を確認
- **CLI のヘルプ** → `k1s0 --help` または `k1s0 <command> --help`
