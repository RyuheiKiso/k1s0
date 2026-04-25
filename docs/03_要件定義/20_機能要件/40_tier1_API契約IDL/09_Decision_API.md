# 09. Decision API

ZEN Engine による JDM（JSON Decision Model）評価 API。ルール評価と結果の根拠（trace）を返し、tier2 側で Git 管理された JDM 文書を tier1 に登録して評価する。本 API は「ビジネスルールをコードから分離し、業務部門が編集可能な形式で保持する」という k1s0 の差別化価値の中核である。

## 要件対応

- 要件ファイル: [../10_tier1_API要件/09_Decision_API.md](../10_tier1_API要件/09_Decision_API.md)
- 要件 ID: FR-T1-DECISION-001〜004
- 共通型: [00_共通型定義.md](00_共通型定義.md)

## JDM（JSON Decision Model）スキーマ

tier2 が登録する JDM 文書は以下の JSON Schema（抜粋、機械可読な完全版は `proto/k1s0/tier1/decision/v1/jdm_schema.json` で管理）に従う。ZEN Engine v0.36 の JDM v1 仕様に準拠し、k1s0 独自の非決定要素禁止ルールを schema validator で強制する。

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://k1s0.jtc.local/schema/jdm/v1",
  "type": "object",
  "required": ["contentType", "nodes", "edges"],
  "properties": {
    "contentType": { "const": "application/vnd.gorules.decision" },
    "nodes": {
      "type": "array",
      "items": {
        "oneOf": [
          { "$ref": "#/$defs/inputNode" },
          { "$ref": "#/$defs/outputNode" },
          { "$ref": "#/$defs/decisionTableNode" },
          { "$ref": "#/$defs/expressionNode" },
          { "$ref": "#/$defs/functionNode" },
          { "$ref": "#/$defs/switchNode" }
        ]
      }
    },
    "edges": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["sourceId", "targetId"],
        "properties": {
          "sourceId": { "type": "string" },
          "targetId": { "type": "string" },
          "sourceHandle": { "type": "string" }
        }
      }
    }
  },
  "$defs": {
    "decisionTableNode": {
      "type": "object",
      "required": ["id", "type", "content"],
      "properties": {
        "type": { "const": "decisionTableNode" },
        "content": {
          "type": "object",
          "required": ["inputs", "outputs", "rules", "hitPolicy"],
          "properties": {
            "hitPolicy": { "enum": ["first", "collect"] },
            "inputs":  { "type": "array", "items": { "$ref": "#/$defs/columnDef" } },
            "outputs": { "type": "array", "items": { "$ref": "#/$defs/columnDef" } },
            "rules":   { "type": "array", "items": { "type": "object" } }
          }
        }
      }
    },
    "columnDef": {
      "type": "object",
      "required": ["id", "name", "field"],
      "properties": {
        "id":   { "type": "string" },
        "name": { "type": "string" },
        "field":{ "type": "string" }
      }
    }
  }
}
```

## JDM 非決定要素禁止ルール（k1s0 独自）

以下を含む JDM 文書は schema validator で reject する（NFR-I-SLO-009 Correctness 100% 担保のため）。

- `time.Now()` / `now()` / 現在時刻関数呼出し（代わりに評価時に `evaluation_context.now` で注入）
- `random()` / 乱数関数
- 外部 HTTP / DB アクセス（expression 内）
- 再帰深度 10 階層超（Decision graph の depth）
- 1 decision table の rule 数 10,000 超

これらは CI の JDM lint で検出し、違反は PR マージ不可とする。

## Protobuf 定義

```protobuf
// Decision API (FR-T1-DECISION-001〜004)
syntax = "proto3";
package k1s0.tier1.decision.v1;
import "k1s0/tier1/common/v1/common.proto";

service DecisionService {
  // ルール評価 (同期)
  rpc Evaluate(EvaluateRequest) returns (EvaluateResponse);
  // バッチ評価 (複数入力を一括評価)
  rpc BatchEvaluate(BatchEvaluateRequest) returns (BatchEvaluateResponse);
}

