---
id: arch.tier2.api_design_convention
axis: tier2
phase: architecture
kind: policy
status: published
version: 1.0.0
depends_on:
  - arch.tier2.tier2_index
  - detail.tier1.bidi_conformance
  - detail.tier1.schema_evolution_conformance
covered_by:
  defense_in_depth_layers: [A, B]
  proof_classes: []
---

# tier2 API 設計規約（wire 仕様）

## 一文方針
- tier2 公開 API（Library / Service）の wire 仕様は proto + Buf workflow + tier1 Bidi conformance_class + Idempotency-Key + Pagination + Long-running operation の 6 規約で固定。手書き API は CI fail。

## API バージョニング（wire 上の表現）
- proto package に `vN` を含める（例: `manufacturing.procurement.v1`）
- MAJOR up は新 package version、旧 package と並行提供（180 日 deprecation）
- MINOR / PATCH up は同 package version 内で purely additive

## エラーレスポンス形式
- gRPC Status code + Status detail（業務エラー subtype 含む）
- HTTP / JSON 経路は Connect Error spec（HTTP status + JSON body `code/message/details`）
- 業務エラーは [業務エラー監査コンプライアンス](08_業務エラー監査コンプライアンス.md) BusinessConflict subtype の 4 種（`stale_write` / `lost_update` / `supersede` / `concurrent_edit`）+ recovery_hints

## 冪等性（Idempotency）
- 全 mutation API に Idempotency-Key 必須（client SDK 自動採番、`x-idempotency-key` header）
- TTL: 24h（[client オフライン耐性方針](../09_client設計方針/03_オフライン耐性方針.md)）
- server 側は 32 atomic 三表書込で dedup
- chained idempotency_key（rebase 後の再送）は chain 親子関係で dedup 判定

## Pagination
- cursor-based pagination（offset-based は採用しない）
- request: `page_size` + `page_token`（empty で最初）
- response: `next_page_token`（empty で終端）
- field 命名は規約化（[スキーマ進化適合仕様](../../04_詳細設計/01_適合仕様/06_スキーマ進化適合仕様.md) `v1_external_proto` BACKWARD）

## Streaming
- bidi: tier1 Bidi conformance_class（`v1_interactive` / `v1_alert` / `v1_event_feed` / `v1_live_snapshot` / `v1_bulk_upload`）
- proto annotation `tier1.bidi.conformance_class` 必須
- External Proto は射影マップで unary / SSE に変換（External に bidi 直接公開は restricted）

## Long-running operation（LRO）
- 進捗を streaming で外に出さない
- 「Start RPC → cursor 付き GetStatus / Poll RPC」のペアで表現
- Temporal Workflow / Saga と同型
- timeout / cancellation は Workflow 側で制御

## リクエスト / レスポンス共通規約
- 全 RPC method に annotation 必須（`retry_policy_class` / `slo_class` / `quota_class` / `observability_class` / `capability_class`）
- 全 PII field に `field_pii` / `redaction_class` annotation
- field 命名: snake_case
- enum: `ENUM_NAME_VALUE` 形式

## deprecation
- proto file に `deprecated = true` annotation
- 新 RPC への移行 codemod を tier2 が提供
- deprecation 24 ヶ月後に削除可能（external proto は 24 ヶ月、internal proto は同期 deploy で即時可）

## wire 規約 YAML 宣言
- API デザインの規約（命名 / pagination / streaming / LRO）は YAML で宣言
- Buf custom lint が CI で違反検出
- `wire_convention.yaml` を tier2 リポジトリの単一の真として保持

## 採用しない設計
- offset-based pagination
- 進捗の streaming 配信
- annotation 不在の RPC method
- 手書き API stub
- proto を介さない wire 仕様

## 関連参照
- [tier2 設計方針 index](README.md)
- [Bidi 適合仕様](../../04_詳細設計/01_適合仕様/01_Bidi適合仕様.md)
- [スキーマ進化適合仕様](../../04_詳細設計/01_適合仕様/06_スキーマ進化適合仕様.md)
- [client コード生成方針](../09_client設計方針/08_コード生成方針.md)
