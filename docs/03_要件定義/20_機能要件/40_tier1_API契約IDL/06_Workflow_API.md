# 06. Workflow API

Temporal をバックエンドとする長時間ワークフロー API。開始・シグナル・クエリ・キャンセル・強制終了・状態取得を提供し、tier2/tier3 が Saga パターンや人間承認を含む数日〜数週間のビジネスプロセスを「関数呼出の延長」として記述できるようにする。

## 要件対応

- 要件ファイル: [../10_tier1_API要件/06_Workflow_API.md](../10_tier1_API要件/06_Workflow_API.md)
- 要件 ID: FR-T1-WORKFLOW-001〜005
- 共通型: [00_共通型定義.md](00_共通型定義.md)

## 設計のポイント

`workflow_id` の冪等性（`idempotent=true` で同一 ID の重複開始は既存実行を返す）は、クライアント側のリトライと Workflow 実行の一意性を両立させる。`Signal` はワークフローへの入力イベント、`Query` は副作用なしの読取り、`Cancel` はワークフロー内の補償処理を発火させる正常終了の依頼、`Terminate` は補償処理をスキップする強制終了と、4 種類の介入が明確に分離されている。状態列挙 `WorkflowStatus` は Temporal の標準状態に準拠し、`CONTINUED_AS_NEW` は長期実行ワークフローの履歴圧縮のために独立した状態として扱う。

## Protobuf 定義

```protobuf
// Workflow API (FR-T1-WORKFLOW-001〜005)
syntax = "proto3";
package k1s0.tier1.workflow.v1;
import "k1s0/tier1/common/v1/common.proto";

service WorkflowService {
  // ワークフロー開始
  rpc Start(StartRequest) returns (StartResponse);
  // シグナル送信 (ワークフローへの入力イベント)
  rpc Signal(SignalRequest) returns (SignalResponse);
  // クエリ (ワークフロー状態の読取り、副作用なし)
  rpc Query(QueryRequest) returns (QueryResponse);
  // 正常終了の依頼 (キャンセル)
  rpc Cancel(CancelRequest) returns (CancelResponse);
  // 強制終了
  rpc Terminate(TerminateRequest) returns (TerminateResponse);
  // 状態取得
  rpc GetStatus(GetStatusRequest) returns (GetStatusResponse);
}

message StartRequest {
  // ワークフロー種別 (tier2 で登録されたコード名)
  string workflow_type = 1;
  // 実行 ID (指定なければ tier1 が UUID を生成)
  string workflow_id = 2;
  // 初期入力
  bytes input = 3;
  // 冪等性 (同一 workflow_id の重複開始は既存実行を返す)
  bool idempotent = 4;
  k1s0.tier1.common.v1.TenantContext context = 5;
}

message StartResponse {
  string workflow_id = 1;
  string run_id = 2;
}

message SignalRequest {
  string workflow_id = 1;
  string signal_name = 2;
  bytes payload = 3;
  k1s0.tier1.common.v1.TenantContext context = 4;
}

message SignalResponse {}

message QueryRequest {
  string workflow_id = 1;
  string query_name = 2;
  bytes payload = 3;
  k1s0.tier1.common.v1.TenantContext context = 4;
}

message QueryResponse {
  bytes result = 1;
}

message CancelRequest {
  string workflow_id = 1;
  string reason = 2;
  k1s0.tier1.common.v1.TenantContext context = 3;
}

message CancelResponse {}

message TerminateRequest {
  string workflow_id = 1;
  string reason = 2;
  k1s0.tier1.common.v1.TenantContext context = 3;
}

message TerminateResponse {}

message GetStatusRequest {
  string workflow_id = 1;
  k1s0.tier1.common.v1.TenantContext context = 2;
}

// 実行状態の列挙
enum WorkflowStatus {
  RUNNING = 0;
  COMPLETED = 1;
  FAILED = 2;
  CANCELED = 3;
  TERMINATED = 4;
  CONTINUED_AS_NEW = 5;
}

message GetStatusResponse {
  WorkflowStatus status = 1;
  string run_id = 2;
  // 完了時の出力 (status = COMPLETED の時のみ)
  bytes output = 3;
  // 失敗時のエラー詳細
  k1s0.tier1.common.v1.ErrorDetail error = 4;
}
```
