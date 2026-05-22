/**
 * index.ts — k1s0 tier1 Library TypeScript 公開 API エントリーポイント
 * 05_鍵管理適合仕様.md / 04_認証適合仕様.md / 17 機能カテゴリに基づく 4 言語等価強度 SDK の TypeScript 実装。
 * 生 key bytes / 生 access_token は公開 API シグネチャに一切露出しない。
 *
 * ============================================================
 * 3-layer split 規約（Y-bidi-library-split）
 * ============================================================
 * 本モジュールは Bidi 適合仕様 3-layer split 規約の「Layer 1: SDK（公開 API）」に該当する。
 *
 *   Layer 1: SDK（公開 API）— 本ファイル（index.ts）から re-export される型
 *     - tier2 / tier3 / client が消費する公開 API を提供する。
 *     - KeyHandle / AuthContext / ObservabilityProvider 等の型を再エクスポートする。
 *
 *   Layer 2: Internal（server 内部 API）— export しない型
 *     - tier1 server 実装のみが使う内部 API（index.ts から re-export しない）。
 *     - （将来） internal.ts に実装する。
 *
 *   Layer 3: ProtoBridge（Buf codegen 出力の薄い wrapper）— proto_bridge.ts
 *     - buf generate 出力の proto 型 → Library 型変換のみを担う。
 *     - proto_bridge.ts は index.ts から re-export しない（proto 型の公開 API 露出禁止）。
 */

// KeyClass enum + KeyHandle abstract class + StubKeyHandle class を再エクスポートする
export { KeyClass, KeyHandle, StubKeyHandle } from "./keyHandle.js";

// AuthClass enum + AuthContext class を再エクスポートする
export { AuthClass, AuthContext } from "./authContext.js";

// Repository<T> interface + RlsBypassError + verifyTenantId を再エクスポートする
export { Repository, RlsBypassError, verifyTenantId } from "./repository.js";

// Observability L3: Logger / Tracer / MetricMeter / ObservabilityProvider を再エクスポートする
export type {
  Attr,
  Logger,
  Span,
  SpanContext,
  Tracer,
  MetricMeter,
  ObservabilityProvider,
  LogEntry,
} from "./observability.js";
export { attr, Severity, SpanKind } from "./observability.js";

// Profiling L2*: Profiler / ContinuousProfiler を再エクスポートする
export type {
  ProfileOptions,
  ProfileSummary,
  ContinuousProfilerConfig,
  Profiler,
  ContinuousProfiler,
} from "./profiling.js";
export { ProfileType } from "./profiling.js";

// Configuration / Feature Flag L2*: FeatureFlagClient / ConfigClient を再エクスポートする
export type {
  EvalContext,
  BoolEvalResult,
  StringEvalResult,
  NumberEvalResult,
  ConfigValue,
  FeatureFlagClient,
  ConfigClient,
} from "./config.js";
export { FlagEvalReason } from "./config.js";

// KeyValue / Cache L3: CacheClient / CacheLock を再エクスポートする
export type {
  CacheTtl,
  CacheSetOptions,
  CacheClient,
  CacheLockOptions,
  CacheLock,
} from "./cache.js";

// Object Storage L3: ObjectStorageClient / MultipartObjectStorageClient を再エクスポートする
export type {
  StorageObjectMeta,
  StoragePutOptions,
  StorageGetOptions,
  StorageListOptions,
  StorageListResult,
  PresignedUrlOptions,
  ObjectStorageClient,
  MultipartUploadHandle,
  MultipartObjectStorageClient,
} from "./storage.js";

// RPC / Gateway L3: RpcUnaryClient / RpcStreamingClient / GatewayHandler を再エクスポートする
export type {
  RpcMetadata,
  RpcCallOptions,
  RpcStreamClient,
  RpcUnaryClient,
  RpcStreamingClient,
  GatewayRequest,
  GatewayResponse,
  GatewayHandler,
  GatewayMiddleware,
} from "./rpc.js";
export { RpcStatusCode, RpcError } from "./rpc.js";

// Messaging / EventBus L1+: MessagingProducer / MessagingConsumer / OutboxRelay を再エクスポートする
export type {
  OutboxMessage,
  DeliveredMessage,
  MessagingProduceResult,
  ConsumerGroupOptions,
  MessagingConsumerHandler,
  MessagingProducer,
  MessagingConsumer,
  OutboxRelay,
} from "./messaging.js";

