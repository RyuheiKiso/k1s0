# Saga 補償フロー詳細

## 概要

本文書は `server.md` で定義された Saga Orchestrator の補償トランザクションについて、
`workflows/order-fulfillment.yaml` を例とした具体的なフロー詳細を記述する。

Saga サーバーの実装: `regions/system/server/rust/saga/`

## order-fulfillment Saga

### ワークフロー定義

`regions/system/server/rust/saga/workflows/order-fulfillment.yaml` で定義される
3ステップのワークフロー:

```
Step 1: reserve-inventory   → InventoryService.Reserve
Step 2: process-payment     → PaymentService.Charge
Step 3: arrange-shipping    → ShippingService.CreateShipment
```

### 正常フロー（全ステップ成功）

```
OrderService → SagaOrchestrator: StartSaga("order-fulfillment", payload)
  SagaOrchestrator → InventoryService: Reserve(item_id, qty)
  InventoryService → SagaOrchestrator: OK(reservation_id)
  SagaOrchestrator → PaymentService: Charge(amount, payment_method)
  PaymentService → SagaOrchestrator: OK(payment_id)
  SagaOrchestrator → ShippingService: CreateShipment(order_id, address)
  ShippingService → SagaOrchestrator: OK(shipment_id)
  SagaOrchestrator → OrderService: SagaCompleted
```

### 補償フロー1: Step 2（PaymentService）で失敗

```
SagaOrchestrator: PaymentService.Charge が失敗
  ↓ COMPENSATING 状態に遷移
  SagaOrchestrator → InventoryService: Release(reservation_id)  ← Step 1 の補償
  InventoryService → SagaOrchestrator: OK
  ↓ COMPENSATED 状態に遷移
  SagaOrchestrator → OrderService: SagaFailed(PAYMENT_FAILED)
```

### 補償フロー2: Step 3（ShippingService）で失敗

```
SagaOrchestrator: ShippingService.CreateShipment が失敗
  ↓ COMPENSATING 状態に遷移
  SagaOrchestrator → PaymentService: Refund(payment_id, amount)   ← Step 2 の補償
  PaymentService → SagaOrchestrator: OK
  SagaOrchestrator → InventoryService: Release(reservation_id)   ← Step 1 の補償
  InventoryService → SagaOrchestrator: OK
  ↓ COMPENSATED 状態に遷移
  SagaOrchestrator → OrderService: SagaFailed(SHIPPING_FAILED)
```

## Saga ステータス遷移

```
STARTED
  │
  ├─ ステップ成功 → RUNNING
  │                  │
  │                  ├─ 全ステップ成功 → COMPLETED (終端)
  │                  └─ ステップ失敗 → COMPENSATING
  │                                      │
  │                                      ├─ 補償成功 → COMPENSATED (終端)
  │                                      └─ 補償失敗 → COMPENSATION_FAILED (終端)
  └─ キャンセル要求 → CANCELLED (終端)
```

## 冪等性の保証

各補償メソッドは冪等キーを使用して重複実行を安全に処理する:

| サービス | メソッド | 冪等性キー | 実装方針 |
|---------|---------|-----------|---------|
| InventoryService | Release | `reservation_id` | reservation_id で重複解放を検出・スキップ |
| PaymentService | Refund | `payment_id` | payment_id で重複返金を検出・スキップ |
| ShippingService | Cancel | `shipment_id` | shipment_id で重複キャンセルを検出・スキップ |

```rust
// 冪等性チェックの実装パターン（各サービスで共通）
pub async fn release_stock(
    &self,
    reservation_id: Uuid,
) -> Result<(), ServiceError> {
    // 同一 reservation_id で既に解放済みかを確認する（冪等性保証）
    if self.repository.is_released(reservation_id).await? {
        tracing::info!("Stock already released for reservation_id={}", reservation_id);
        return Ok(());
    }
    self.repository.release(reservation_id).await
}
```

## 補償失敗時のエスカレーション

### リトライポリシー（workflow YAML で設定）

```yaml
# workflows/order-fulfillment.yaml の retry 設定
steps:
  - name: reserve-inventory
    compensate: release-inventory
    retry:
      max_attempts: 3
      backoff: exponential
      initial_interval_ms: 1000
```

### エスカレーション手順

1. **自動リトライ**: exponential backoff で最大 `max_attempts` 回リトライ
2. **COMPENSATION_FAILED 遷移**: リトライ上限到達後、`COMPENSATION_FAILED` 状態に遷移
3. **DLQ 投入**: dlq-manager サーバーへのメッセージ送信（手動介入待ち）
4. **Slack 通知**: AlertManager → prometheus-msteams 経由で通知
5. **手動補償 API**: `POST /api/v1/sagas/{id}/compensate` で手動実行

### 監視・アラート

```promql
# 補償失敗の検出クエリ（Prometheus）
increase(saga_compensation_failed_total[5m]) > 0
```

Grafana の `overview.json` ダッシュボードで Saga 状態の分布を確認できる。

## テスト

### 統合テスト

`regions/system/server/rust/saga/tests/` 配下のテスト:
- `integration_test.rs`: 基本的な Saga 実行・補償フローの E2E テスト
- `kafka_integration_test.rs`: Kafka を介したイベント発行のテスト
- `postgres_repository_test.rs`: 実 DB を使ったリポジトリテスト
- `workflow_engine_test.rs`: ワークフローエンジンのユニットテスト

### シナリオテスト例

```rust
// 補償フローのテスト: PaymentService 失敗時に在庫が解放されることを確認する
#[tokio::test]
#[ignore] // 実サービスが必要なため CI の統合テストで実行する
async fn test_compensation_on_payment_failure() {
    // PaymentService をスタブで差し替え、失敗を注入する
    let payment_stub = StubPaymentService::always_fail();
    // Saga を開始する
    let saga_id = saga_client.start("order-fulfillment", payload).await.unwrap();
    // Saga が COMPENSATED 状態になることを確認する
    assert_saga_status(saga_id, SagaStatus::Compensated).await;
    // 在庫が元の状態に戻っていることを確認する
    assert_stock_released(item_id).await;
}
```

## 関連ドキュメント

- Saga サーバー全体設計: `docs/servers/system/saga/server.md`
- Saga DB スキーマ: `docs/servers/system/saga/database.md`
- ワークフロー定義: `regions/system/server/rust/saga/workflows/`
- 分散ロック方針: `docs/architecture/conventions/分散ロック方針.md`
