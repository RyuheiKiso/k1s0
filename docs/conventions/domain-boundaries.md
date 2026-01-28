# domain 境界の判断基準

このドキュメントでは、k1s0 の3層アーキテクチャにおける domain 層の境界を判断するための基準を説明します。

## 1. 概要

domain 層は、framework 層と feature 層の間に位置し、特定の業務領域に共通するビジネスロジックを管理します。適切な境界を定義することで、コードの再利用性と保守性を向上させます。

```
framework (技術基盤) -> domain (業務領域共通) -> feature (個別機能)
```

---

## 2. domain に含めるべきもの

### 2.1 エンティティ（Entities）

**含める条件**:
- 複数の feature で参照される
- 一意の識別子を持つ
- ビジネス上の重要な概念

**例**:
```rust
// manufacturing domain
pub struct WorkOrder { ... }
pub struct ProductionLine { ... }
pub struct Schedule { ... }

// inventory domain
pub struct Stock { ... }
pub struct Location { ... }
pub struct Movement { ... }
```

### 2.2 値オブジェクト（Value Objects）

**含める条件**:
- 複数の feature で使用される
- ビジネスルール（バリデーション）を含む
- 不変性を保証する必要がある

**例**:
```rust
pub struct Quantity { ... }       // 数量（0以上のバリデーション）
pub struct Money { ... }          // 金額（通貨付き）
pub struct DateRange { ... }      // 期間（開始<=終了のバリデーション）
pub struct EmailAddress { ... }   // メールアドレス（形式バリデーション）
```

### 2.3 ドメインサービス（Domain Services）

**含める条件**:
- 複数のエンティティにまたがるビジネスロジック
- 特定のエンティティに属さない計算ロジック
- 複数の feature で共有されるビジネスルール

**例**:
```rust
pub struct SchedulingService { ... }    // スケジューリング計算
pub struct PricingService { ... }       // 価格計算
pub struct AllocationService { ... }    // 在庫引当
```

### 2.4 リポジトリインターフェース（Repository Traits）

**含める条件**:
- エンティティの永続化抽象化として共有
- 複数の feature で同じデータアクセスパターン

**例**:
```rust
#[async_trait]
pub trait WorkOrderRepository: Send + Sync {
    async fn find_by_id(&self, id: &WorkOrderId) -> Result<Option<WorkOrder>, DomainError>;
    async fn save(&self, work_order: &WorkOrder) -> Result<(), DomainError>;
}
```

### 2.5 共通アプリケーションサービス

**含める条件**:
- 複数の feature で共有されるユースケース
- ビジネスルールの組み合わせロジック

**例**:
```rust
pub struct WorkOrderService<R: WorkOrderRepository> {
    // 作業指示の作成、開始、完了などの共通ユースケース
}
```

---

## 3. domain に含めないもの

### 3.1 プレゼンテーション層のコード

**理由**: feature 固有であり、API 設計は feature ごとに異なる可能性がある

**含めないもの**:
- REST/gRPC ハンドラー
- リクエスト/レスポンス DTO
- ミドルウェア
- シリアライゼーション定義

```rust
// これらは feature 層に配置
pub struct CreateWorkOrderRequest { ... }  // API リクエスト
pub struct WorkOrderResponse { ... }       // API レスポンス
pub async fn create_work_order(req: Request) -> Response { ... }  // ハンドラー
```

### 3.2 インフラストラクチャ層の実装

**理由**: データベースや外部サービスの詳細は feature 固有

**含めないもの**:
- リポジトリ実装（DB アクセス）
- 外部 API クライアント実装
- キャッシュ実装
- メッセージキュー実装

```rust
// これらは feature 層の infrastructure に配置
pub struct WorkOrderRepositoryImpl { pool: DbPool }
pub struct ExternalApiClient { client: HttpClient }
```

### 3.3 環境固有の設定

**理由**: 設定は feature ごとにカスタマイズが必要

