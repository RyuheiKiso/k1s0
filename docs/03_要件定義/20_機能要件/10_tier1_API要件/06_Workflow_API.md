# Workflow API

本書は、tier1 が公開する Workflow API の機能要件を定義する。tier2/tier3 の業務ワークフロー・Saga を、Dapr Workflow（短期）と Temporal Go SDK（長期実行）の 2 バックエンドで提供する。

## API 概要

業務の多段階処理（申請 → 承認 → 通知 → 在庫引当 → 決済）を、失敗時の補償ロジック（ロールバック）を含めて宣言的に記述する API。短期（数秒〜数十分）は Dapr Workflow、長期（数時間〜数日）は Temporal が担当する。

内部実装は、短期 Dapr Workflow（Go SDK）と長期 Temporal Go SDK を併用し、tier1 が呼び分ける。tier2 からは単一の API 体系に見える。ZEN Engine の Decision API は、Workflow の中で呼び出される分岐条件評価として役割分担する。

## 機能要件

### FR-T1-WORKFLOW-001: 短期ワークフロー（Dapr Workflow）

**現状**: tier2 が多段階処理を書くには、State API でフラグ管理、PubSub でイベント連携、Saga パターンを手書きする必要がある。失敗時の補償ロジックはコード分散し、保守が困難。

**要件達成後**: `k1s0.Workflow.RunShort("workflow-name", input)` で Dapr Workflow を起動する。Workflow 定義は Go の関数（`ActivityX → ActivityY → ActivityZ`）で書き、エラー時の補償は `try/catch` 相当で表現。実行状態は Dapr が永続化し、Pod 再起動でも中断しない。

**崩れた時**: 多段階処理の失敗リカバリが tier2 アプリ個別実装となり、再実行・巻き戻しの挙動がバラつく。障害発生時のデバッグが困難になる。

**受け入れ基準**:
- Workflow 実行状態を State API バックエンド（Valkey）で永続化
- Pod 再起動後も Workflow は途中から再開
- 最大実行時間はデフォルト 30 分、超過で自動タイムアウト
- Workflow 実行ごとに一意の `workflow_id` を返す

### FR-T1-WORKFLOW-002: 長期実行 Saga（Temporal）

**現状**: 数時間〜数日に及ぶ業務プロセス（稟議承認、月次バッチ、在庫棚卸）を Dapr Workflow で書くと、タイマー精度・スケーラビリティ・Workflow バージョニングの制約に当たる。

**要件達成後**: `k1s0.Workflow.RunLong("workflow-name", input, options)` で Temporal Workflow を起動する。Temporal のタイマー（日単位）、バージョニング、大量並列実行に対応。tier2 から見たインタフェースは Dapr Workflow と共通。

**崩れた時**: 長期業務プロセスの実装が tier2 ごとにバラつき、状態永続化のスケジュール制御・再開処理で事故が発生する。

**受け入れ基準**:
- 最大実行時間は無制限（Temporal の仕様に従う）
- Workflow バージョニングで互換性維持
- Temporal と Dapr Workflow の共通 API で tier2 から切替可能
- 優先度 SHOULD（Temporal 運用負荷次第、Phase 1b 評価）

### FR-T1-WORKFLOW-003: 補償ロジック（Compensating Transactions）

**現状**: Saga パターンの補償処理（ロールバック）は tier2 が個別実装する。複数サービスに跨る補償の順序制御は手書きで、忘れや順序誤りが発生する。

**要件達成後**: Workflow 定義内で各 Activity に `compensation` を宣言する。途中で失敗した場合、実行済み Activity の補償が逆順で自動実行される。補償失敗は別途イベントで通知される。

**崩れた時**: 分散トランザクションの不整合が発生し、業務データが半端な状態で残る。復旧に人手介入が必要になる。

**受け入れ基準**:
- 各 Activity に compensation 関数を登録可能
- Workflow 失敗時に実行済み Activity の compensation を逆順実行
- compensation 失敗は Audit API に記録、SRE に通知

### FR-T1-WORKFLOW-004: タイマー・遅延実行

**現状**: tier2 が遅延実行（例: 3 日後に催促メール送信）を実装するには、k8s CronJob か外部スケジューラ依存となる。Workflow 中の「待機」は State ポーリングで代用される。

**要件達成後**: Workflow 定義内で `WaitForTimer(duration)` を呼び出す。Dapr Workflow / Temporal のタイマー機構で精密な遅延実行。Pod 再起動でもタイマーは保持される。

**崩れた時**: 遅延処理の精度が保証されず、督促メールが 3 日後ではなく 2 日後や 4 日後に送信される事故が発生する。

**受け入れ基準**:
- 最小遅延 1 秒、最大 Dapr Workflow 7 日 / Temporal 無制限
- タイマー精度は ±1 秒以内
- Pod 再起動でもタイマーは保持

### FR-T1-WORKFLOW-005: 外部イベント待ち受け

**現状**: Workflow が外部イベント（承認通知、ユーザー確認）を待つには、イベント受信 → State 書き込み → Workflow ポーリングの組み合わせが必要。

**要件達成後**: Workflow 定義内で `WaitForEvent("event_name")` を呼び、外部から `SignalWorkflow(workflow_id, "event_name", payload)` で通知する。ポーリング不要。

**崩れた時**: Workflow が外部入力待ちのままハングし、タイムアウト制御も個別実装となる。

**受け入れ基準**:
- WaitForEvent はタイムアウト付き（指定ない場合は無制限）
- Signal は冪等、同じ event_name が複数回送られても最初のものが採用される
- 優先度 COULD（Phase 1c で判定）

## 入出力仕様

```
k1s0.Workflow.RunShort(
    workflow_name: string,
    input: Proto message,
    options?: {
        timeout_seconds?: int,
        idempotency_key?: string
    }
) -> (workflow_id: string, error: K1s0Error?)

k1s0.Workflow.RunLong(
    workflow_name: string,
    input: Proto message,
    options?: { ... }
) -> (workflow_id: string, error: K1s0Error?)

k1s0.Workflow.GetStatus(workflow_id: string) -> (status: WorkflowStatus, result?, error?)
k1s0.Workflow.SignalWorkflow(workflow_id: string, event_name: string, payload: any) -> error?
k1s0.Workflow.CancelWorkflow(workflow_id: string) -> error?
```

`WorkflowStatus` は `Running | Completed | Failed | Cancelled | TimedOut` の enum。

## 受け入れ基準（全要件共通）

- Workflow 実行の Start/Complete/Fail は Audit API に記録される
- workflow_id は UUID v7（時系列ソート可能）
- tier2 は Activity 内で State / PubSub / Decision / Binding の各 API を呼べる

## Phase 対応

- **Phase 1a**: 未提供
- **Phase 1b**: FR-T1-WORKFLOW-001、003、004（Dapr Workflow 中心）
- **Phase 1b/1c**: FR-T1-WORKFLOW-002（Temporal 採用判定）
- **Phase 1c**: FR-T1-WORKFLOW-005

## 関連非機能要件

- **NFR-A-CONT-004**: Workflow 実行の永続化と再開性
- **NFR-B-RES-003**: 並列 Workflow 数の水平拡張
- **NFR-E-MON-003**: Workflow 実行の Audit 記録
- **NFR-C-MON-003**: Workflow の遅延・滞留監視
