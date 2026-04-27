# infra/dapr/components — Dapr Component CRD 定義

tier1 公開 API（State / PubSub / Secrets / Binding 等）の backing store を Dapr Component として宣言する。`infra/dapr/control-plane/` の Operator が CRD として watcher し、`tier1-*` アプリ Pod の sidecar に配信する。

## 設計正典

- [`docs/05_実装/00_ディレクトリ設計/50_infraレイアウト/04_Dapr_Component配置.md`](../../../docs/05_実装/00_ディレクトリ設計/50_infraレイアウト/04_Dapr_Component配置.md) — IMP-DIR-INFRA-074
- [`docs/02_構想設計/adr/ADR-DATA-001-cnpg.md`](../../../docs/02_構想設計/adr/) / `ADR-DATA-002-kafka.md` / `ADR-DATA-003-minio.md` / `ADR-DATA-004-valkey.md`
- [`docs/02_構想設計/adr/ADR-SEC-001-secret-management.md`](../../../docs/02_構想設計/adr/) — OpenBao 採用根拠

## 同梱 Component（リリース時点）

| Component | type | 用途 |
|---|---|---|
| `state/postgres.yaml` | `state.postgresql` | tier1 State API の主要 backing store（CloudNativePG） |
| `state/redis-cache.yaml` | `state.valkey` | tier1 State API の短時間 TTL キャッシュ |
| `pubsub/kafka.yaml` | `pubsub.kafka` | tier1 PubSub API（Strimzi Kafka、mTLS） |
| `secrets/vault.yaml` | `secretstores.hashicorp.vault` | tier1 Secrets API（OpenBao、SPIRE 経由認証） |
| `binding/s3-inbound.yaml` | `bindings.aws.s3` | MinIO への input binding（zip / parquet 取込） |
| `binding/smtp-outbound.yaml` | `bindings.smtp` | テナント通知 / アラート配信 |
| `configuration/default.yaml` | `Configuration` | tracing / mTLS / accessControl の共通設定 |

## 同梱しないもの（採用後の運用拡大時）

- `workflow/temporal.yaml` — Workflow API の Temporal 統合（運用蓄積後）

## 環境差分の適用

各環境固有の設定（dev はネットワーク内 IP、prod は外部 endpoint 等）は本ディレクトリではなく [`infra/environments/<env>/dapr-components-overlay/`](../../environments/) で Kustomize patch として記述する（[`08_環境別パッチ配置.md`](../../../docs/05_実装/00_ディレクトリ設計/50_infraレイアウト/08_環境別パッチ配置.md)）。

リリース時点 では overlay は未配置（`infra/environments/<env>/` の README に「`infra/dapr/components/` 配置時に追記する設計」と明記済）。本 PR で base が揃ったため、採用初期 段階で overlay を追加する運びとなる。

## scopes と認証

各 Component の `scopes:` で参照可能な appId を絞り、`auth.secretStore: vault` で接続情報を OpenBao 経由に統一する。tier2 / tier3 アプリは tier1 facade を経由するため、本 Component は直接見えない（ADR-TIER1-001 / TIER1-003）。