// Schema Registry L2*: SchemaRegistryClient / SchemaCodec を再エクスポートする
export type {
  SchemaReference,
  SchemaInfo,
  SchemaRegistryClient,
  SchemaCodec,
} from "./schema.js";
export { SchemaFormat, CompatibilityMode } from "./schema.js";

// Relational Store / Single-leader L1+: DbClient / DbTx を再エクスポートする
export type {
  DbTxOptions,
  DbTx,
  DbClient,
  DbPoolStats,
} from "./db.js";
export { DbTxIsoLevel } from "./db.js";

// Relational Store / Distributed SQL L1+: DistributedDbClient / DistributedDbTx を再エクスポートする
export type {
  DistributedTxOptions,
  DistributedDbTx,
  DistributedDbClient,
  DistributedDbNodeInfo,
  BulkInsertOptions,
  DistributedBulkClient,
} from "./dbDistributed.js";
export { DistributedTxPriority } from "./dbDistributed.js";

// Vector Search L1+: VectorSearchClient を再エクスポートする
export type {
  Vector,
  VectorPoint,
  VectorSearchQuery,
  VectorSearchResult,
  HnswConfig,
  VectorCollectionConfig,
  VectorSearchClient,
} from "./vector.js";
export { VectorDistanceMetric, VectorIndexType } from "./vector.js";

// Workflow / Long-running Saga L1+: WorkflowClient / WorkflowRun を再エクスポートする
export type {
  WorkflowOptions,
  WorkflowExecution,
  WorkflowRun,
  WorkflowDescription,
  WorkflowClient,
} from "./workflow.js";
export { WorkflowStatus } from "./workflow.js";

// Rule Engine L1+: RuleEngineClient / CachedRuleEngineClient を再エクスポートする
export type {
  PolicyInput,
  PolicyResult,
  PolicyBundle,
  PolicyValidateOptions,
  RuleEngineClient,
  CachedRuleEngineClient,
  PolicyAuditEntry,
} from "./rules.js";
export { makePolicyPath } from "./rules.js";
export type { PolicyPath } from "./rules.js";

// Secret Management L3: SecretStore / SecretMetadata / SecretRotationPolicy を再エクスポートする
export type {
  SecretMetadata,
  SecretRotationPolicy,
  SecretStore,
} from "./secret.js";

// Observability 実装: ObservabilityProvider の facade 実装を再エクスポートする
export { createObservabilityProvider, StubLogger } from "./observabilityImpl.js";

// Messaging 実装: in-memory stub 実装を再エクスポートする（テスト用）
export {
  InMemoryMessagingProducer,
  InMemoryMessagingConsumer,
  NoopOutboxRelay,
} from "./messagingImpl.js";

// Messaging 実装: Kafka production 実装を再エクスポートする
export type {
  KafkaClientOptions,
  KafkaProducerOptions,
  KafkaConsumerOptions,
  KafkaOutboxRelayOptions,
} from "./messagingImpl.js";
export {
  KafkaMessagingProducer,
  KafkaMessagingConsumer,
  KafkaOutboxRelay,
} from "./messagingImpl.js";

// Secret 実装: in-memory stub 実装を再エクスポートする（テスト用）
export { InMemorySecretStore } from "./secretImpl.js";

// Conformance Assertion ID デコレータ: spec 01 §assertion id の連結（4 言語等価強度）
// ConformanceAssertId ブランド型 / assertId ヘルパー / conformanceAssert 関数を再エクスポートする
export type { ConformanceAssertId } from "./conformanceAssert.js";
export { assertId, conformanceAssert } from "./conformanceAssert.js";

// Frontend Transport Negotiation: Companion 役割 B の 8 adapter capability negotiation
export type {
  ClientCapabilities,
  BidiMessage,
  BidiChannel,
  TransportNegotiationClient,
  NegotiationResult,
} from "./frontend/transport_negotiation.js";
export {
  TransportKind,
  defaultClientCapabilities,
  InMemoryBidiChannel,
} from "./frontend/transport_negotiation.js";
