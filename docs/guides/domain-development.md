# domain 開発ガイド

このガイドでは、k1s0 の3層アーキテクチャにおける domain 層の開発方法について説明します。

## 1. domain 層の役割と責務

### 1.1 概要

domain 層は、k1s0 の3層アーキテクチャ（framework -> domain -> feature）における中間層です。特定の業務領域に共通するビジネスロジックを一元管理し、複数の feature から共有できるようにします。

```
framework (技術基盤) -> domain (業務領域共通) -> feature (個別機能)
```

### 1.2 責務の範囲

domain 層が担当するもの:

| 種類 | 説明 | 例 |
|------|------|-----|
| エンティティ | 一意の識別子を持つビジネスオブジェクト | WorkOrder, Product, Customer |
| 値オブジェクト | 不変のオブジェクト、属性で等価性を判断 | Quantity, Money, Address |
| ドメインサービス | エンティティに属さないビジネスロジック | SchedulingService, PricingService |
| リポジトリインターフェース | データアクセスの抽象化（実装は feature 層） | WorkOrderRepository trait |
| 共通アプリケーションサービス | 複数 feature で共有されるユースケース | CreateWorkOrderService |

domain 層が担当しないもの:

- プレゼンテーション層（REST/gRPC エンドポイント）
- 具体的なリポジトリ実装（DB アクセス）
- 環境固有の設定
- 外部サービスとの直接連携

### 1.3 framework 層との違い

| 観点 | framework 層 | domain 層 |
|------|------------|---------|
| 目的 | 技術的な共通基盤 | 業務ロジックの共有 |
| 例 | k1s0-error, k1s0-config | manufacturing, inventory |
| バージョン | k1s0 CLI と連動 | 独立した SemVer |
| 業務知識 | 含まない | 含む |

---

## 2. domain の作成方法

### 2.1 new-domain コマンド

```bash
# 基本的な使用法
k1s0 new-domain --type backend-rust --name manufacturing

# オプション一覧
k1s0 new-domain \
  --type backend-rust \      # テンプレートタイプ
  --name manufacturing \        # domain 名（kebab-case）
  --output ./domain \        # 出力先（デフォルト: domain/{type}/{name}）
  --force                    # 既存ディレクトリを上書き
```

### 2.2 サポートされるタイプ

| タイプ | 言語 | 出力先 |
|--------|------|-------|
| `backend-rust` | Rust | `domain/backend/rust/{name}/` |
| `backend-go` | Go | `domain/backend/go/{name}/` |
| `frontend-react` | TypeScript | `domain/frontend/react/{name}/` |
| `frontend-flutter` | Dart | `domain/frontend/flutter/{name}/` |

### 2.3 命名規則

domain 名は以下のルールに従います:

- kebab-case を使用（小文字、ハイフン区切り）
- 業務概念を表す名前を使用
- 技術用語を避ける

良い例:
```
manufacturing          # 生産管理
inventory           # 在庫管理
user-management     # ユーザー管理
order-processing    # 注文処理
```

避けるべき例:
```
database-access     # 技術用語
common              # 曖昧
shared-utils        # 技術用語 + 曖昧
manufacturing_domain   # snake_case は不可
```

予約語（使用不可）:
- `framework`, `feature`, `domain`, `k1s0`, `common`, `shared`, `utils`, `lib`

---

## 3. ディレクトリ構造

### 3.1 生成されるディレクトリ構造（Rust）

```
domain/backend/rust/manufacturing/
├── .k1s0/
│   └── manifest.json         # domain メタ情報
├── Cargo.toml                # 依存関係定義
├── README.md                 # domain 説明
├── CHANGELOG.md              # 変更履歴
├── src/
│   ├── lib.rs                # クレートルート
│   ├── domain/               # ドメイン層
│   │   ├── mod.rs
│   │   ├── entities/         # エンティティ
│   │   │   └── mod.rs
│   │   ├── value_objects/    # 値オブジェクト
│   │   │   └── mod.rs
│   │   ├── repositories/     # リポジトリトレイト
│   │   │   └── mod.rs
│   │   └── services/         # ドメインサービス
│   │       └── mod.rs
│   ├── application/          # アプリケーション層
│   │   ├── mod.rs
│   │   ├── services/         # アプリケーションサービス
│   │   │   └── mod.rs
│   │   └── dtos/             # データ転送オブジェクト
│   │       └── mod.rs
│   └── errors.rs             # domain 固有エラー
└── tests/
    └── integration_test.rs   # 統合テスト
```

