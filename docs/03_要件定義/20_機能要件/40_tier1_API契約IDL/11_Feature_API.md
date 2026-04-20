# 11. Feature API

Feature Flag 評価 API。flagd / OpenFeature 仕様に準拠し、Release（段階解放）/ Experiment（A/B テスト）/ Ops（kill switch）/ Permission（権限変更）の 4 種別を区別して扱う。従来の「コード内の `if ENABLE_X` 定数」を外部化し、デプロイなしに機能の有効化・無効化・割当比率変更を可能にする。

## 要件対応

- 要件ファイル: [../10_tier1_API要件/11_Feature_API.md](../10_tier1_API要件/11_Feature_API.md)
- 要件 ID: FR-T1-FEATURE-001〜004
- 共通型: [00_共通型定義.md](00_共通型定義.md)

## 設計のポイント

評価 API は値型ごとに 4 つの RPC（`EvaluateBoolean` / `EvaluateString` / `EvaluateNumber` / `EvaluateObject`）に分け、戻り値の型安全性を Protobuf レベルで担保する。`FlagMetadata.reason` は評価ロジックの透明性のため必須とし、監査や A/B 効果測定で「なぜこの variant が選ばれたか」を後追いできるようにする。Admin 側サービス `FeatureAdminService` では `PERMISSION` 種別 Flag の登録時に Product Council の `approval_id` を必須とし、未指定時は reject することで「権限昇格のフラグ化」による乗っ取りリスクを防ぐ。

## Protobuf 定義（評価）

```protobuf
// Feature API (FR-T1-FEATURE-001〜004)
syntax = "proto3";
package k1s0.tier1.feature.v1;
import "k1s0/tier1/common/v1/common.proto";

service FeatureService {
  // Boolean Flag 評価
  rpc EvaluateBoolean(EvaluateRequest) returns (BooleanResponse);
  // String Flag 評価 (Variant)
  rpc EvaluateString(EvaluateRequest) returns (StringResponse);
  // 数値 Flag 評価
  rpc EvaluateNumber(EvaluateRequest) returns (NumberResponse);
  // JSON オブジェクト Flag 評価
  rpc EvaluateObject(EvaluateRequest) returns (ObjectResponse);
}

// Flag 評価の共通入力
message EvaluateRequest {
  // Flag キー (命名規則: <tenant>.<component>.<feature>)
  string flag_key = 1;
  // 評価コンテキスト (targetingKey は subject と同一)
  map<string, string> evaluation_context = 2;
  k1s0.tier1.common.v1.TenantContext context = 3;
}

// Flag の種別 (OpenFeature / k1s0 固有)
enum FlagKind {
  RELEASE = 0;
  EXPERIMENT = 1;
  OPS = 2;
  PERMISSION = 3;
}

message FlagMetadata {
  FlagKind kind = 1;
  // バリアント名 (有効化理由の参考)
  string variant = 2;
  // 評価の理由 (DEFAULT / TARGETING_MATCH / SPLIT / ERROR)
  string reason = 3;
}

message BooleanResponse {
  bool value = 1;
  FlagMetadata metadata = 2;
}

message StringResponse {
  string value = 1;
  FlagMetadata metadata = 2;
}

message NumberResponse {
  double value = 1;
  FlagMetadata metadata = 2;
}

message ObjectResponse {
  bytes value_json = 1;
  FlagMetadata metadata = 2;
}
```

## Protobuf 定義（管理）

