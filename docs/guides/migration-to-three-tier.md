# 3層構造への移行ガイド

このガイドでは、k1s0 の2層構造（framework -> feature）から3層構造（framework -> domain -> feature）への移行方法について説明します。

## 1. 概要

### 1.1 移行の背景

2層構造では以下の課題がありました:

- 複数の feature で同じビジネスロジックが重複
- ビジネスルール変更時に複数の feature を個別に更新が必要
- feature 間でコードを共有する標準的な方法がない

3層構造を導入することで、共通のビジネスロジックを domain 層に集約し、コードの再利用性と保守性を向上させます。

### 1.2 移行の原則

1. **段階的移行**: 一度に全てを移行せず、段階的に進める
2. **後方互換性**: 既存の feature は domain なしでも動作を継続
3. **テスト駆動**: 移行前後でテストが通ることを確認
4. **最小限の影響**: 既存の API を可能な限り維持

---

## 2. 移行判断のフローチャート

```
開始
  │
  ▼
複数の feature で同じビジネスロジックが
重複している？
  │
  ├─ いいえ → 現状維持（2層構造のまま）
  │
  └─ はい
      │
      ▼
    そのロジックは複数の feature で
    今後も共有される見込みがある？
      │
      ├─ いいえ → 現状維持
      │
      └─ はい
          │
          ▼
        domain 層に切り出す
```

---

## 3. 移行前の準備

### 3.1 現状分析

移行対象の特定:

```bash
# 重複コードの調査（例: エンティティ名で検索）
grep -r "struct WorkOrder" feature/backend/rust/

# 依存関係の確認
k1s0 lint --rules K022  # Clean Architecture 違反をチェック
```

### 3.2 移行対象の選定基準

domain に切り出すべきもの:

| 種類 | 基準 | 例 |
|------|------|-----|
| エンティティ | 2つ以上の feature で使用 | WorkOrder, Product |
| 値オブジェクト | 2つ以上の feature で使用 | Quantity, Money |
| ドメインサービス | ビジネスルールが複雑かつ共有 | SchedulingService |
| リポジトリトレイト | 標準的なインターフェースとして共有 | WorkOrderRepository |

domain に切り出さないもの:

- feature 固有のユースケース
- プレゼンテーション層のコード
- インフラストラクチャ層の実装

### 3.3 テストの確認

```bash
# 移行前のテスト状態を記録
cd CLI && cargo test --all 2>&1 | tee ../migration-test-before.log
```

---

## 4. 移行手順

### Phase 1: domain の作成

#### Step 1.1: domain の雛形を作成

```bash
k1s0 new-domain --type backend-rust --name manufacturing
```

#### Step 1.2: 初期バージョンを設定

```bash
k1s0 domain version --name manufacturing --set 0.1.0
```

生成される manifest.json:

```json
{
  "layer": "domain",
  "version": "0.1.0",
  "dependencies": {
    "framework": ["k1s0-error", "k1s0-config"]
  }
}
```

### Phase 2: コードの切り出し

#### Step 2.1: エンティティの移行

**Before（feature 内）**:
```rust
// feature/backend/rust/work-order-api/src/domain/entities/work_order.rs
pub struct WorkOrder {
    pub id: Uuid,
    pub product_id: Uuid,
    pub quantity: u32,
    pub status: String,
}
```

**After（domain 層）**:
```rust
// domain/backend/rust/manufacturing/src/domain/entities/work_order.rs
use crate::domain::value_objects::{WorkOrderId, ProductReference, Quantity, WorkOrderStatus};

pub struct WorkOrder {
    id: WorkOrderId,
    product: ProductReference,
    quantity: Quantity,
    status: WorkOrderStatus,
}

impl WorkOrder {
    // ビジネスロジックをエンティティに移動
    pub fn start(&mut self) -> Result<(), DomainError> { ... }
    pub fn complete(&mut self) -> Result<(), DomainError> { ... }
}
```

#### Step 2.2: 値オブジェクトの移行

**Before（feature 内）**:
```rust
// feature/backend/rust/work-order-api/src/domain/types.rs
pub type WorkOrderId = Uuid;
pub type Quantity = u32;
```

