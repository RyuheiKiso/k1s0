# Saga 補償フロー詳細

## 概要

本文書は `server.md` で定義された Saga Orchestrator の補償トランザクションについて、
`workflows/task-assignment.yaml` を例とした具体的なフロー詳細を記述する。

Saga サーバーの実装: `regions/system/server/rust/saga/`

## task-assignment Saga

### ワークフロー定義

`regions/system/server/rust/saga/workflows/task-assignment.yaml` で定義される
3ステップのワークフロー:

```
Step 1: create-task             → TaskService.CreateTask
Step 2: increment-board-column  → BoardService.IncrementColumn
Step 3: log-activity            → ActivityService.CreateActivity
```

### 正常フロー（全ステップ成功）

```
TaskService → SagaOrchestrator: StartSaga("task-assignment", payload)
  SagaOrchestrator → TaskService: CreateTask(task_id, assignee)
  TaskService → SagaOrchestrator: OK(task_id)
  SagaOrchestrator → BoardService: IncrementColumn(board_id, column)
  BoardService → SagaOrchestrator: OK(column_count)
  SagaOrchestrator → ActivityService: CreateActivity(task_id, action)
  ActivityService → SagaOrchestrator: OK(activity_id)
  SagaOrchestrator → TaskService: SagaCompleted
```

### 補償フロー1: Step 2（BoardService）で失敗

```
SagaOrchestrator: BoardService.IncrementColumn が失敗
  ↓ COMPENSATING 状態に遷移
  SagaOrchestrator → TaskService: CancelTask(task_id)  ← Step 1 の補償
  TaskService → SagaOrchestrator: OK
  ↓ COMPENSATED 状態に遷移
  SagaOrchestrator → TaskService: SagaFailed(BOARD_INCREMENT_FAILED)
```

### 補償フロー2: Step 3（ActivityService）で失敗

```
SagaOrchestrator: ActivityService.CreateActivity が失敗
  ↓ COMPENSATING 状態に遷移
  SagaOrchestrator → BoardService: DecrementColumn(board_id, column)  ← Step 2 の補償
  BoardService → SagaOrchestrator: OK
  SagaOrchestrator → TaskService: CancelTask(task_id)                 ← Step 1 の補償
  TaskService → SagaOrchestrator: OK
  ↓ COMPENSATED 状態に遷移
  SagaOrchestrator → TaskService: SagaFailed(ACTIVITY_FAILED)
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
| TaskService | CancelTask | `task_id` | task_id で重複キャンセルを検出・スキップ |
| BoardService | DecrementColumn | `board_id` | board_id で重複デクリメントを検出・スキップ |
| ActivityService | DeleteActivity | `activity_id` | activity_id で重複削除を検出・スキップ |

```rust
// 冪等性チェックの実装パターン（各サービスで共通）
pub async fn cancel_task(
    &self,
    task_id: Uuid,
) -> Result<(), ServiceError> {
    // 同一 task_id で既にキャンセル済みかを確認する（冪等性保証）
    if self.repository.is_cancelled(task_id).await? {
        tracing::info!("Task already cancelled for task_id={}", task_id);
        return Ok(());
    }
    self.repository.cancel(task_id).await
}
```

## 補償失敗時のエスカレーション

### リトライポリシー（workflow YAML で設定）

```yaml
# workflows/task-assignment.yaml の retry 設定
steps:
  - name: create-task
    compensate: TaskService.CancelTask
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
// 補償フローのテスト: BoardService 失敗時にタスクがキャンセルされることを確認する
#[tokio::test]
#[ignore] // 実サービスが必要なため CI の統合テストで実行する
async fn test_compensation_on_board_increment_failure() {
    // BoardService をスタブで差し替え、失敗を注入する
    let board_stub = StubBoardService::always_fail();
    // Saga を開始する
    let saga_id = saga_client.start("task-assignment", payload).await.unwrap();
    // Saga が COMPENSATED 状態になることを確認する
    assert_saga_status(saga_id, SagaStatus::Compensated).await;
    // タスクがキャンセルされていることを確認する
    assert_task_cancelled(task_id).await;
}
```

## 関連ドキュメント

- Saga サーバー全体設計: `docs/servers/system/saga/server.md`
- Saga DB スキーマ: `docs/servers/system/saga/database.md`
- ワークフロー定義: `regions/system/server/rust/saga/workflows/`
- 分散ロック方針: `docs/architecture/conventions/分散ロック方針.md`