### 3.2 manifest.json

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

---

## 4. 実装例

### 4.1 エンティティ（Entity）

エンティティは一意の識別子を持ち、ライフサイクルを通じて同一性を維持するオブジェクトです。

```rust
// src/domain/entities/work_order.rs

use chrono::{DateTime, Utc};
use crate::domain::value_objects::{ProductReference, Quantity, WorkOrderId, WorkOrderStatus};

/// 作業指示
#[derive(Debug, Clone)]
pub struct WorkOrder {
    id: WorkOrderId,
    product: ProductReference,
    quantity: Quantity,
    status: WorkOrderStatus,
    scheduled_start: DateTime<Utc>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl WorkOrder {
    /// 新しい作業指示を作成
    pub fn new(
        id: WorkOrderId,
        product: ProductReference,
        quantity: Quantity,
        scheduled_start: DateTime<Utc>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            product,
            quantity,
            status: WorkOrderStatus::Draft,
            scheduled_start,
            created_at: now,
            updated_at: now,
        }
    }

    /// 作業指示を開始
    pub fn start(&mut self) -> Result<(), DomainError> {
        if self.status != WorkOrderStatus::Scheduled {
            return Err(DomainError::InvalidStateTransition {
                from: self.status.clone(),
                to: WorkOrderStatus::InProgress,
            });
        }
        self.status = WorkOrderStatus::InProgress;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// 作業指示を完了
    pub fn complete(&mut self) -> Result<(), DomainError> {
        if self.status != WorkOrderStatus::InProgress {
            return Err(DomainError::InvalidStateTransition {
                from: self.status.clone(),
                to: WorkOrderStatus::Completed,
            });
        }
        self.status = WorkOrderStatus::Completed;
        self.updated_at = Utc::now();
        Ok(())
    }

    // Getters
    pub fn id(&self) -> &WorkOrderId { &self.id }
    pub fn product(&self) -> &ProductReference { &self.product }
    pub fn quantity(&self) -> &Quantity { &self.quantity }
    pub fn status(&self) -> &WorkOrderStatus { &self.status }
}
```

### 4.2 値オブジェクト（Value Object）

値オブジェクトは不変で、属性の組み合わせで等価性を判断するオブジェクトです。

```rust
// src/domain/value_objects/quantity.rs

use crate::domain::errors::ValidationError;

/// 数量単位
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuantityUnit {
    Piece,
    Kilogram,
    Liter,
    Meter,
}

/// 数量
#[derive(Debug, Clone, PartialEq)]
pub struct Quantity {
    value: u32,
    unit: QuantityUnit,
}

impl Quantity {
    /// 新しい数量を作成
    pub fn new(value: u32, unit: QuantityUnit) -> Result<Self, ValidationError> {
        if value == 0 {
            return Err(ValidationError::ZeroQuantity);
        }
        Ok(Self { value, unit })
    }

    /// 数量を加算
    pub fn add(&self, other: &Quantity) -> Result<Quantity, ValidationError> {
        if self.unit != other.unit {
            return Err(ValidationError::UnitMismatch {
                expected: self.unit.clone(),
                actual: other.unit.clone(),
            });
        }
        Ok(Quantity {
            value: self.value + other.value,
            unit: self.unit.clone(),
        })
    }

    /// 数量を減算
    pub fn subtract(&self, other: &Quantity) -> Result<Quantity, ValidationError> {
        if self.unit != other.unit {
            return Err(ValidationError::UnitMismatch {
                expected: self.unit.clone(),
                actual: other.unit.clone(),
            });
        }
        if self.value < other.value {
            return Err(ValidationError::NegativeQuantity);
        }
        Ok(Quantity {
            value: self.value - other.value,
            unit: self.unit.clone(),
        })
    }

    pub fn value(&self) -> u32 { self.value }
    pub fn unit(&self) -> &QuantityUnit { &self.unit }
}

// src/domain/value_objects/work_order_id.rs

use uuid::Uuid;

/// 作業指示ID
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorkOrderId(Uuid);

impl WorkOrderId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for WorkOrderId {
    fn default() -> Self {
        Self::new()
    }
}
```

### 4.3 ドメインサービス（Domain Service）

ドメインサービスは、エンティティや値オブジェクトに自然に属さないビジネスロジックを担当します。