**After（domain 層）**:
```rust
// domain/backend/rust/manufacturing/src/domain/value_objects/quantity.rs
#[derive(Debug, Clone, PartialEq)]
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

    // バリデーションロジックを値オブジェクトに移動
}
```

#### Step 2.3: ドメインサービスの移行

**Before（feature 内）**:
```rust
// feature/backend/rust/work-order-api/src/application/services/scheduling.rs
pub fn calculate_completion_date(work_order: &WorkOrder, capacity: f64) -> DateTime<Utc> {
    // ビジネスロジック
}
```

**After（domain 層）**:
```rust
// domain/backend/rust/manufacturing/src/domain/services/scheduling_service.rs
pub struct SchedulingService;

impl SchedulingService {
    pub fn calculate_completion_date(
        &self,
        work_order: &WorkOrder,
        capacity: &ProductionCapacity,
    ) -> DateTime<Utc> {
        // ビジネスロジック
    }
}
```

#### Step 2.4: リポジトリインターフェースの移行

**Before（feature 内）**:
```rust
// feature/backend/rust/work-order-api/src/domain/repositories/work_order_repo.rs
#[async_trait]
pub trait WorkOrderRepo {
    async fn find(&self, id: Uuid) -> Option<WorkOrder>;
    async fn save(&self, order: &WorkOrder);
}
```

**After（domain 層）**:
```rust
// domain/backend/rust/manufacturing/src/domain/repositories/work_order_repository.rs
#[async_trait]
pub trait WorkOrderRepository: Send + Sync {
    async fn find_by_id(&self, id: &WorkOrderId) -> Result<Option<WorkOrder>, DomainError>;
    async fn save(&self, work_order: &WorkOrder) -> Result<(), DomainError>;
}
```

### Phase 3: feature の更新

#### Step 3.1: domain への依存を追加

feature の manifest.json を更新:

```json
{
  "layer": "feature",
  "domain": "manufacturing",
  "domain_version": "^0.1.0",
  "dependencies": {
    "framework": ["k1s0-error", "k1s0-config", "k1s0-db"],
    "domain": {
      "manufacturing": "^0.1.0"
    }
  }
}
```

feature の Cargo.toml を更新:

```toml
[dependencies]
manufacturing = { path = "../../../../domain/backend/rust/manufacturing" }
```

#### Step 3.2: import 文の更新

**Before**:
```rust
use crate::domain::entities::WorkOrder;
use crate::domain::repositories::WorkOrderRepo;
```

**After**:
```rust
use manufacturing::domain::entities::WorkOrder;
use manufacturing::domain::repositories::WorkOrderRepository;
```

#### Step 3.3: リポジトリ実装の更新

**Before**:
```rust
impl WorkOrderRepo for WorkOrderRepoImpl {
    async fn find(&self, id: Uuid) -> Option<WorkOrder> { ... }
}
```

**After**:
```rust
use manufacturing::domain::repositories::WorkOrderRepository;

impl WorkOrderRepository for WorkOrderRepositoryImpl {
    async fn find_by_id(&self, id: &WorkOrderId) -> Result<Option<WorkOrder>, DomainError> {
        // 新しいインターフェースに合わせて実装
    }
}
```

### Phase 4: テストと検証

#### Step 4.1: domain のテスト

```bash
cd domain/backend/rust/manufacturing
cargo test
```

#### Step 4.2: feature のテスト

```bash
cd feature/backend/rust/work-order-api
cargo test
```

#### Step 4.3: Lint チェック

```bash
k1s0 lint
```

期待される結果:
- K041 エラーなし（domain が見つかる）
- K042 エラーなし（バージョン制約を満たす）
- K047 エラーなし（domain に version が設定されている）

#### Step 4.4: 全体テスト

```bash
cd CLI && cargo test --all
```

---

## 5. 段階的移行アプローチ

### 5.1 アプローチ 1: Strangler Fig パターン

既存のコードを徐々に domain に移行し、最終的に古いコードを削除します。

```
Phase 1: domain を作成（空の状態）
Phase 2: 1つのエンティティを domain に移行
Phase 3: feature から domain のエンティティを参照
Phase 4: 古いコードを削除
Phase 5: 次のエンティティを移行...
```