message EvaluateRequest {
  // ルール ID (tier2 で登録した JDM 文書の識別子)
  string rule_id = 1;
  // ルールバージョン (省略時は最新有効)
  string rule_version = 2;
  // 入力 (JDM の context に相当、任意 JSON)
  bytes input_json = 3;
  // trace 情報を返すか (デバッグ用、PII を含む可能性あり)
  bool include_trace = 4;
  k1s0.tier1.common.v1.TenantContext context = 5;
}

message EvaluateResponse {
  // 出力 (JDM 評価結果、任意 JSON)
  bytes output_json = 1;
  // 評価されたノードのトレース (include_trace=true の時のみ)
  bytes trace_json = 2;
  // 評価にかかった時間 (マイクロ秒)
  int64 elapsed_us = 3;
}

message BatchEvaluateRequest {
  string rule_id = 1;
  string rule_version = 2;
  repeated bytes inputs_json = 3;
  k1s0.tier1.common.v1.TenantContext context = 4;
}

message BatchEvaluateResponse {
  repeated bytes outputs_json = 1;
}

// JDM ルール文書の登録・バージョン管理 (リリース時点 で proto 追加予定)
service DecisionAdminService {
  // JDM 文書の登録 (schema validator と非決定要素 linter を通過必須)
  rpc RegisterRule(RegisterRuleRequest) returns (RegisterRuleResponse);
  // バージョン一覧
  rpc ListVersions(ListVersionsRequest) returns (ListVersionsResponse);
  // 特定バージョンの取得 (レビュー用)
  rpc GetRule(GetRuleRequest) returns (GetRuleResponse);
}

message RegisterRuleRequest {
  // ルール ID (tenant 内で一意)
  string rule_id = 1;
  // JDM 文書 (前節 JSON Schema に準拠、UTF-8 JSON)
  bytes jdm_document = 2;
  // Sigstore 署名 (ADR-RULE-001、registry に登録する署名)
  bytes sigstore_signature = 3;
  // コミット ID (Git commit hash、JDM バージョン追跡用)
  string commit_hash = 4;
  k1s0.tier1.common.v1.TenantContext context = 5;
}

message RegisterRuleResponse {
  // 採番されたバージョン (tenant + rule_id 内で一意、単調増加)
  string rule_version = 1;
  // 発効可能となる時刻 (即時なら registered_at と同じ)
  int64 effective_at_ms = 2;
}

message ListVersionsRequest {
  string rule_id = 1;
  k1s0.tier1.common.v1.TenantContext context = 2;
}

message ListVersionsResponse {
  repeated RuleVersionMeta versions = 1;
}

message RuleVersionMeta {
  string rule_version = 1;
  string commit_hash = 2;
  int64 registered_at_ms = 3;
  string registered_by = 4;
  // DEPRECATED 状態 (非推奨のみ true、廃止後は ListVersions から消える)
  bool deprecated = 5;
}

message GetRuleRequest {
  string rule_id = 1;
  string rule_version = 2;
  k1s0.tier1.common.v1.TenantContext context = 3;
}

message GetRuleResponse {
  bytes jdm_document = 1;
  RuleVersionMeta meta = 2;
}
```

## 実装方式（要件段階の合意事項）

- **IPC 方式**: Go ファサード（Dapr 経由）→ ZEN Engine (Rust) は **gRPC over Unix domain socket**（`unix:///var/run/k1s0/zen.sock`）で呼出し。FFI は Rust side の panic が Go プロセス全体を落とすリスクがあり採用しない
- **JDM バージョニング**: Git commit hash（rule_version として返却）。古いバージョンは 90 日間並走可能、その後は ListVersions から消える
- **決定論的性**: 同一 `(rule_version, evaluation_context)` は 100% 同一出力（NFR-I-SLO-009）。外部参照禁止 linter で担保
- **キャッシュ**: 評価結果は evaluation_context の SHA-256 を key とした in-memory LRU（1,000 エントリ、TTL 1 分）を Go 側で保持。Decision 評価自体は sub-ms だが、tier1 側での network/marshal コストを抑制