**含めないもの**:
- config/*.yaml
- 環境変数マッピング
- 接続文字列

### 3.4 feature 固有のユースケース

**理由**: 特定の feature でのみ必要なロジック

**含めないもの**:
```rust
// work-order-api 固有のユースケース
pub async fn export_work_orders_to_csv(...) { ... }

// dashboard 固有のユースケース
pub async fn get_work_order_statistics(...) { ... }
```

---

## 4. framework と domain の境界

### 4.1 framework 層の責務

framework 層は **技術的な共通基盤** を提供します。

| framework | 責務 | 例 |
|-----------|------|-----|
| k1s0-error | エラーハンドリング基盤 | `K1s0Error`, `ErrorCode` |
| k1s0-config | 設定読み込み基盤 | `ConfigLoader`, `ConfigOptions` |
| k1s0-db | DB 接続基盤 | `DbPool`, `Transaction` |
| k1s0-validation | バリデーション基盤 | `Validator`, `ValidationRule` |
| k1s0-observability | ロギング/トレーシング | `Logger`, `Tracer` |

### 4.2 domain 層が framework に依存する例

```rust
// domain の Cargo.toml
[dependencies]
k1s0-error = { path = "..." }      # エラー型を使用
k1s0-validation = { path = "..." } # バリデーションを使用

// domain のエラー型
use k1s0_error::K1s0Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Validation failed: {0}")]
    Validation(String),
    ...
}

impl From<DomainError> for K1s0Error {
    fn from(err: DomainError) -> Self {
        // k1s0-error の型に変換
    }
}
```

### 4.3 境界の判断

```
質問: このコードはビジネス知識を含んでいるか？

含んでいない → framework 層
含んでいる  → domain 層または feature 層

例:
- JSON シリアライゼーション → framework（技術的）
- 「数量は 0 以上」 → domain（ビジネスルール）
- HTTP ハンドラー → feature（API 設計）
```

---

## 5. domain と feature の境界

### 5.1 feature 層の責務

feature 層は **具体的なユースケース実装** を担当します。

| feature | 責務 |
|---------|------|
| work-order-api | 作業指示の REST/gRPC API |
| work-order-dashboard | 作業指示のダッシュボード画面 |
| manufacturing-report | 生産レポート生成 |

### 5.2 境界の判断

```
質問: このコードは複数の feature で共有されるか？

共有される   → domain 層
共有されない → feature 層

追加質問: 将来的に共有される可能性は？

可能性高い → domain 層（早期切り出し）
可能性低い → feature 層（必要になったら移行）
```

### 5.3 実例

**domain に配置**:
```rust
// 複数の feature で使われる WorkOrder エンティティ
pub struct WorkOrder { ... }

// 複数の feature で使われる完了日計算
pub fn calculate_completion_date(...) { ... }
```

**feature に配置**:
```rust
// work-order-api 固有の REST ハンドラー
pub async fn create_work_order_handler(...) { ... }

// dashboard 固有の集計ロジック
pub fn aggregate_work_orders_by_status(...) { ... }

// report 固有の CSV エクスポート
pub fn export_to_csv(...) { ... }
```

---

## 6. 判断フローチャート

### 6.1 新規コードの配置

```
新しいコードを書く
    │
    ▼
ビジネス知識を含んでいるか？
    │
    ├─ いいえ → framework 層の検討
    │           │
    │           ▼
    │         汎用的な技術コンポーネントか？
    │           │
    │           ├─ はい → framework 層
    │           └─ いいえ → feature 層（infrastructure）
    │
    └─ はい → domain または feature の検討
              │
              ▼
            複数の feature で共有されるか？
              │
              ├─ はい → domain 層
              │
              └─ いいえ
                  │
                  ▼
                将来的に共有される可能性は？
                  │
                  ├─ 高い → domain 層（早期切り出し）
                  └─ 低い → feature 層
```

### 6.2 既存コードの移行判断

```
既存のコードを分析
    │
    ▼
同じコードが複数の feature に存在するか？
    │
    ├─ いいえ → 現状維持
    │
    └─ はい
        │
        ▼
      そのコードはエンティティ、値オブジェクト、
      ドメインサービスのいずれかか？
        │
        ├─ はい → domain 層に移行
        │
        └─ いいえ
            │
            ▼
          技術的なユーティリティか？
            │
            ├─ はい → framework 層の検討
            └─ いいえ → 共通モジュールとして feature 内で管理
```

---

## 7. 具体例での判断

### 7.1 例 1: バリデーションロジック

```rust
fn validate_quantity(value: u32) -> Result<(), ValidationError> {
    if value == 0 {
        return Err(ValidationError::ZeroQuantity);
    }
    Ok(())
}
```

**判断**:
- ビジネス知識を含む: はい（「数量は0以上」）
- 複数の feature で共有: はい（在庫管理、注文管理など）

**結論**: domain 層（値オブジェクト `Quantity` として）

### 7.2 例 2: 日付フォーマット

```rust
fn format_date(date: DateTime<Utc>) -> String {
    date.format("%Y-%m-%d").to_string()
}
```

**判断**:
- ビジネス知識を含む: いいえ（技術的なフォーマット）
- 汎用的: はい

**結論**:
- 既存の framework (k1s0-config) で提供するか
- feature 内のユーティリティとして配置

### 7.3 例 3: 在庫引当ロジック

```rust
pub struct AllocationService;

impl AllocationService {
    pub fn allocate(
        &self,
        order: &Order,
        available_stock: &Stock,
    ) -> Result<Allocation, AllocationError> {
        // 在庫引当のビジネスロジック
    }
}
```

**判断**:
- ビジネス知識を含む: はい（在庫引当ルール）
- 複数の feature で共有: はい（注文API、在庫管理UI）

**結論**: domain 層（inventory domain のドメインサービスとして）

### 7.4 例 4: API レスポンスのシリアライゼーション

```rust
#[derive(Serialize)]
pub struct WorkOrderResponse {
    pub id: String,
    pub status: String,
}

impl From<WorkOrder> for WorkOrderResponse {
    fn from(wo: WorkOrder) -> Self { ... }
}
```

**判断**:
- ビジネス知識を含む: いいえ（プレゼンテーション）
- API 固有: はい

**結論**: feature 層（presentation）

---

## 8. アンチパターン

### 8.1 過度に細かい domain

```
# 避けるべき
domain/
├── work-order/          # エンティティ1つだけ
├── quantity/            # 値オブジェクト1つだけ
└── scheduling-service/  # サービス1つだけ
```

**問題**: 管理オーバーヘッドが増大、依存関係が複雑化

**対策**: 関連する概念をまとめて 1 つの domain に

```
# 推奨
domain/
└── manufacturing/          # WorkOrder + Quantity + SchedulingService
```

### 8.2 巨大な domain

```
# 避けるべき
domain/
└── manufacturing/       # 生産 + 在庫 + 品質 + 物流 + ...
```

**問題**: 変更影響範囲が大きい、理解が困難

**対策**: Bounded Context に沿って分割

```
# 推奨
domain/
├── manufacturing/          # 生産管理
├── inventory/           # 在庫管理
├── quality/             # 品質管理
└── logistics/           # 物流管理
```

### 8.3 インフラストラクチャを含む domain

```rust
// 避けるべき: domain 内に DB アクセスコード
pub struct WorkOrderRepositoryImpl {
    pool: DbPool,  // インフラストラクチャ依存
}
```

**問題**: domain がインフラストラクチャに依存、テストが困難

**対策**: トレイトのみを domain に配置、実装は feature に

```rust
// domain: トレイトのみ
pub trait WorkOrderRepository { ... }

// feature: 実装
pub struct WorkOrderRepositoryImpl { pool: DbPool }
impl WorkOrderRepository for WorkOrderRepositoryImpl { ... }
```

---

## 9. 関連ドキュメント

- [domain 開発ガイド](../guides/domain-development.md)
- [3層構造への移行ガイド](../guides/migration-to-three-tier.md)
- [Clean Architecture](../architecture/clean-architecture.md)
- [ADR-0006: 3層アーキテクチャ](../adr/ADR-0006-three-layer-architecture.md)