### 5.2 アプローチ 2: Facade パターン

domain をファサードとして作成し、内部で既存の feature コードを参照します。

```rust
// domain/backend/rust/manufacturing/src/facade.rs

// Phase 1: 既存のコードを再エクスポート
pub use work_order_api::domain::entities::WorkOrder;
pub use work_order_api::domain::repositories::WorkOrderRepo as WorkOrderRepository;

// Phase 2: 徐々に独自実装に置き換え
// pub struct WorkOrder { ... }  // 新実装
// pub use work_order_api::domain::entities::WorkOrder;  // 削除
```

### 5.3 アプローチ 3: 並行運用

一定期間、両方の実装を維持し、テストで検証します。

```rust
// 両方の実装を維持
mod legacy {
    pub use work_order_api::domain::entities::WorkOrder;
}

mod new {
    pub struct WorkOrder { ... }
}

// テストで両者を比較
#[cfg(test)]
mod migration_tests {
    #[test]
    fn test_equivalence() {
        let legacy = legacy::WorkOrder::new(...);
        let new = new::WorkOrder::new(...);
        assert_eq!(legacy.id, new.id().as_uuid());
    }
}
```

---

## 6. 移行チェックリスト

### 6.1 移行前

- [ ] 重複コードの特定
- [ ] 移行対象の選定
- [ ] テストの実行と結果記録
- [ ] 移行計画の作成

### 6.2 domain 作成

- [ ] `k1s0 new-domain` で雛形作成
- [ ] manifest.json のバージョン設定
- [ ] Cargo.toml の依存関係設定

### 6.3 コード移行

- [ ] エンティティの移行
- [ ] 値オブジェクトの移行
- [ ] ドメインサービスの移行
- [ ] リポジトリインターフェースの移行
- [ ] エラー型の移行

### 6.4 feature 更新

- [ ] manifest.json の更新（domain 依存追加）
- [ ] Cargo.toml の更新
- [ ] import 文の更新
- [ ] リポジトリ実装の更新

### 6.5 テスト・検証

- [ ] domain のユニットテスト
- [ ] feature のユニットテスト
- [ ] 統合テスト
- [ ] Lint チェック（K040-K047）
- [ ] E2E テスト

### 6.6 ドキュメント

- [ ] CHANGELOG.md の更新
- [ ] README.md の更新
- [ ] 移行ガイドの作成（必要に応じて）

---

## 7. トラブルシューティング

### 7.1 循環依存エラー

**症状**: K043 エラーが発生

**原因**: domain 間で循環参照が発生

**解決策**:
1. 依存関係を見直し、一方向にする
2. 共通部分を別の domain に切り出す
3. インターフェースで依存を逆転させる

```rust
// 解決策 3: 依存性逆転
// domain-a から domain-b へ依存している場合

// domain-a/src/interfaces/external.rs
pub trait ExternalService {
    fn process(&self, data: &Data) -> Result<(), Error>;
}

// domain-b はこのトレイトを実装
// domain-a は domain-b に直接依存しない
```

### 7.2 バージョン制約エラー

**症状**: K042 エラーが発生

**原因**: feature が要求するバージョンと domain のバージョンが不一致

**解決策**:
```bash
# domain のバージョンを確認
k1s0 domain version --name manufacturing

# feature のバージョン制約を更新
k1s0 feature update-domain --name work-order-api --domain manufacturing --version "^1.0.0"
```

### 7.3 import パスの問題

**症状**: コンパイルエラー「cannot find module」

**原因**: Cargo.toml のパス設定が間違っている

**解決策**:
```toml
# 正しいパス設定
[dependencies]
manufacturing = { path = "../../../../domain/backend/rust/manufacturing" }

# ワークスペースを使用している場合
manufacturing = { workspace = true }
```

---

## 8. 関連ドキュメント

- [domain 開発ガイド](domain-development.md)
- [domain バージョン管理ガイド](domain-versioning.md)
- [domain 境界の判断基準](../conventions/domain-boundaries.md)
- [ADR-0006: 3層アーキテクチャ](../adr/ADR-0006-three-layer-architecture.md)
- [Lint ルール K040-K047](../design/lint/rules-layer-deps.md)
