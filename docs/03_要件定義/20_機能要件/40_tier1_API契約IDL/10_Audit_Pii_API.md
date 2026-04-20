# 10. Audit / Pii API

監査イベント記録 API と PII（個人識別情報）自動判定 API を 1 ファイルに束ねる。監査は WORM（Write Once Read Many）保管され 7 年間改竄不可、PII 判定は Decision API を内部的に利用して日本特有の PII（氏名・メール・電話・マイナンバー・クレカ番号等）を検出・マスクする。両者は異なる責務だが、「テナントアクティビティの可観測性と規制準拠」という共通の軸で AuditService / PiiService の 2 サービスに分割した上で 1 API 単位として扱う。

## 要件対応

- 要件ファイル: [../10_tier1_API要件/10_Audit_Pii_API.md](../10_tier1_API要件/10_Audit_Pii_API.md)
- 要件 ID: FR-T1-AUDIT-001〜003 / FR-T1-PII-001〜002
- 共通型: [00_共通型定義.md](00_共通型定義.md)

## 設計のポイント

`AuditEvent.action` は CRUD + LOGIN + EXPORT の 6 種類を基本とし、`resource` は URN 形式（`k1s0:tenant:<tid>:resource:<type>/<id>`）でリソース識別子を正規化する。これにより監査クエリ（特定ユーザ・特定リソース・特定期間）がフィルタ `attributes` 等価一致で横断的に機能する。`RecordAuditResponse.audit_id` は WORM ストアでの固有 ID であり、監査人がログの抽出経路を追跡できる。

`PiiService` の `Classify` は判定のみ、`Mask` は置換後テキストを返す分離をとる。`PiiFinding.confidence` は 0.0〜1.0 の信頼度で、運用側で閾値（既定 0.8）を設定して false positive を抑制する。検出種別（`NAME` / `EMAIL` / `PHONE` / `MYNUMBER` / `CREDITCARD` 等）は日本法令（個人情報保護法・マイナンバー法）で重要度が異なるため、詳細設計で enum 化しつつ、要件定義段階では拡張性のために `string` で保持する。

## Protobuf 定義

```protobuf
// Audit / Pii API (FR-T1-AUDIT-001〜003 / FR-T1-PII-001〜002)
syntax = "proto3";
package k1s0.tier1.audit.v1;
import "k1s0/tier1/common/v1/common.proto";
import "google/protobuf/timestamp.proto";

service AuditService {
  rpc Record(RecordAuditRequest) returns (RecordAuditResponse);
  rpc Query(QueryAuditRequest) returns (QueryAuditResponse);
}

service PiiService {
  rpc Classify(ClassifyRequest) returns (ClassifyResponse);
  rpc Mask(MaskRequest) returns (MaskResponse);
}

message AuditEvent {
  google.protobuf.Timestamp timestamp = 1;
  // 操作主体 (user_id / workload_id)
  string actor = 2;
  // 操作種別 (CREATE / READ / UPDATE / DELETE / LOGIN / EXPORT)
  string action = 3;
  // 対象リソース (URN 形式: k1s0:tenant:<tid>:resource:<type>/<id>)
  string resource = 4;
  // 操作結果 (SUCCESS / DENIED / ERROR)
  string outcome = 5;
  // 追加コンテキスト
  map<string, string> attributes = 6;
}

message RecordAuditRequest {
  AuditEvent event = 1;
  k1s0.tier1.common.v1.TenantContext context = 2;
}

message RecordAuditResponse {
  // WORM ストアでの固有 ID
  string audit_id = 1;
}

message QueryAuditRequest {
  // 範囲指定
  google.protobuf.Timestamp from = 1;
  google.protobuf.Timestamp to = 2;
  // フィルタ (任意の attributes 等価一致)
  map<string, string> filters = 3;
  int32 limit = 4;
  k1s0.tier1.common.v1.TenantContext context = 5;
}

message QueryAuditResponse {
  repeated AuditEvent events = 1;
}

message ClassifyRequest {
  // 判定対象テキスト
  string text = 1;
  k1s0.tier1.common.v1.TenantContext context = 2;
}

message PiiFinding {
  // 検出された PII 種別 (NAME / EMAIL / PHONE / MYNUMBER / CREDITCARD 等)
  string type = 1;
  // 文字列内の位置 (start, end)
  int32 start = 2;
  int32 end = 3;
  // 信頼度 (0.0〜1.0)
  double confidence = 4;
}

message ClassifyResponse {
  repeated PiiFinding findings = 1;
  // PII を含むか (findings が空でなければ true)
  bool contains_pii = 2;
}

message MaskRequest {
  string text = 1;
  k1s0.tier1.common.v1.TenantContext context = 2;
}

message MaskResponse {
  // マスク後のテキスト (氏名 → [NAME]、メール → [EMAIL])
  string masked_text = 1;
  repeated PiiFinding findings = 2;
}
```
