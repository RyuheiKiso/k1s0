# tier1/library/go

k1s0 tier1 Library の Go 実装。4 言語等価強度 SDK の一部。

## 3-layer split 規約（Y-bidi-library-split）

Bidi 適合仕様 3-layer split 規約に従い、以下の 3 層に分離する。

### Layer 1: SDK（公開 API）

tier2 / tier3 / client が消費する公開 API を提供するパッケージ。

| パッケージ | 責務 |
|---|---|
| `auth_context/` | AuthClass enum + AuthContext struct |
| `keyhandle/` | KeyClass enum + KeyHandle interface |
| `cache/` | CacheClient interface |
| `config/` | FeatureFlagClient / ConfigClient interface |
| `db/` | DbClient / DbTx interface |
| `db_distributed/` | DistributedDbClient interface |
| `messaging/` | MessagingProducer / MessagingConsumer interface |
| `observability/` | Logger / Tracer / MetricMeter interface |
| `profiling/` | Profiler interface |
| `rpc/` | RpcClient / RpcServer interface |
| `rules/` | RuleEngineClient interface |
| `schema/` | SchemaRegistryClient interface |
| `secret/` | SecretStore interface |
| `storage/` | ObjectStorageClient interface |
| `vector/` | VectorSearchClient interface |
| `workflow/` | WorkflowClient interface |
| `frontend/` | TransportNegotiationClient / BidiChannel interface |
| `conformance/` | ConformanceAssertID / Assert() ヘルパー |
| `repository/` | Repository interface |

### Layer 2: Internal（server 内部 API）

tier1 server 実装のみが使う内部 API。  
Go の package internal 規則により、`library/go/internal/` 配下のパッケージは外部モジュールからインポート禁止。  
（将来: P10 フェーズで `internal/` パッケージを追加する）

### Layer 3: ProtoBridge（Buf codegen 出力の薄い wrapper）

`buf generate` 出力の protobuf 型 → Library 型への変換のみを担う。

| パッケージ | 責務 |
|---|---|
| `proto_bridge/` | BidiMessageProto / ResumeTokenProto（stub） |

proto 型を SDK 層（Layer 1）の公開 API シグネチャに露出禁止。  
P7（crosscutting）フェーズで Buf codegen と接続して完全実装に移行する。

---

設計方針の詳細は `docs/03_概要設計/02_tier1設計方針/README.md` を参照。
