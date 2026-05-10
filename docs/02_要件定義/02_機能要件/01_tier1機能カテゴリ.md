---
id: req.functional.tier1_functional
axis: tier1
phase: requirement
kind: requirement
status: draft
depends_on:
  - req.functional.functional_index
  - arch.tier1.feature_categories
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# tier1 機能カテゴリ要件

## 一文方針
- tier1 が提供する 17 機能カテゴリ × 3 抽象化レベル（L3 / L2\* / L1+）を要件として宣言する。詳細仕様は [tier1 提供機能カテゴリ](../../03_概要設計/02_tier1設計方針/04_提供機能カテゴリ.md) + 9 適合仕様 を参照。

## 17 機能カテゴリ要件

### Observability 系（4 カテゴリ、L3 + L2\*）
- Logging / Tracing / Metrics（L3、OpenTelemetry SDK + OTLP）
- Profiling（L2\*、Parca + Pyroscope の同族）

### Auth 系（2 カテゴリ、L3）
- Authentication / Authorization（L3、OIDC / OAuth 2.1、Keycloak）
- Secret Management（L3、OpenBao + External Secrets Operator）

### Configuration 系（1 カテゴリ、L2\*）
- Feature Flag（L2\*、OpenFeature provider 族）

### Storage / Cache（2 カテゴリ、L3）
- KeyValue / Cache（L3、RESP 族 = Valkey / Dragonfly / KeyDB）
- Object Storage（L3、S3 API 族 = Ceph RGW / MinIO / SeaweedFS）

### Communication（2 カテゴリ、L3 + L2\*）
- RPC / Gateway（L3 + transport-neutral）
- Schema Registry（L2\*、Confluent Schema Registry wire 互換族 = Apicurio / Karapace）

### Persistence / Workflow（5 カテゴリ、L1+）
- Messaging / EventBus（L1+、Apache Kafka）
- Relational Store / Single-leader（L1+、PostgreSQL）
- Vector Search（L1+、pgvector）
- Workflow / Long-running Saga（L1+、Temporal）
- Rule Engine（L1+、ZEN Engine）

### 将来カテゴリ（1 カテゴリ、L1+）
- Relational Store / Distributed SQL（v2 候補）

## 9 適合仕様要件
詳細は [04_詳細設計/01_適合仕様/](../../04_詳細設計/01_適合仕様/) 01〜09 を参照:
- Bidi 適合仕様（5 conformance_class、8 adapter）
- 移行 Pair 適合仕様（4 primary pair × 5 phase）
- 観測適合仕様（5 signal class × 4 dimension layer）
- 認証適合仕様（5 auth_class）
- 鍵管理適合仕様（5 key_class、KEK shamir M-of-N）
- スキーマ進化適合仕様（6 schema_class）
- SLO 適合仕様（6 slo_class、MWMBR alert）
- OSS ライフサイクル適合仕様（6 lifecycle_class、8 signal）
- テナント容量適合仕様（5 quota_class、4 階層 enforcement）

## 受入条件
- 17 機能カテゴリ全実装、L1+ 移行 toolchain dry-run green
- 9 適合仕様の release_gate cell 全 green
- Companion（役割 A / B）が 4 stack で動作

## 関連参照
- [機能要件 index](README.md)
- [tier1 提供機能カテゴリ](../../03_概要設計/02_tier1設計方針/04_提供機能カテゴリ.md)
- [tier1 設計方針 index](../../03_概要設計/02_tier1設計方針/README.md)