```protobuf
// Flag 定義の登録・更新 (Phase 1b 提供)
service FeatureAdminService {
  rpc RegisterFlag(RegisterFlagRequest) returns (RegisterFlagResponse);
  rpc GetFlag(GetFlagRequest) returns (GetFlagResponse);
  rpc ListFlags(ListFlagsRequest) returns (ListFlagsResponse);
}

// flagd 互換の Flag 定義 (k1s0 は OpenFeature / flagd 仕様に準拠)
message FlagDefinition {
  // Flag キー (命名規則: <tenant>.<component>.<feature>)
  string flag_key = 1;
  // Flag 種別 (RELEASE / EXPERIMENT / OPS / PERMISSION)
  FlagKind kind = 2;
  // 戻り値型 (boolean / string / number / object)
  FlagValueType value_type = 3;
  // デフォルト variant の名前 (下記 variants にキーが存在すること)
  string default_variant = 4;
  // variants 定義: variant 名 → 値 (value_type に応じた JSON literal)
  map<string, google.protobuf.Value> variants = 5;
  // targeting ルール (先頭から評価、最初に match したもの採用)
  repeated TargetingRule targeting = 6;
  // 状態 (ENABLED / DISABLED / ARCHIVED)
  FlagState state = 7;
  // 説明 (監査・運用者向け)
  string description = 8;
}

enum FlagValueType {
  FLAG_VALUE_UNSPECIFIED = 0;
  FLAG_VALUE_BOOLEAN = 1;
  FLAG_VALUE_STRING = 2;
  FLAG_VALUE_NUMBER = 3;
  FLAG_VALUE_OBJECT = 4;
}

enum FlagState {
  FLAG_STATE_UNSPECIFIED = 0;
  FLAG_STATE_ENABLED = 1;
  FLAG_STATE_DISABLED = 2;
  FLAG_STATE_ARCHIVED = 3;
}

// targeting ルール (JsonLogic 互換、flagd 仕様準拠)
// 例: { "if": [ { "==": [{ "var": "userRole" }, "admin"] }, "blue-variant", "red-variant" ] }
message TargetingRule {
  // ルール ID (監査用、tenant+flag 内で一意)
  string rule_id = 1;
  // JsonLogic 式 (bytes で保持、登録時に schema validator 通過必須)
  bytes json_logic_expr = 2;
  // 評価成立時に返す variant 名
  string variant_if_match = 3;
  // Fractional split (A/B テスト用、weights 合計 100 必須)
  repeated FractionalSplit fractional = 4;
}

// Experiment 種別の Flag で A/B 比率を指定
message FractionalSplit {
  string variant = 1;
  // 重み (0〜100、全エントリ合計 100 必須)
  int32 weight = 2;
}

message RegisterFlagRequest {
  FlagDefinition flag = 1;
  // 変更理由 (permission 種別 Flag の場合 Product Council 承認番号必須)
  string change_reason = 2;
  // permission 種別時の承認番号 (空値は permission 種別で reject)
  string approval_id = 3;
  k1s0.tier1.common.v1.TenantContext context = 4;
}

message RegisterFlagResponse {
  // バージョン (flag_key 内で単調増加)
  int64 version = 1;
}

message GetFlagRequest {
  string flag_key = 1;
  // バージョン (省略時は最新)
  optional int64 version = 2;
  k1s0.tier1.common.v1.TenantContext context = 3;
}

message GetFlagResponse {
  FlagDefinition flag = 1;
  int64 version = 2;
}

message ListFlagsRequest {
  // 種別フィルタ (省略で全種別)
  optional FlagKind kind = 1;
  // 状態フィルタ (省略で ENABLED のみ)
  optional FlagState state = 2;
  k1s0.tier1.common.v1.TenantContext context = 3;
}

message ListFlagsResponse {
  repeated FlagDefinition flags = 1;
}
```

## 4 種別の運用ルール

- **RELEASE**: 新機能の段階解放。variants は `{on, off}` が基本、targeting で canary → GA の段階拡大。廃止期限（sunset date）を必須項目とし、90 日超の放置は自動 ARCHIVED
- **EXPERIMENT**: A/B テスト。FractionalSplit を使用、最低 2 variants + 1 control。実験終了後は勝ち variant を default に昇格して ARCHIVED
- **OPS**: 運用 kill switch（例: 外部 API 連携を緊急遮断）。variants は `{enabled, disabled}` のみ、targeting は最小限
- **PERMISSION**: 権限変更（feature 可視性）。登録時に Product Council の approval_id 必須、未指定は reject。変更履歴は 7 年保管（NFR-E-MON-001 に準拠）
