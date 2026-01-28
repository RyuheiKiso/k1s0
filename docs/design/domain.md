# domain 層設計

## 概要

domain 層は、k1s0 の3層アーキテクチャ（framework -> domain -> feature）における中間層です。
特定の業務領域に共通するビジネスロジックを一元管理し、複数の feature から共有できるようにします。

## domain の責務

domain 層は以下を提供します。

### 1. エンティティ（Entities）

ビジネスドメインの核となるオブジェクト。一意の識別子を持ち、ライフサイクルを通じて同一性を維持します。

```rust
// domain/backend/rust/production/src/domain/entities/work_order.rs
pub struct WorkOrder {
    pub id: WorkOrderId,
    pub product: ProductReference,
    pub quantity: Quantity,
    pub status: WorkOrderStatus,
    pub scheduled_start: DateTime<Utc>,
}
```

### 2. 値オブジェクト（Value Objects）

不変で同一性を持たないオブジェクト。属性の組み合わせで等価性を判断します。

```rust
// domain/backend/rust/production/src/domain/value_objects/quantity.rs
#[derive(Clone, Debug, PartialEq)]
pub struct Quantity {
    value: u32,
    unit: QuantityUnit,
}

impl Quantity {
    pub fn new(value: u32, unit: QuantityUnit) -> Result<Self, ValidationError> {
        if value == 0 {
            return Err(ValidationError::ZeroQuantity);
        }
        Ok(Self { value, unit })
    }
}
```

### 3. ドメインサービス（Domain Services）

エンティティや値オブジェクトに自然に属さないビジネスロジック。

```rust
// domain/backend/rust/production/src/domain/services/scheduling.rs
pub struct SchedulingService;

impl SchedulingService {
    pub fn calculate_completion_date(
        &self,
        work_order: &WorkOrder,
        capacity: &ProductionCapacity,
    ) -> DateTime<Utc> {
        // スケジューリングロジック
    }
}
```

### 4. リポジトリインターフェース

データアクセス層の抽象化。実装は feature 層で行う。

```rust
// domain/backend/rust/production/src/domain/repositories/work_order_repository.rs
#[async_trait]
pub trait WorkOrderRepository: Send + Sync {
    async fn find_by_id(&self, id: &WorkOrderId) -> Result<Option<WorkOrder>, DomainError>;
    async fn save(&self, work_order: &WorkOrder) -> Result<(), DomainError>;
}
```

### 5. アプリケーションサービス

ユースケースの共通部分。feature 固有のユースケースは feature 層で実装。

```rust
// domain/backend/rust/production/src/application/services/work_order_service.rs
pub struct WorkOrderService<R: WorkOrderRepository> {
    repository: R,
    scheduling: SchedulingService,
}

impl<R: WorkOrderRepository> WorkOrderService<R> {
    pub async fn create_work_order(&self, command: CreateWorkOrderCommand) -> Result<WorkOrder, DomainError> {
        // 共通のビジネスロジック
    }
}
```

## domain の作成

### コマンド

```bash
k1s0 new-domain --type backend-rust --name production
```

### 生成されるディレクトリ構造

```
domain/backend/rust/production/
├── .k1s0/
│   └── manifest.json
├── Cargo.toml
├── README.md
├── CHANGELOG.md
├── src/
│   ├── lib.rs
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── entities/
│   │   │   └── mod.rs
│   │   ├── value_objects/
│   │   │   └── mod.rs
│   │   └── services/
│   │       └── mod.rs
│   ├── application/
│   │   ├── mod.rs
│   │   └── services/
│   │       └── mod.rs
│   └── infrastructure/
│       ├── mod.rs
│       └── repositories/
│           └── mod.rs
└── tests/
    └── integration_test.rs
```

### manifest.json

```json
{
  "schema_version": "1.0.0",
  "k1s0_version": "0.1.0",
  "template": {
    "name": "backend-rust",
    "version": "0.1.0",
    "source": "local",
    "path": "CLI/templates/backend-rust/domain",
    "fingerprint": "..."
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

## バージョン管理

### SemVer ルール

domain は Semantic Versioning に従います。

- **MAJOR**: 破壊的変更（API 非互換）
- **MINOR**: 後方互換な機能追加
- **PATCH**: 後方互換なバグ修正

### バージョン更新

```bash
# 現在のバージョンを確認
k1s0 domain version --name production

# バージョンを更新
k1s0 domain version --name production --bump minor
```

### 破壊的変更の記録

manifest.json に breaking_changes を記録します。

```json
{
  "version": "2.0.0",
  "breaking_changes": {
    "2.0.0": "WorkOrder.quantity を Quantity 値オブジェクトに変更",
    "1.0.0": "初回リリース"
  }
}
```

## domain への依存

### feature から domain への依存

```bash
k1s0 new-feature --type backend-rust --name work-order-api --domain production
```

feature の manifest.json:

```json
{
  "layer": "feature",
  "domain": "production",
  "domain_version": "^1.2.0",
  "dependencies": {
    "domain": {
      "production": "^1.2.0"
    }
  }
}
```

### 依存の更新

```bash
k1s0 feature update-domain --name work-order-api --domain production --version "^2.0.0"
```

## 非推奨化

domain を非推奨にする場合は、manifest.json に deprecated を設定します。

```json
{
  "deprecated": {
    "since": "1.5.0",
    "migrate_to": "production-v2",
    "deadline": "2026-12-31",
    "reason": "新しい production-v2 domain に機能を統合"
  }
}
```

非推奨の domain を使用している feature は、`k1s0 lint` で K044 警告が表示されます。

## Lint ルール

domain に関連する Lint ルール。

| ID | 説明 |
|----|------|
| K040 | 層間依存の基本違反 |
| K041 | domain が見つからない |
| K042 | domain バージョン制約不整合 |
| K043 | 循環依存の検出 |
| K044 | 非推奨 domain の使用 |
| K045 | min_framework_version 違反 |
| K046 | breaking_changes の影響 |
| K047 | domain 層の version 未設定 |

## ベストプラクティス

### 1. domain の粒度

- **細かすぎない**: 1つの domain に 2-3 以上の関連エンティティをまとめる
- **大きすぎない**: 100ファイル以上になったら分割を検討
- **業務境界**: Bounded Context（境界づけられたコンテキスト）に沿う

### 2. 命名規則

- kebab-case を使用: `production`, `inventory`, `user-management`
- 技術用語を避ける: `database-access` ではなく業務名
- 予約語を避ける: `framework`, `feature`, `domain`, `k1s0`, `common`, `shared`

### 3. 依存の最小化

- framework への依存は必要最小限に
- 他の domain への依存は慎重に（循環依存のリスク）

### 4. テスト

- domain 層は単体テスト可能に設計
- 外部依存（DB、外部API）はモック化
- `tests/` ディレクトリに統合テストを配置

## 関連ドキュメント

- [ADR-0006: 3層アーキテクチャ](../adr/ADR-0006-three-layer-architecture.md)
- [Clean Architecture](../architecture/clean-architecture.md)
- [Lint ルール](lint.md)