```rust
// src/domain/services/scheduling_service.rs

use chrono::{DateTime, Duration, Utc};
use crate::domain::entities::WorkOrder;
use crate::domain::value_objects::ProductionCapacity;

/// スケジューリングサービス
pub struct SchedulingService;

impl SchedulingService {
    /// 完了予定日を計算
    pub fn calculate_completion_date(
        &self,
        work_order: &WorkOrder,
        capacity: &ProductionCapacity,
    ) -> DateTime<Utc> {
        let quantity = work_order.quantity().value();
        let daily_capacity = capacity.daily_output();

        let days_required = (quantity as f64 / daily_capacity as f64).ceil() as i64;

        work_order.scheduled_start() + Duration::days(days_required)
    }

    /// 作業指示の優先度を計算
    pub fn calculate_priority(
        &self,
        work_order: &WorkOrder,
        deadline: DateTime<Utc>,
    ) -> Priority {
        let remaining_time = deadline - work_order.scheduled_start();

        if remaining_time < Duration::hours(24) {
            Priority::Critical
        } else if remaining_time < Duration::days(3) {
            Priority::High
        } else if remaining_time < Duration::days(7) {
            Priority::Medium
        } else {
            Priority::Low
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}
```

### 4.4 リポジトリインターフェース

リポジトリインターフェース（トレイト）は domain 層で定義し、実装は feature 層で行います。

```rust
// src/domain/repositories/work_order_repository.rs

use async_trait::async_trait;
use crate::domain::entities::WorkOrder;
use crate::domain::value_objects::WorkOrderId;
use crate::domain::errors::DomainError;

/// 作業指示リポジトリトレイト
#[async_trait]
pub trait WorkOrderRepository: Send + Sync {
    /// ID で作業指示を検索
    async fn find_by_id(&self, id: &WorkOrderId) -> Result<Option<WorkOrder>, DomainError>;

    /// 作業指示を保存
    async fn save(&self, work_order: &WorkOrder) -> Result<(), DomainError>;

    /// 作業指示を削除
    async fn delete(&self, id: &WorkOrderId) -> Result<(), DomainError>;

    /// 全ての作業指示を取得
    async fn find_all(&self) -> Result<Vec<WorkOrder>, DomainError>;

    /// ステータスで作業指示を検索
    async fn find_by_status(&self, status: &WorkOrderStatus) -> Result<Vec<WorkOrder>, DomainError>;
}
```

### 4.5 アプリケーションサービス

アプリケーションサービスは、ユースケースを実装します。複数の feature で共有されるユースケースは domain 層に配置します。

```rust
// src/application/services/work_order_service.rs

use crate::domain::entities::WorkOrder;
use crate::domain::repositories::WorkOrderRepository;
use crate::domain::services::SchedulingService;
use crate::domain::value_objects::{ProductReference, Quantity, WorkOrderId};
use crate::domain::errors::DomainError;
use crate::application::dtos::{CreateWorkOrderCommand, WorkOrderDto};

/// 作業指示サービス
pub struct WorkOrderService<R: WorkOrderRepository> {
    repository: R,
    scheduling_service: SchedulingService,
}

impl<R: WorkOrderRepository> WorkOrderService<R> {
    pub fn new(repository: R) -> Self {
        Self {
            repository,
            scheduling_service: SchedulingService,
        }
    }

    /// 作業指示を作成
    pub async fn create_work_order(
        &self,
        command: CreateWorkOrderCommand,
    ) -> Result<WorkOrderDto, DomainError> {
        // バリデーション
        let quantity = Quantity::new(command.quantity, command.unit)?;
        let product = ProductReference::new(command.product_id)?;

        // エンティティ作成
        let work_order = WorkOrder::new(
            WorkOrderId::new(),
            product,
            quantity,
            command.scheduled_start,
        );

        // 永続化
        self.repository.save(&work_order).await?;

        Ok(WorkOrderDto::from(work_order))
    }

    /// 作業指示を開始
    pub async fn start_work_order(
        &self,
        id: WorkOrderId,
    ) -> Result<WorkOrderDto, DomainError> {
        let mut work_order = self.repository
            .find_by_id(&id)
            .await?
            .ok_or(DomainError::NotFound { entity: "WorkOrder", id: id.to_string() })?;

        work_order.start()?;
        self.repository.save(&work_order).await?;

        Ok(WorkOrderDto::from(work_order))
    }
}
```

