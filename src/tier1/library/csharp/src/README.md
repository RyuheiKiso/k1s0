# tier1/library/csharp/src

k1s0 tier1 Library の C# 実装ソースディレクトリ。Bidi 適合仕様 3-layer split 規約に従い分離する。

## 3-layer split 規約（Y-bidi-library-split）

### Layer 1: SDK（公開 API）— `K1s0.Tier1` 名前空間

tier2 / tier3 / client が消費する公開 API を提供するファイル。

| ファイル | 責務 |
|---|---|
| `IKeyHandle.cs` / `KeyHandle.cs` | IKeyHandle interface + StubKeyHandle 実装 |
| `AuthContext.cs` | AuthClass enum + AuthContext struct |
| `ICache.cs` / `CacheImpl.cs` | ICacheClient interface + 実装 |
| `IFeatureFlag.cs` / `ConfigImpl.cs` | IFeatureFlagClient / IConfigClient interface + 実装 |
| `IRelationalStore.cs` / `DbImpl.cs` | IDbClient / IDbTx interface + 実装 |
| `IDistributedSql.cs` / `DbDistributedImpl.cs` | IDistributedDbClient interface + 実装 |
| `IMessaging.cs` / `MessagingImpl.cs` | IMessagingProducer / IMessagingConsumer interface + 実装 |
| `IObjectStorage.cs` / `ObjectStorageImpl.cs` | IObjectStorageClient interface + 実装 |
| `IObservability.cs` / `ObservabilityImpl.cs` | ILogger / ITracer / IMetricMeter interface + 実装 |
| `IProfiling.cs` / `ProfilingImpl.cs` | IProfiler interface + 実装 |
| `IRpc.cs` / `RpcImpl.cs` | IRpcClient / IRpcServer interface + 実装 |
| `IRuleEngine.cs` / `RuleEngineImpl.cs` | IRuleEngineClient interface + 実装 |
| `ISchemaRegistry.cs` / `SchemaRegistryImpl.cs` | ISchemaRegistryClient interface + 実装 |
| `ISecretStore.cs` / `SecretStoreImpl.cs` | ISecretStore interface + 実装 |
| `IVectorSearch.cs` / `VectorSearchImpl.cs` | IVectorSearchClient interface + 実装 |
| `IWorkflow.cs` / `WorkflowImpl.cs` | IWorkflowClient / IWorkflowRun interface + 実装 |
| `Repository.cs` | IRepository interface + 実装 |
| `Frontend/TransportNegotiation.cs` | ITransportNegotiationClient / IBidiChannel interface |
| `ConformanceAssertAttribute.cs` | \[ConformanceAssert\] 属性（4 言語等価強度） |

### Layer 2: Internal（server 内部 API）— `K1s0.Tier1.Internal` 名前空間

tier1 server 実装のみが使う内部 API（`internal` アクセス修飾子で保護）。  
（将来: P10 フェーズで `Internal/` サブディレクトリに実装する）

### Layer 3: ProtoBridge（Buf codegen 出力の薄い wrapper）— `K1s0.Tier1.ProtoBridge` 名前空間

`buf generate` 出力の protobuf 型 → Library 型への変換のみを担う。

| ファイル | 責務 |
|---|---|
| `ProtoBridge.cs` | BidiMessageProto / ResumeTokenProto（stub） |

proto 型を SDK 層（Layer 1）の公開 API シグネチャに露出禁止。  
P7（crosscutting）フェーズで Buf codegen と接続して完全実装に移行する。

---

設計方針の詳細は `docs/03_概要設計/02_tier1設計方針/README.md` を参照。