---

## 5. ベストプラクティス

### 5.1 domain の粒度

**適切な粒度**:
- 2-3個以上の関連するエンティティをまとめる
- 100ファイル以上になったら分割を検討
- Bounded Context（境界づけられたコンテキスト）に沿う

**良い例**:
```
manufacturing/           # 生産管理（WorkOrder, ProductionLine, Schedule）
inventory/            # 在庫管理（Stock, Location, Movement）
```

**悪い例**:
```
work-order/          # エンティティ1つだけ -> 細かすぎ
manufacturing/       # 生産 + 在庫 + 品質管理 -> 大きすぎ
```

### 5.2 依存関係の管理

```rust
// Cargo.toml での framework 依存
[dependencies]
k1s0-error = { path = "../../../../framework/backend/rust/crates/k1s0-error" }
k1s0-config = { path = "../../../../framework/backend/rust/crates/k1s0-config" }

// 他の domain への依存（必要最小限に）
manufacturing-core = { path = "../manufacturing-core" }  # 慎重に
```

### 5.3 テストの書き方

```rust
// tests/integration_test.rs

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::{Quantity, QuantityUnit};

    #[test]
    fn test_quantity_creation() {
        let qty = Quantity::new(10, QuantityUnit::Piece);
        assert!(qty.is_ok());

        let zero_qty = Quantity::new(0, QuantityUnit::Piece);
        assert!(zero_qty.is_err());
    }

    #[test]
    fn test_quantity_addition() {
        let qty1 = Quantity::new(10, QuantityUnit::Piece).unwrap();
        let qty2 = Quantity::new(5, QuantityUnit::Piece).unwrap();

        let result = qty1.add(&qty2).unwrap();
        assert_eq!(result.value(), 15);
    }
}
```

### 5.4 エラーハンドリング

```rust
// src/domain/errors.rs

use k1s0_error::K1s0Error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Entity not found: {entity} with id {id}")]
    NotFound { entity: &'static str, id: String },

    #[error("Invalid state transition from {from:?} to {to:?}")]
    InvalidStateTransition {
        from: WorkOrderStatus,
        to: WorkOrderStatus,
    },

    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),

    #[error("Repository error: {0}")]
    Repository(String),
}

impl From<DomainError> for K1s0Error {
    fn from(err: DomainError) -> Self {
        match err {
            DomainError::NotFound { .. } => {
                K1s0Error::not_found(err.to_string())
            }
            DomainError::Validation(_) => {
                K1s0Error::validation(err.to_string())
            }
            _ => K1s0Error::internal(err.to_string()),
        }
    }
}
```

---

## 6. feature からの利用

### 6.1 domain に依存する feature の作成

```bash
k1s0 new-feature --type backend-rust --name work-order-api --domain manufacturing
```

### 6.2 feature での domain 利用例

```rust
// feature/backend/rust/work-order-api/src/infrastructure/repositories/work_order_repository_impl.rs

use async_trait::async_trait;
use manufacturing::domain::entities::WorkOrder;
use manufacturing::domain::repositories::WorkOrderRepository;
use manufacturing::domain::value_objects::WorkOrderId;
use manufacturing::domain::errors::DomainError;
use k1s0_db::DbPool;

pub struct WorkOrderRepositoryImpl {
    pool: DbPool,
}

#[async_trait]
impl WorkOrderRepository for WorkOrderRepositoryImpl {
    async fn find_by_id(&self, id: &WorkOrderId) -> Result<Option<WorkOrder>, DomainError> {
        // 実際のDB操作
        sqlx::query_as::<_, WorkOrderRow>("SELECT * FROM work_orders WHERE id = $1")
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DomainError::Repository(e.to_string()))?
            .map(|row| row.into())
            .transpose()
    }

    // ... 他のメソッド実装
}
```

---

## 7. 関連ドキュメント

- [ADR-0006: 3層アーキテクチャ](../adr/ADR-0006-three-layer-architecture.md)
- [domain バージョン管理ガイド](domain-versioning.md)
- [3層構造への移行ガイド](migration-to-three-tier.md)
- [domain 境界の判断基準](../conventions/domain-boundaries.md)
- [Clean Architecture](../architecture/clean-architecture.md)
- [Lint ルール K040-K047](../design/lint/rules-layer-deps.md)
