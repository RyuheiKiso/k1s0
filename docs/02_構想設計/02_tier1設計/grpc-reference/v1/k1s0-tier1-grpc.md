# Protocol Documentation

<a name="top"></a>

## Table of Contents

- [k1s0/tier1/common/v1/common.proto](#k1s0_tier1_common_v1_common-proto)
  - [ErrorDetail](#k1s0-tier1-common-v1-ErrorDetail)
  - [TenantContext](#k1s0-tier1-common-v1-TenantContext)
  
  - [K1s0ErrorCategory](#k1s0-tier1-common-v1-K1s0ErrorCategory)
  
- [k1s0/tier1/audit/v1/audit_service.proto](#k1s0_tier1_audit_v1_audit_service-proto)
  - [AuditEvent](#k1s0-tier1-audit-v1-AuditEvent)
  - [AuditEvent.AttributesEntry](#k1s0-tier1-audit-v1-AuditEvent-AttributesEntry)
  - [QueryAuditRequest](#k1s0-tier1-audit-v1-QueryAuditRequest)
  - [QueryAuditRequest.FiltersEntry](#k1s0-tier1-audit-v1-QueryAuditRequest-FiltersEntry)
  - [QueryAuditResponse](#k1s0-tier1-audit-v1-QueryAuditResponse)
  - [RecordAuditRequest](#k1s0-tier1-audit-v1-RecordAuditRequest)
  - [RecordAuditResponse](#k1s0-tier1-audit-v1-RecordAuditResponse)
  
  - [AuditService](#k1s0-tier1-audit-v1-AuditService)
  
- [k1s0/tier1/binding/v1/binding_service.proto](#k1s0_tier1_binding_v1_binding_service-proto)
  - [InvokeBindingRequest](#k1s0-tier1-binding-v1-InvokeBindingRequest)
  - [InvokeBindingRequest.MetadataEntry](#k1s0-tier1-binding-v1-InvokeBindingRequest-MetadataEntry)
  - [InvokeBindingResponse](#k1s0-tier1-binding-v1-InvokeBindingResponse)
  - [InvokeBindingResponse.MetadataEntry](#k1s0-tier1-binding-v1-InvokeBindingResponse-MetadataEntry)
  
  - [BindingService](#k1s0-tier1-binding-v1-BindingService)
  
- [k1s0/tier1/decision/v1/decision_service.proto](#k1s0_tier1_decision_v1_decision_service-proto)
  - [BatchEvaluateRequest](#k1s0-tier1-decision-v1-BatchEvaluateRequest)
  - [BatchEvaluateResponse](#k1s0-tier1-decision-v1-BatchEvaluateResponse)
  - [EvaluateRequest](#k1s0-tier1-decision-v1-EvaluateRequest)
  - [EvaluateResponse](#k1s0-tier1-decision-v1-EvaluateResponse)
  - [GetRuleRequest](#k1s0-tier1-decision-v1-GetRuleRequest)
  - [GetRuleResponse](#k1s0-tier1-decision-v1-GetRuleResponse)
  - [ListVersionsRequest](#k1s0-tier1-decision-v1-ListVersionsRequest)
  - [ListVersionsResponse](#k1s0-tier1-decision-v1-ListVersionsResponse)
  - [RegisterRuleRequest](#k1s0-tier1-decision-v1-RegisterRuleRequest)
  - [RegisterRuleResponse](#k1s0-tier1-decision-v1-RegisterRuleResponse)
  - [RuleVersionMeta](#k1s0-tier1-decision-v1-RuleVersionMeta)
  
  - [DecisionAdminService](#k1s0-tier1-decision-v1-DecisionAdminService)
  - [DecisionService](#k1s0-tier1-decision-v1-DecisionService)
  
- [k1s0/tier1/feature/v1/feature_service.proto](#k1s0_tier1_feature_v1_feature_service-proto)
  - [BooleanResponse](#k1s0-tier1-feature-v1-BooleanResponse)
  - [EvaluateRequest](#k1s0-tier1-feature-v1-EvaluateRequest)
  - [EvaluateRequest.EvaluationContextEntry](#k1s0-tier1-feature-v1-EvaluateRequest-EvaluationContextEntry)
  - [FlagDefinition](#k1s0-tier1-feature-v1-FlagDefinition)
  - [FlagDefinition.VariantsEntry](#k1s0-tier1-feature-v1-FlagDefinition-VariantsEntry)
  - [FlagMetadata](#k1s0-tier1-feature-v1-FlagMetadata)
  - [FractionalSplit](#k1s0-tier1-feature-v1-FractionalSplit)
  - [GetFlagRequest](#k1s0-tier1-feature-v1-GetFlagRequest)
  - [GetFlagResponse](#k1s0-tier1-feature-v1-GetFlagResponse)
  - [ListFlagsRequest](#k1s0-tier1-feature-v1-ListFlagsRequest)
  - [ListFlagsResponse](#k1s0-tier1-feature-v1-ListFlagsResponse)
  - [NumberResponse](#k1s0-tier1-feature-v1-NumberResponse)
  - [ObjectResponse](#k1s0-tier1-feature-v1-ObjectResponse)
  - [RegisterFlagRequest](#k1s0-tier1-feature-v1-RegisterFlagRequest)
  - [RegisterFlagResponse](#k1s0-tier1-feature-v1-RegisterFlagResponse)
  - [StringResponse](#k1s0-tier1-feature-v1-StringResponse)
  - [TargetingRule](#k1s0-tier1-feature-v1-TargetingRule)
  
  - [FlagKind](#k1s0-tier1-feature-v1-FlagKind)
  - [FlagState](#k1s0-tier1-feature-v1-FlagState)
  - [FlagValueType](#k1s0-tier1-feature-v1-FlagValueType)
  
  - [FeatureAdminService](#k1s0-tier1-feature-v1-FeatureAdminService)
  - [FeatureService](#k1s0-tier1-feature-v1-FeatureService)
  
- [k1s0/tier1/health/v1/health_service.proto](#k1s0_tier1_health_v1_health_service-proto)
  - [DependencyStatus](#k1s0-tier1-health-v1-DependencyStatus)
  - [LivenessRequest](#k1s0-tier1-health-v1-LivenessRequest)
  - [LivenessResponse](#k1s0-tier1-health-v1-LivenessResponse)
  - [ReadinessRequest](#k1s0-tier1-health-v1-ReadinessRequest)
  - [ReadinessResponse](#k1s0-tier1-health-v1-ReadinessResponse)
  - [ReadinessResponse.DependenciesEntry](#k1s0-tier1-health-v1-ReadinessResponse-DependenciesEntry)
  
  - [HealthService](#k1s0-tier1-health-v1-HealthService)
  
- [k1s0/tier1/log/v1/log_service.proto](#k1s0_tier1_log_v1_log_service-proto)
  - [BulkSendLogRequest](#k1s0-tier1-log-v1-BulkSendLogRequest)
  - [BulkSendLogResponse](#k1s0-tier1-log-v1-BulkSendLogResponse)
  - [LogEntry](#k1s0-tier1-log-v1-LogEntry)
  - [LogEntry.AttributesEntry](#k1s0-tier1-log-v1-LogEntry-AttributesEntry)
  - [SendLogRequest](#k1s0-tier1-log-v1-SendLogRequest)
  - [SendLogResponse](#k1s0-tier1-log-v1-SendLogResponse)
  
  - [Severity](#k1s0-tier1-log-v1-Severity)
  
  - [LogService](#k1s0-tier1-log-v1-LogService)
  
- [k1s0/tier1/pii/v1/pii_service.proto](#k1s0_tier1_pii_v1_pii_service-proto)
  - [ClassifyRequest](#k1s0-tier1-pii-v1-ClassifyRequest)
  - [ClassifyResponse](#k1s0-tier1-pii-v1-ClassifyResponse)
  - [MaskRequest](#k1s0-tier1-pii-v1-MaskRequest)
  - [MaskResponse](#k1s0-tier1-pii-v1-MaskResponse)
  - [PiiFinding](#k1s0-tier1-pii-v1-PiiFinding)
  
  - [PiiService](#k1s0-tier1-pii-v1-PiiService)
  
- [k1s0/tier1/pubsub/v1/pubsub_service.proto](#k1s0_tier1_pubsub_v1_pubsub_service-proto)
  - [BulkPublishEntry](#k1s0-tier1-pubsub-v1-BulkPublishEntry)
  - [BulkPublishRequest](#k1s0-tier1-pubsub-v1-BulkPublishRequest)
  - [BulkPublishResponse](#k1s0-tier1-pubsub-v1-BulkPublishResponse)
  - [Event](#k1s0-tier1-pubsub-v1-Event)
  - [Event.MetadataEntry](#k1s0-tier1-pubsub-v1-Event-MetadataEntry)
  - [PublishRequest](#k1s0-tier1-pubsub-v1-PublishRequest)
  - [PublishRequest.MetadataEntry](#k1s0-tier1-pubsub-v1-PublishRequest-MetadataEntry)
  - [PublishResponse](#k1s0-tier1-pubsub-v1-PublishResponse)
  - [SubscribeRequest](#k1s0-tier1-pubsub-v1-SubscribeRequest)
  
  - [PubSubService](#k1s0-tier1-pubsub-v1-PubSubService)
  
- [k1s0/tier1/secrets/v1/secrets_service.proto](#k1s0_tier1_secrets_v1_secrets_service-proto)
  - [BulkGetSecretRequest](#k1s0-tier1-secrets-v1-BulkGetSecretRequest)
  - [BulkGetSecretResponse](#k1s0-tier1-secrets-v1-BulkGetSecretResponse)
  - [BulkGetSecretResponse.ResultsEntry](#k1s0-tier1-secrets-v1-BulkGetSecretResponse-ResultsEntry)
  - [GetSecretRequest](#k1s0-tier1-secrets-v1-GetSecretRequest)
  - [GetSecretResponse](#k1s0-tier1-secrets-v1-GetSecretResponse)
  - [GetSecretResponse.ValuesEntry](#k1s0-tier1-secrets-v1-GetSecretResponse-ValuesEntry)
  - [RotateSecretRequest](#k1s0-tier1-secrets-v1-RotateSecretRequest)
  - [RotateSecretResponse](#k1s0-tier1-secrets-v1-RotateSecretResponse)
  
  - [SecretsService](#k1s0-tier1-secrets-v1-SecretsService)
  
- [k1s0/tier1/serviceinvoke/v1/serviceinvoke_service.proto](#k1s0_tier1_serviceinvoke_v1_serviceinvoke_service-proto)
  - [InvokeChunk](#k1s0-tier1-serviceinvoke-v1-InvokeChunk)
  - [InvokeRequest](#k1s0-tier1-serviceinvoke-v1-InvokeRequest)
  - [InvokeResponse](#k1s0-tier1-serviceinvoke-v1-InvokeResponse)
  
  - [InvokeService](#k1s0-tier1-serviceinvoke-v1-InvokeService)
  
- [k1s0/tier1/state/v1/state_service.proto](#k1s0_tier1_state_v1_state_service-proto)
  - [BulkGetRequest](#k1s0-tier1-state-v1-BulkGetRequest)
  - [BulkGetResponse](#k1s0-tier1-state-v1-BulkGetResponse)
  - [BulkGetResponse.ResultsEntry](#k1s0-tier1-state-v1-BulkGetResponse-ResultsEntry)
  - [DeleteRequest](#k1s0-tier1-state-v1-DeleteRequest)
  - [DeleteResponse](#k1s0-tier1-state-v1-DeleteResponse)
  - [GetRequest](#k1s0-tier1-state-v1-GetRequest)
  - [GetResponse](#k1s0-tier1-state-v1-GetResponse)
  - [SetRequest](#k1s0-tier1-state-v1-SetRequest)
  - [SetResponse](#k1s0-tier1-state-v1-SetResponse)
  - [TransactOp](#k1s0-tier1-state-v1-TransactOp)
  - [TransactRequest](#k1s0-tier1-state-v1-TransactRequest)
  - [TransactResponse](#k1s0-tier1-state-v1-TransactResponse)
  
  - [StateService](#k1s0-tier1-state-v1-StateService)
  
- [k1s0/tier1/telemetry/v1/telemetry_service.proto](#k1s0_tier1_telemetry_v1_telemetry_service-proto)
  - [EmitMetricRequest](#k1s0-tier1-telemetry-v1-EmitMetricRequest)
  - [EmitMetricResponse](#k1s0-tier1-telemetry-v1-EmitMetricResponse)
  - [EmitSpanRequest](#k1s0-tier1-telemetry-v1-EmitSpanRequest)
  - [EmitSpanResponse](#k1s0-tier1-telemetry-v1-EmitSpanResponse)
  - [Metric](#k1s0-tier1-telemetry-v1-Metric)
  - [Metric.LabelsEntry](#k1s0-tier1-telemetry-v1-Metric-LabelsEntry)
  - [Span](#k1s0-tier1-telemetry-v1-Span)
  - [Span.AttributesEntry](#k1s0-tier1-telemetry-v1-Span-AttributesEntry)
  
  - [MetricKind](#k1s0-tier1-telemetry-v1-MetricKind)
  
  - [TelemetryService](#k1s0-tier1-telemetry-v1-TelemetryService)
  
- [k1s0/tier1/workflow/v1/workflow_service.proto](#k1s0_tier1_workflow_v1_workflow_service-proto)
  - [CancelRequest](#k1s0-tier1-workflow-v1-CancelRequest)
  - [CancelResponse](#k1s0-tier1-workflow-v1-CancelResponse)
  - [GetStatusRequest](#k1s0-tier1-workflow-v1-GetStatusRequest)
  - [GetStatusResponse](#k1s0-tier1-workflow-v1-GetStatusResponse)
  - [QueryRequest](#k1s0-tier1-workflow-v1-QueryRequest)
  - [QueryResponse](#k1s0-tier1-workflow-v1-QueryResponse)
  - [SignalRequest](#k1s0-tier1-workflow-v1-SignalRequest)
  - [SignalResponse](#k1s0-tier1-workflow-v1-SignalResponse)
  - [StartRequest](#k1s0-tier1-workflow-v1-StartRequest)
  - [StartResponse](#k1s0-tier1-workflow-v1-StartResponse)
  - [TerminateRequest](#k1s0-tier1-workflow-v1-TerminateRequest)
  - [TerminateResponse](#k1s0-tier1-workflow-v1-TerminateResponse)
  
  - [WorkflowStatus](#k1s0-tier1-workflow-v1-WorkflowStatus)
  
  - [WorkflowService](#k1s0-tier1-workflow-v1-WorkflowService)
  
- [Scalar Value Types](#scalar-value-types)

<a name="k1s0_tier1_common_v1_common-proto"></a>
<p align="right"><a href="#top">Top</a></p>

## k1s0/tier1/common/v1/common.proto

本ファイルは tier1 公開 12 API すべてが import する共通型を定義する。
個別 API の proto からは `import &#34;k1s0/tier1/common/v1/common.proto&#34;;` で参照する。

定義範囲:

- TenantContext      : 全 RPC が伝搬するテナント識別コンテキスト
- ErrorDetail        : google.rpc.Status.details に埋め込むエラー詳細
- K1s0ErrorCategory  : 機械可読なエラーカテゴリ enum（10 値）

設計正典:
  docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/00_共通型定義.md
  docs/02_構想設計/adr/ADR-TIER1-002-protobuf-grpc.md

変更ルール: 既存フィールド削除は破壊的変更、追加は MINOR 版で許可。
proto 構文宣言（proto3）

<a name="k1s0-tier1-common-v1-ErrorDetail"></a>

### ErrorDetail

エラー詳細。tier1 は google.rpc.Status の `details` 配列に
本メッセージを 1 つ埋め込んで返す。`code` で詳細検索、
`category` で switch 分岐、`retryable` でクライアント側再試行判定。

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| code | [string](#string) |  | エラーコード（E-&lt;CATEGORY&gt;-&lt;MODULE&gt;-&lt;NUMBER&gt; 形式、文字列） |
| category | [K1s0ErrorCategory](#k1s0-tier1-common-v1-K1s0ErrorCategory) |  | 機械可読カテゴリ（switch 分岐用、enum 追加時は後方互換維持） |
| message | [string](#string) |  | 人間可読なメッセージ（テナント表示可、PII を含めてはならない） |
| retryable | [bool](#bool) |  | 再試行可否（true の場合クライアントは指数バックオフで再試行） |
| retry_after_ms | [int32](#int32) |  | 再試行までの推奨待機時間（ミリ秒、retryable=true の時のみ意味を持つ） |

<a name="k1s0-tier1-common-v1-TenantContext"></a>

### TenantContext

呼出元テナントを特定する識別子。gRPC メタデータヘッダ
（x-tenant-id / x-correlation-id / x-subject）と Request 側 message の
`context` フィールドの両方で伝搬し、interceptor で整合性を検証する。

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| tenant_id | [string](#string) |  | テナント ID（UUID v4 文字列、tier1 が JWT クレームと突き合わせて検証） |
| subject | [string](#string) |  | 呼出元の主体（workload_id または user_id、SPIFFE ID 互換） |
| correlation_id | [string](#string) |  | 相関 ID（OTel traceparent と紐付けて全 tier 横断トレース） |

<a name="k1s0-tier1-common-v1-K1s0ErrorCategory"></a>

### K1s0ErrorCategory

機械可読なエラーカテゴリ。docs `00_tier1_API共通規約.md` の 8 カテゴリ &#43;
UNSPECIFIED &#43; DEADLINE_EXCEEDED の計 10 値。
新規追加は MINOR 版で許可、既存値の削除・意味変更は禁止。
未知カテゴリを受け取った tier2/tier3 SDK は UNSPECIFIED として扱う。

| Name | Number | Description |
| ---- | ------ | ----------- |
| K1S0_ERROR_UNSPECIFIED | 0 | 未指定（既定値、クライアントは UNKNOWN として扱う） |
| K1S0_ERROR_INVALID_ARGUMENT | 1 | 呼出側入力誤り（リトライ不可） |
| K1S0_ERROR_UNAUTHENTICATED | 2 | JWT 不在・署名不正・期限切れ |
| K1S0_ERROR_PERMISSION_DENIED | 3 | RBAC 拒否・テナント越境・allowlist 外 |
| K1S0_ERROR_NOT_FOUND | 4 | キー / バージョン / リソース未存在 |
| K1S0_ERROR_CONFLICT | 5 | ETag 不一致・冪等性キー衝突 |
| K1S0_ERROR_RESOURCE_EXHAUSTED | 6 | レート制限・クォータ超過（retry_after_ms 必須） |
| K1S0_ERROR_UNAVAILABLE | 7 | 一時的バックエンド不能（retry_after_ms 必須、指数バックオフ） |
| K1S0_ERROR_INTERNAL | 8 | tier1 バグ・未分類（Audit に Severity 2 で記録） |
| K1S0_ERROR_DEADLINE_EXCEEDED | 9 | gRPC Deadline 超過（副作用未発生扱い、再試行可） |

<a name="k1s0_tier1_audit_v1_audit_service-proto"></a>
<p align="right"><a href="#top">Top</a></p>

## k1s0/tier1/audit/v1/audit_service.proto

本ファイルは tier1 公開 Audit API の正式 proto。
監査イベントの WORM ストア記録と検索を提供する。
PII API（Classify / Mask）は別ファイル（pii/v1/pii_service.proto）に分離。

設計正典:
  docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/10_Audit_Pii_API.md（AuditService 部）
  docs/03_要件定義/20_機能要件/10_tier1_API要件/10_Audit_Pii_API.md

関連要件: FR-T1-AUDIT-001〜003

注: 正典 IDL では AuditService と PiiService を 1 ファイル（package
    k1s0.tier1.audit.v1）にまとめているが、ディレクトリ設計（DS-DIR-*/
    IMP-DIR-*）と Pod 構成（t1-audit / t1-pii の 2 Pod 独立）に従い、
    本リポジトリでは 2 ファイル / 2 パッケージに分割する。
    RPC / message / フィールドは IDL と完全一致。
proto 構文宣言（proto3）

<a name="k1s0-tier1-audit-v1-AuditEvent"></a>

### AuditEvent

監査イベント

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| timestamp | [google.protobuf.Timestamp](#google-protobuf-Timestamp) |  | 発生時刻（UTC） |
| actor | [string](#string) |  | 操作主体（user_id / workload_id） |
| action | [string](#string) |  | 操作種別（CREATE / READ / UPDATE / DELETE / LOGIN / EXPORT） |
| resource | [string](#string) |  | 対象リソース（URN 形式: k1s0:tenant:&lt;tid&gt;:resource:&lt;type&gt;/&lt;id&gt;） |
| outcome | [string](#string) |  | 操作結果（SUCCESS / DENIED / ERROR） |
| attributes | [AuditEvent.AttributesEntry](#k1s0-tier1-audit-v1-AuditEvent-AttributesEntry) | repeated | 追加コンテキスト（ip / user_agent / request_id 等） |

<a name="k1s0-tier1-audit-v1-AuditEvent-AttributesEntry"></a>

### AuditEvent.AttributesEntry

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| key | [string](#string) |  |  |
| value | [string](#string) |  |  |

<a name="k1s0-tier1-audit-v1-QueryAuditRequest"></a>

### QueryAuditRequest

Query リクエスト

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| from | [google.protobuf.Timestamp](#google-protobuf-Timestamp) |  | 範囲開始 |
| to | [google.protobuf.Timestamp](#google-protobuf-Timestamp) |  | 範囲終了 |
| filters | [QueryAuditRequest.FiltersEntry](#k1s0-tier1-audit-v1-QueryAuditRequest-FiltersEntry) | repeated | フィルタ（任意の attributes 等価一致、AND 結合） |
| limit | [int32](#int32) |  | 件数上限（既定 100、最大 1000） |
| context | [k1s0.tier1.common.v1.TenantContext](#k1s0-tier1-common-v1-TenantContext) |  | 呼出元コンテキスト |

<a name="k1s0-tier1-audit-v1-QueryAuditRequest-FiltersEntry"></a>

### QueryAuditRequest.FiltersEntry

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| key | [string](#string) |  |  |
| value | [string](#string) |  |  |

<a name="k1s0-tier1-audit-v1-QueryAuditResponse"></a>

### QueryAuditResponse

Query 応答

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| events | [AuditEvent](#k1s0-tier1-audit-v1-AuditEvent) | repeated | 検索結果（時刻昇順、出力時に PII Mask 自動適用） |

<a name="k1s0-tier1-audit-v1-RecordAuditRequest"></a>

### RecordAuditRequest

Record リクエスト

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| event | [AuditEvent](#k1s0-tier1-audit-v1-AuditEvent) |  | 記録対象イベント |
| context | [k1s0.tier1.common.v1.TenantContext](#k1s0-tier1-common-v1-TenantContext) |  | 呼出元コンテキスト |

<a name="k1s0-tier1-audit-v1-RecordAuditResponse"></a>

### RecordAuditResponse

Record 応答

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| audit_id | [string](#string) |  | WORM ストアでの固有 ID（再現性のあるハッシュ含む） |

<a name="k1s0-tier1-audit-v1-AuditService"></a>

### AuditService

Audit API。WORM ストア（Postgres &#43; immutable view）に追記専用で記録する。

| Method Name | Request Type | Response Type | Description |
| ----------- | ------------ | ------------- | ------------|
| Record | [RecordAuditRequest](#k1s0-tier1-audit-v1-RecordAuditRequest) | [RecordAuditResponse](#k1s0-tier1-audit-v1-RecordAuditResponse) | 監査イベント記録（成功時は audit_id を返す、改竄不可） |
| Query | [QueryAuditRequest](#k1s0-tier1-audit-v1-QueryAuditRequest) | [QueryAuditResponse](#k1s0-tier1-audit-v1-QueryAuditResponse) | 監査イベント検索（範囲 &#43; フィルタ、出力には PII Mask が自動適用） |

<a name="k1s0_tier1_binding_v1_binding_service-proto"></a>
<p align="right"><a href="#top">Top</a></p>

## k1s0/tier1/binding/v1/binding_service.proto

本ファイルは tier1 公開 Binding API の正式 proto。
外部システム（HTTP / SMTP / S3 等）への出力バインディング呼出を提供する。

設計正典:
  docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/05_Binding_API.md
  docs/03_要件定義/20_機能要件/10_tier1_API要件/05_Binding_API.md

関連要件: FR-T1-BINDING-001〜004
proto 構文宣言（proto3）

<a name="k1s0-tier1-binding-v1-InvokeBindingRequest"></a>

### InvokeBindingRequest

Invoke リクエスト

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| name | [string](#string) |  | バインディング名（運用側で事前設定、例: s3-archive / smtp-notify） |
| operation | [string](#string) |  | 操作種別（create / get / list / delete / send 等、バインディング型依存） |
| data | [bytes](#bytes) |  | 操作データ本文 |
| metadata | [InvokeBindingRequest.MetadataEntry](#k1s0-tier1-binding-v1-InvokeBindingRequest-MetadataEntry) | repeated | メタデータ（content-type / to / subject 等、バインディング型依存） |
| context | [k1s0.tier1.common.v1.TenantContext](#k1s0-tier1-common-v1-TenantContext) |  | 呼出元コンテキスト |

<a name="k1s0-tier1-binding-v1-InvokeBindingRequest-MetadataEntry"></a>

### InvokeBindingRequest.MetadataEntry

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| key | [string](#string) |  |  |
| value | [string](#string) |  |  |

<a name="k1s0-tier1-binding-v1-InvokeBindingResponse"></a>

### InvokeBindingResponse

Invoke 応答

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| data | [bytes](#bytes) |  | 応答本文（操作種別とバインディング型に依存） |
| metadata | [InvokeBindingResponse.MetadataEntry](#k1s0-tier1-binding-v1-InvokeBindingResponse-MetadataEntry) | repeated | メタデータ（外部システムから返るヘッダ等） |

<a name="k1s0-tier1-binding-v1-InvokeBindingResponse-MetadataEntry"></a>

### InvokeBindingResponse.MetadataEntry

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| key | [string](#string) |  |  |
| value | [string](#string) |  |  |

<a name="k1s0-tier1-binding-v1-BindingService"></a>

### BindingService

Binding API。バインディング名は運用側で事前設定（s3-archive / smtp-notify 等）。

| Method Name | Request Type | Response Type | Description |
| ----------- | ------------ | ------------- | ------------|
| Invoke | [InvokeBindingRequest](#k1s0-tier1-binding-v1-InvokeBindingRequest) | [InvokeBindingResponse](#k1s0-tier1-binding-v1-InvokeBindingResponse) | 出力バインディング呼出（tier1 → 外部システムへ送信） |

<a name="k1s0_tier1_decision_v1_decision_service-proto"></a>
<p align="right"><a href="#top">Top</a></p>

## k1s0/tier1/decision/v1/decision_service.proto

本ファイルは tier1 公開 Decision API の正式 proto。
ZEN Engine による JDM（JSON Decision Model）ルール評価とルール文書管理を提供する。

設計正典:
  docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/09_Decision_API.md
  docs/03_要件定義/20_機能要件/10_tier1_API要件/09_Decision_API.md

関連要件: FR-T1-DECISION-001〜004
proto 構文宣言（proto3）

<a name="k1s0-tier1-decision-v1-BatchEvaluateRequest"></a>

### BatchEvaluateRequest

BatchEvaluate リクエスト

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| rule_id | [string](#string) |  | ルール ID（全入力で共通） |
| rule_version | [string](#string) |  | ルールバージョン |
| inputs_json | [bytes](#bytes) | repeated | 入力 JSON 列（順序を保って評価される） |
| context | [k1s0.tier1.common.v1.TenantContext](#k1s0-tier1-common-v1-TenantContext) |  | 呼出元コンテキスト |

<a name="k1s0-tier1-decision-v1-BatchEvaluateResponse"></a>

### BatchEvaluateResponse

BatchEvaluate 応答

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| outputs_json | [bytes](#bytes) | repeated | 出力 JSON 列（inputs_json と同じ順序） |

<a name="k1s0-tier1-decision-v1-EvaluateRequest"></a>

### EvaluateRequest

Evaluate リクエスト

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| rule_id | [string](#string) |  | ルール ID（tier2 で登録した JDM 文書の識別子） |
| rule_version | [string](#string) |  | ルールバージョン（省略時は最新有効） |
| input_json | [bytes](#bytes) |  | 入力（JDM の context に相当、任意 JSON） |
| include_trace | [bool](#bool) |  | trace 情報を返すか（デバッグ用、PII を含む可能性あり） |
| context | [k1s0.tier1.common.v1.TenantContext](#k1s0-tier1-common-v1-TenantContext) |  | 呼出元コンテキスト |

<a name="k1s0-tier1-decision-v1-EvaluateResponse"></a>

### EvaluateResponse

Evaluate 応答

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| output_json | [bytes](#bytes) |  | 出力（JDM 評価結果、任意 JSON） |
| trace_json | [bytes](#bytes) |  | 評価されたノードのトレース（include_trace=true の時のみ、空 bytes） |
| elapsed_us | [int64](#int64) |  | 評価にかかった時間（マイクロ秒） |

<a name="k1s0-tier1-decision-v1-GetRuleRequest"></a>

### GetRuleRequest

GetRule リクエスト

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| rule_id | [string](#string) |  | 対象ルール ID |
| rule_version | [string](#string) |  | 取得バージョン |
| context | [k1s0.tier1.common.v1.TenantContext](#k1s0-tier1-common-v1-TenantContext) |  | 呼出元コンテキスト |

<a name="k1s0-tier1-decision-v1-GetRuleResponse"></a>

### GetRuleResponse

GetRule 応答

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| jdm_document | [bytes](#bytes) |  | JDM 文書本体 |
| meta | [RuleVersionMeta](#k1s0-tier1-decision-v1-RuleVersionMeta) |  | メタ情報 |

<a name="k1s0-tier1-decision-v1-ListVersionsRequest"></a>

### ListVersionsRequest

ListVersions リクエスト

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| rule_id | [string](#string) |  | 対象ルール ID |
| context | [k1s0.tier1.common.v1.TenantContext](#k1s0-tier1-common-v1-TenantContext) |  | 呼出元コンテキスト |

<a name="k1s0-tier1-decision-v1-ListVersionsResponse"></a>

### ListVersionsResponse

ListVersions 応答

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| versions | [RuleVersionMeta](#k1s0-tier1-decision-v1-RuleVersionMeta) | repeated | バージョン一覧（登録時刻昇順） |

<a name="k1s0-tier1-decision-v1-RegisterRuleRequest"></a>

### RegisterRuleRequest

RegisterRule リクエスト

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| rule_id | [string](#string) |  | ルール ID（tenant 内で一意） |
| jdm_document | [bytes](#bytes) |  | JDM 文書（前節 JSON Schema に準拠、UTF-8 JSON） |
| sigstore_signature | [bytes](#bytes) |  | Sigstore 署名（ADR-RULE-001、registry に登録する署名） |
| commit_hash | [string](#string) |  | コミット ID（Git commit hash、JDM バージョン追跡用） |
| context | [k1s0.tier1.common.v1.TenantContext](#k1s0-tier1-common-v1-TenantContext) |  | 呼出元コンテキスト |

<a name="k1s0-tier1-decision-v1-RegisterRuleResponse"></a>

### RegisterRuleResponse

RegisterRule 応答

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| rule_version | [string](#string) |  | 採番されたバージョン（tenant &#43; rule_id 内で一意、単調増加） |
| effective_at_ms | [int64](#int64) |  | 発効可能となる時刻（即時なら registered_at と同じ、Unix epoch ミリ秒） |

<a name="k1s0-tier1-decision-v1-RuleVersionMeta"></a>

### RuleVersionMeta

ルールバージョンのメタ情報

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| rule_version | [string](#string) |  | バージョン文字列 |
| commit_hash | [string](#string) |  | Git commit hash |
| registered_at_ms | [int64](#int64) |  | 登録時刻（Unix epoch ミリ秒） |
| registered_by | [string](#string) |  | 登録者（subject 相当） |
| deprecated | [bool](#bool) |  | DEPRECATED 状態（非推奨のみ true、廃止後は ListVersions から消える） |

<a name="k1s0-tier1-decision-v1-DecisionAdminService"></a>

### DecisionAdminService

JDM ルール文書の登録・バージョン管理（リリース時点 で proto 追加予定）

| Method Name | Request Type | Response Type | Description |
| ----------- | ------------ | ------------- | ------------|
| RegisterRule | [RegisterRuleRequest](#k1s0-tier1-decision-v1-RegisterRuleRequest) | [RegisterRuleResponse](#k1s0-tier1-decision-v1-RegisterRuleResponse) | JDM 文書の登録（schema validator と非決定要素 linter を通過必須） |
| ListVersions | [ListVersionsRequest](#k1s0-tier1-decision-v1-ListVersionsRequest) | [ListVersionsResponse](#k1s0-tier1-decision-v1-ListVersionsResponse) | バージョン一覧 |
| GetRule | [GetRuleRequest](#k1s0-tier1-decision-v1-GetRuleRequest) | [GetRuleResponse](#k1s0-tier1-decision-v1-GetRuleResponse) | 特定バージョンの取得（レビュー用） |

<a name="k1s0-tier1-decision-v1-DecisionService"></a>

### DecisionService

Decision 評価 API。tier1 内の Rust 実装（ZEN Engine 統合）にディスパッチする。

| Method Name | Request Type | Response Type | Description |
| ----------- | ------------ | ------------- | ------------|
| Evaluate | [EvaluateRequest](#k1s0-tier1-decision-v1-EvaluateRequest) | [EvaluateResponse](#k1s0-tier1-decision-v1-EvaluateResponse) | ルール評価（同期、非決定要素を含むルールは登録時に弾かれる） |
| BatchEvaluate | [BatchEvaluateRequest](#k1s0-tier1-decision-v1-BatchEvaluateRequest) | [BatchEvaluateResponse](#k1s0-tier1-decision-v1-BatchEvaluateResponse) | バッチ評価（複数入力を一括評価、JIT 最適化対象） |

<a name="k1s0_tier1_feature_v1_feature_service-proto"></a>
<p align="right"><a href="#top">Top</a></p>

## k1s0/tier1/feature/v1/feature_service.proto

本ファイルは tier1 公開 Feature API の正式 proto。
flagd / OpenFeature 互換の Feature Flag 評価と管理を提供する。

設計正典:
  docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/11_Feature_API.md
  docs/03_要件定義/20_機能要件/10_tier1_API要件/11_Feature_API.md

関連要件: FR-T1-FEATURE-001〜004
proto 構文宣言（proto3）

<a name="k1s0-tier1-feature-v1-BooleanResponse"></a>

### BooleanResponse

Boolean 評価応答

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| value | [bool](#bool) |  | 評価値 |
| metadata | [FlagMetadata](#k1s0-tier1-feature-v1-FlagMetadata) |  | メタ情報 |

<a name="k1s0-tier1-feature-v1-EvaluateRequest"></a>

### EvaluateRequest

Flag 評価の共通入力

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| flag_key | [string](#string) |  | Flag キー（命名規則: &lt;tenant&gt;.&lt;component&gt;.&lt;feature&gt;） |
| evaluation_context | [EvaluateRequest.EvaluationContextEntry](#k1s0-tier1-feature-v1-EvaluateRequest-EvaluationContextEntry) | repeated | 評価コンテキスト（targetingKey は subject と同一） |
| context | [k1s0.tier1.common.v1.TenantContext](#k1s0-tier1-common-v1-TenantContext) |  | 呼出元コンテキスト |

<a name="k1s0-tier1-feature-v1-EvaluateRequest-EvaluationContextEntry"></a>

### EvaluateRequest.EvaluationContextEntry

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| key | [string](#string) |  |  |
| value | [string](#string) |  |  |

<a name="k1s0-tier1-feature-v1-FlagDefinition"></a>

### FlagDefinition

flagd 互換の Flag 定義。k1s0 は OpenFeature / flagd 仕様に準拠。

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| flag_key | [string](#string) |  | Flag キー（命名規則: &lt;tenant&gt;.&lt;component&gt;.&lt;feature&gt;） |
| kind | [FlagKind](#k1s0-tier1-feature-v1-FlagKind) |  | Flag 種別（RELEASE / EXPERIMENT / OPS / PERMISSION） |
| value_type | [FlagValueType](#k1s0-tier1-feature-v1-FlagValueType) |  | 戻り値型（boolean / string / number / object） |
| default_variant | [string](#string) |  | デフォルト variant の名前（下記 variants にキーが存在すること） |
| variants | [FlagDefinition.VariantsEntry](#k1s0-tier1-feature-v1-FlagDefinition-VariantsEntry) | repeated | variants 定義: variant 名 → 値（value_type に応じた JSON literal） |
| targeting | [TargetingRule](#k1s0-tier1-feature-v1-TargetingRule) | repeated | targeting ルール（先頭から評価、最初に match したもの採用） |
| state | [FlagState](#k1s0-tier1-feature-v1-FlagState) |  | 状態（ENABLED / DISABLED / ARCHIVED） |
| description | [string](#string) |  | 説明（監査・運用者向け） |

<a name="k1s0-tier1-feature-v1-FlagDefinition-VariantsEntry"></a>

### FlagDefinition.VariantsEntry

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| key | [string](#string) |  |  |
| value | [google.protobuf.Value](#google-protobuf-Value) |  |  |

<a name="k1s0-tier1-feature-v1-FlagMetadata"></a>

### FlagMetadata

Flag 評価のメタ情報（OpenFeature の EvaluationDetails と整合）

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| kind | [FlagKind](#k1s0-tier1-feature-v1-FlagKind) |  | Flag 種別 |
| variant | [string](#string) |  | バリアント名（有効化理由の参考） |
| reason | [string](#string) |  | 評価の理由（DEFAULT / TARGETING_MATCH / SPLIT / ERROR） |

<a name="k1s0-tier1-feature-v1-FractionalSplit"></a>

### FractionalSplit

Experiment 種別の Flag で A/B 比率を指定

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| variant | [string](#string) |  | バリアント名 |
| weight | [int32](#int32) |  | 重み（0〜100、全エントリ合計 100 必須） |

<a name="k1s0-tier1-feature-v1-GetFlagRequest"></a>

### GetFlagRequest

GetFlag リクエスト

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| flag_key | [string](#string) |  | 対象 Flag キー |
| version | [int64](#int64) | optional | バージョン（省略時は最新） |
| context | [k1s0.tier1.common.v1.TenantContext](#k1s0-tier1-common-v1-TenantContext) |  | 呼出元コンテキスト |

<a name="k1s0-tier1-feature-v1-GetFlagResponse"></a>

### GetFlagResponse

GetFlag 応答

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| flag | [FlagDefinition](#k1s0-tier1-feature-v1-FlagDefinition) |  | Flag 定義 |
| version | [int64](#int64) |  | バージョン |

<a name="k1s0-tier1-feature-v1-ListFlagsRequest"></a>

### ListFlagsRequest

ListFlags リクエスト

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| kind | [FlagKind](#k1s0-tier1-feature-v1-FlagKind) | optional | 種別フィルタ（省略で全種別） |
| state | [FlagState](#k1s0-tier1-feature-v1-FlagState) | optional | 状態フィルタ（省略で ENABLED のみ） |
| context | [k1s0.tier1.common.v1.TenantContext](#k1s0-tier1-common-v1-TenantContext) |  | 呼出元コンテキスト |

<a name="k1s0-tier1-feature-v1-ListFlagsResponse"></a>

### ListFlagsResponse

ListFlags 応答

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| flags | [FlagDefinition](#k1s0-tier1-feature-v1-FlagDefinition) | repeated | Flag 定義列 |

<a name="k1s0-tier1-feature-v1-NumberResponse"></a>

### NumberResponse

Number 評価応答

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| value | [double](#double) |  | 評価値 |
| metadata | [FlagMetadata](#k1s0-tier1-feature-v1-FlagMetadata) |  | メタ情報 |

<a name="k1s0-tier1-feature-v1-ObjectResponse"></a>

### ObjectResponse

Object 評価応答

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| value_json | [bytes](#bytes) |  | 評価値（JSON シリアライズ済み bytes） |
| metadata | [FlagMetadata](#k1s0-tier1-feature-v1-FlagMetadata) |  | メタ情報 |

<a name="k1s0-tier1-feature-v1-RegisterFlagRequest"></a>

### RegisterFlagRequest

RegisterFlag リクエスト

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| flag | [FlagDefinition](#k1s0-tier1-feature-v1-FlagDefinition) |  | Flag 定義 |
| change_reason | [string](#string) |  | 変更理由（permission 種別 Flag の場合 Product Council 承認番号必須） |
| approval_id | [string](#string) |  | permission 種別時の承認番号（空値は permission 種別で reject） |
| context | [k1s0.tier1.common.v1.TenantContext](#k1s0-tier1-common-v1-TenantContext) |  | 呼出元コンテキスト |

<a name="k1s0-tier1-feature-v1-RegisterFlagResponse"></a>

### RegisterFlagResponse

RegisterFlag 応答

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| version | [int64](#int64) |  | バージョン（flag_key 内で単調増加） |

<a name="k1s0-tier1-feature-v1-StringResponse"></a>

### StringResponse

String 評価応答

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| value | [string](#string) |  | 評価値 |
| metadata | [FlagMetadata](#k1s0-tier1-feature-v1-FlagMetadata) |  | メタ情報 |

<a name="k1s0-tier1-feature-v1-TargetingRule"></a>

### TargetingRule

targeting ルール（JsonLogic 互換、flagd 仕様準拠）。
例: { &#34;if&#34;: [ { &#34;==&#34;: [{ &#34;var&#34;: &#34;userRole&#34; }, &#34;admin&#34;] }, &#34;blue-variant&#34;, &#34;red-variant&#34; ] }

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| rule_id | [string](#string) |  | ルール ID（監査用、tenant&#43;flag 内で一意） |
| json_logic_expr | [bytes](#bytes) |  | JsonLogic 式（bytes で保持、登録時に schema validator 通過必須） |
| variant_if_match | [string](#string) |  | 評価成立時に返す variant 名 |
| fractional | [FractionalSplit](#k1s0-tier1-feature-v1-FractionalSplit) | repeated | Fractional split（A/B テスト用、weights 合計 100 必須） |

<a name="k1s0-tier1-feature-v1-FlagKind"></a>

### FlagKind

Flag の種別（OpenFeature / k1s0 固有）。
注: 正典 IDL は zero value を `RELEASE = 0` と定義しているため、
    buf STANDARD lint の ENUM_ZERO_VALUE_SUFFIX / ENUM_VALUE_PREFIX を ignore する。
buf:lint:ignore ENUM_ZERO_VALUE_SUFFIX
buf:lint:ignore ENUM_VALUE_PREFIX

| Name | Number | Description |
| ---- | ------ | ----------- |
| RELEASE | 0 | リリース管理用（既定値、コード経路の段階公開） |
| EXPERIMENT | 1 | A/B テスト等の実験用 |
| OPS | 2 | 運用上の緊急切替（Kill switch 含む） |
| PERMISSION | 3 | 権限制御（permission gate、Product Council 承認必須） |

<a name="k1s0-tier1-feature-v1-FlagState"></a>

### FlagState

Flag の状態

| Name | Number | Description |
| ---- | ------ | ----------- |
| FLAG_STATE_UNSPECIFIED | 0 | 未指定（既定値、登録時に弾かれる） |
| FLAG_STATE_ENABLED | 1 | 有効（評価可能） |
| FLAG_STATE_DISABLED | 2 | 無効（評価は default_variant 固定） |
| FLAG_STATE_ARCHIVED | 3 | 廃止（ListFlags から消える、Get は可能） |

<a name="k1s0-tier1-feature-v1-FlagValueType"></a>

### FlagValueType

Flag の戻り値型

| Name | Number | Description |
| ---- | ------ | ----------- |
| FLAG_VALUE_UNSPECIFIED | 0 | 未指定（既定値、登録時に弾かれる） |
| FLAG_VALUE_BOOLEAN | 1 | boolean 型 |
| FLAG_VALUE_STRING | 2 | string 型 |
| FLAG_VALUE_NUMBER | 3 | number 型 |
| FLAG_VALUE_OBJECT | 4 | object 型（任意 JSON） |

<a name="k1s0-tier1-feature-v1-FeatureAdminService"></a>

### FeatureAdminService

Flag 定義の登録・更新（リリース時点 提供）

| Method Name | Request Type | Response Type | Description |
| ----------- | ------------ | ------------- | ------------|
| RegisterFlag | [RegisterFlagRequest](#k1s0-tier1-feature-v1-RegisterFlagRequest) | [RegisterFlagResponse](#k1s0-tier1-feature-v1-RegisterFlagResponse) | Flag 定義の登録（permission 種別は approval_id 必須） |
| GetFlag | [GetFlagRequest](#k1s0-tier1-feature-v1-GetFlagRequest) | [GetFlagResponse](#k1s0-tier1-feature-v1-GetFlagResponse) | Flag 定義の取得 |
| ListFlags | [ListFlagsRequest](#k1s0-tier1-feature-v1-ListFlagsRequest) | [ListFlagsResponse](#k1s0-tier1-feature-v1-ListFlagsResponse) | Flag 定義の一覧 |

<a name="k1s0-tier1-feature-v1-FeatureService"></a>

### FeatureService

Feature Flag 評価 API。OpenFeature 互換、flagd 仕様準拠。

| Method Name | Request Type | Response Type | Description |
| ----------- | ------------ | ------------- | ------------|
| EvaluateBoolean | [EvaluateRequest](#k1s0-tier1-feature-v1-EvaluateRequest) | [BooleanResponse](#k1s0-tier1-feature-v1-BooleanResponse) | Boolean Flag 評価 |
| EvaluateString | [EvaluateRequest](#k1s0-tier1-feature-v1-EvaluateRequest) | [StringResponse](#k1s0-tier1-feature-v1-StringResponse) | String Flag 評価（Variant） |
| EvaluateNumber | [EvaluateRequest](#k1s0-tier1-feature-v1-EvaluateRequest) | [NumberResponse](#k1s0-tier1-feature-v1-NumberResponse) | 数値 Flag 評価 |
| EvaluateObject | [EvaluateRequest](#k1s0-tier1-feature-v1-EvaluateRequest) | [ObjectResponse](#k1s0-tier1-feature-v1-ObjectResponse) | JSON オブジェクト Flag 評価 |

<a name="k1s0_tier1_health_v1_health_service-proto"></a>
<p align="right"><a href="#top">Top</a></p>

## k1s0/tier1/health/v1/health_service.proto

本ファイルは tier1 ヘルスチェック用の補助 proto。
12 API 公開リスト（state / pubsub / serviceinvoke / secrets / binding /
workflow / log / telemetry / decision / audit / feature / pii）には **含まれない**
プロセス生存確認 / k8s probe 用 RPC で、Liveness / Readiness の 2 RPC のみ。

12 API 本体は plan 03-02 で `src/contracts/tier1/k1s0/tier1/&lt;api&gt;/v1/` 配下に展開する。
本ファイルは buf generate パイプラインの動作確認も兼ねる（plan 03-01 完了条件）。

設計:
  docs/03_要件定義/20_機能要件/02_機能一覧.md（12 API の正典）
  docs/02_構想設計/adr/ADR-TIER1-002-protobuf-grpc.md
  plan/03_Contracts実装/02_tier1_proto定義.md

<a name="k1s0-tier1-health-v1-DependencyStatus"></a>

### DependencyStatus

個別依存の状態

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| reachable | [bool](#bool) |  | 接続可能か |
| error_message | [string](#string) |  | 直近のエラー（reachable=false の時のみ意味を持つ） |

<a name="k1s0-tier1-health-v1-LivenessRequest"></a>

### LivenessRequest

Liveness probe のリクエスト（現状フィールドなし、将来拡張用）

<a name="k1s0-tier1-health-v1-LivenessResponse"></a>

### LivenessResponse

Liveness response: process の生存と version 情報を返す。

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| version | [string](#string) |  | tier1 facade のビルドバージョン（SemVer） |
| uptime_seconds | [int64](#int64) |  | 起動時刻からの経過時間（秒） |

<a name="k1s0-tier1-health-v1-ReadinessRequest"></a>

### ReadinessRequest

Readiness probe のリクエスト（現状フィールドなし、将来拡張用）

<a name="k1s0-tier1-health-v1-ReadinessResponse"></a>

### ReadinessResponse

Readiness response: 各依存の状態を個別に返す。

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| ready | [bool](#bool) |  | 全体としての ready 判定（依存すべて OK なら true） |
| dependencies | [ReadinessResponse.DependenciesEntry](#k1s0-tier1-health-v1-ReadinessResponse-DependenciesEntry) | repeated | 各依存（postgres / kafka / openbao / keycloak / 等）の個別状態 |

<a name="k1s0-tier1-health-v1-ReadinessResponse-DependenciesEntry"></a>

### ReadinessResponse.DependenciesEntry

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| key | [string](#string) |  |  |
| value | [DependencyStatus](#k1s0-tier1-health-v1-DependencyStatus) |  |  |

<a name="k1s0-tier1-health-v1-HealthService"></a>

### HealthService

ヘルスチェック用の最小 service。Kubernetes liveness/readiness probe からの
gRPC ヘルスチェックと、tier2/tier3 からの疎通確認に使う。

| Method Name | Request Type | Response Type | Description |
| ----------- | ------------ | ------------- | ------------|
| Liveness | [LivenessRequest](#k1s0-tier1-health-v1-LivenessRequest) | [LivenessResponse](#k1s0-tier1-health-v1-LivenessResponse) | Liveness probe: process が応答可能なら OK。依存 backend は見ない。 |
| Readiness | [ReadinessRequest](#k1s0-tier1-health-v1-ReadinessRequest) | [ReadinessResponse](#k1s0-tier1-health-v1-ReadinessResponse) | Readiness probe: 依存 backend（Postgres / Kafka / OpenBao 等）が到達可能 かどうかも含めて判定する。詳細仕様は plan 04-16。 |

<a name="k1s0_tier1_log_v1_log_service-proto"></a>
<p align="right"><a href="#top">Top</a></p>

## k1s0/tier1/log/v1/log_service.proto

本ファイルは tier1 公開 Log API の正式 proto。
構造化ログ送信（OpenTelemetry Logs 準拠）を提供する。

設計正典:
  docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/07_Log_API.md
  docs/03_要件定義/20_機能要件/10_tier1_API要件/07_Log_API.md

関連要件: FR-T1-LOG-001〜004
proto 構文宣言（proto3）

<a name="k1s0-tier1-log-v1-BulkSendLogRequest"></a>

### BulkSendLogRequest

BulkSend リクエスト

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| entries | [LogEntry](#k1s0-tier1-log-v1-LogEntry) | repeated | 送信エントリ列 |
| context | [k1s0.tier1.common.v1.TenantContext](#k1s0-tier1-common-v1-TenantContext) |  | 呼出元コンテキスト |

<a name="k1s0-tier1-log-v1-BulkSendLogResponse"></a>

### BulkSendLogResponse

BulkSend 応答

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| accepted | [int32](#int32) |  | 受理件数（OTel パイプラインに渡された件数） |
| rejected | [int32](#int32) |  | 拒否件数（PII フィルタや schema 違反で却下された件数） |

<a name="k1s0-tier1-log-v1-LogEntry"></a>

### LogEntry

LogEntry（OTel LogRecord と等価）

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| timestamp | [google.protobuf.Timestamp](#google-protobuf-Timestamp) |  | 発生時刻（UTC、tier2 側で OTel SDK 経由付与） |
| severity | [Severity](#k1s0-tier1-log-v1-Severity) |  | 重大度 |
| body | [string](#string) |  | メッセージ本文（PII 自動検出対象、Pii API で Mask 推奨） |
| attributes | [LogEntry.AttributesEntry](#k1s0-tier1-log-v1-LogEntry-AttributesEntry) | repeated | 属性（service.name / env / trace_id / span_id を含む） |
| stack_trace | [string](#string) |  | 関連する例外スタック（オプション、マルチライン許容） |

<a name="k1s0-tier1-log-v1-LogEntry-AttributesEntry"></a>

### LogEntry.AttributesEntry

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| key | [string](#string) |  |  |
| value | [string](#string) |  |  |

<a name="k1s0-tier1-log-v1-SendLogRequest"></a>

### SendLogRequest

Send リクエスト

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| entry | [LogEntry](#k1s0-tier1-log-v1-LogEntry) |  | 送信エントリ |
| context | [k1s0.tier1.common.v1.TenantContext](#k1s0-tier1-common-v1-TenantContext) |  | 呼出元コンテキスト |

<a name="k1s0-tier1-log-v1-SendLogResponse"></a>

### SendLogResponse

Send 応答

<a name="k1s0-tier1-log-v1-Severity"></a>

### Severity

重大度（OpenTelemetry Log Severity の数値仕様と整合）。
注: 数値タグは OTel 仕様（trace=0 / debug=5 / info=9 / warn=13 / error=17 / fatal=21）に
    固定されているため、buf STANDARD lint の ENUM_ZERO_VALUE_SUFFIX /
    ENUM_VALUE_PREFIX を ignore する。
buf:lint:ignore ENUM_ZERO_VALUE_SUFFIX
buf:lint:ignore ENUM_VALUE_PREFIX

| Name | Number | Description |
| ---- | ------ | ----------- |
| TRACE | 0 | OTel SeverityNumber TRACE（既定値） |
| DEBUG | 5 | OTel SeverityNumber DEBUG |
| INFO | 9 | OTel SeverityNumber INFO |
| WARN | 13 | OTel SeverityNumber WARN |
| ERROR | 17 | OTel SeverityNumber ERROR |
| FATAL | 21 | OTel SeverityNumber FATAL |

<a name="k1s0-tier1-log-v1-LogService"></a>

### LogService

Log API。本 API は OTel Logs パイプラインに直接乗せる（Loki / Grafana で参照）。

| Method Name | Request Type | Response Type | Description |
| ----------- | ------------ | ------------- | ------------|
| Send | [SendLogRequest](#k1s0-tier1-log-v1-SendLogRequest) | [SendLogResponse](#k1s0-tier1-log-v1-SendLogResponse) | 単一エントリ送信 |
| BulkSend | [BulkSendLogRequest](#k1s0-tier1-log-v1-BulkSendLogRequest) | [BulkSendLogResponse](#k1s0-tier1-log-v1-BulkSendLogResponse) | 一括送信（accepted / rejected で集計を返す） |

<a name="k1s0_tier1_pii_v1_pii_service-proto"></a>
<p align="right"><a href="#top">Top</a></p>

## k1s0/tier1/pii/v1/pii_service.proto

本ファイルは tier1 公開 PII API の正式 proto。
PII（個人情報）の検出（Classify）とマスキング（Mask）を提供する。

設計正典:
  docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/10_Audit_Pii_API.md（PiiService 部）
  docs/03_要件定義/20_機能要件/10_tier1_API要件/10_Audit_Pii_API.md

関連要件: FR-T1-PII-001〜002

担当 Pod: t1-pii（Rust 純関数実装、ステートレス、DS-SW-COMP-009）

注: 正典 IDL では AuditService と PiiService を 1 ファイル（package
    k1s0.tier1.audit.v1）にまとめているが、ディレクトリ設計と Pod 構成に従い、
    本リポジトリでは pii.v1 パッケージに分離する。
proto 構文宣言（proto3）

<a name="k1s0-tier1-pii-v1-ClassifyRequest"></a>

### ClassifyRequest

Classify リクエスト

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| text | [string](#string) |  | 判定対象テキスト |
| context | [k1s0.tier1.common.v1.TenantContext](#k1s0-tier1-common-v1-TenantContext) |  | 呼出元コンテキスト |

<a name="k1s0-tier1-pii-v1-ClassifyResponse"></a>

### ClassifyResponse

Classify 応答

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| findings | [PiiFinding](#k1s0-tier1-pii-v1-PiiFinding) | repeated | 検出された PII 一覧（位置順） |
| contains_pii | [bool](#bool) |  | PII を含むか（findings が空でなければ true） |

<a name="k1s0-tier1-pii-v1-MaskRequest"></a>

### MaskRequest

Mask リクエスト

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| text | [string](#string) |  | マスキング対象テキスト |
| context | [k1s0.tier1.common.v1.TenantContext](#k1s0-tier1-common-v1-TenantContext) |  | 呼出元コンテキスト |

<a name="k1s0-tier1-pii-v1-MaskResponse"></a>

### MaskResponse

Mask 応答

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| masked_text | [string](#string) |  | マスク後のテキスト（氏名 → [NAME]、メール → [EMAIL] 等） |
| findings | [PiiFinding](#k1s0-tier1-pii-v1-PiiFinding) | repeated | 検出された PII 一覧（マスキング前の位置情報） |

<a name="k1s0-tier1-pii-v1-PiiFinding"></a>

### PiiFinding

PII 検出結果の 1 件

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| type | [string](#string) |  | 検出された PII 種別（NAME / EMAIL / PHONE / MYNUMBER / CREDITCARD 等） |
| start | [int32](#int32) |  | 文字列内の開始位置（0 始まり、UTF-8 byte 単位ではなく文字単位） |
| end | [int32](#int32) |  | 文字列内の終了位置（exclusive） |
| confidence | [double](#double) |  | 信頼度（0.0〜1.0） |

<a name="k1s0-tier1-pii-v1-PiiService"></a>

### PiiService

PII API。t1-pii Pod は純関数（ステートレス）で副作用なし。

| Method Name | Request Type | Response Type | Description |
| ----------- | ------------ | ------------- | ------------|
| Classify | [ClassifyRequest](#k1s0-tier1-pii-v1-ClassifyRequest) | [ClassifyResponse](#k1s0-tier1-pii-v1-ClassifyResponse) | PII 種別の検出（テキスト → findings 列） |
| Mask | [MaskRequest](#k1s0-tier1-pii-v1-MaskRequest) | [MaskResponse](#k1s0-tier1-pii-v1-MaskResponse) | マスキング（テキスト → 置換後テキスト &#43; findings） |

<a name="k1s0_tier1_pubsub_v1_pubsub_service-proto"></a>
<p align="right"><a href="#top">Top</a></p>

## k1s0/tier1/pubsub/v1/pubsub_service.proto

本ファイルは tier1 公開 PubSub API の正式 proto。
Kafka 抽象 Publish / Subscribe を提供する（テナント境界はトピック接頭辞で自動隔離）。

設計正典:
  docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/03_PubSub_API.md
  docs/03_要件定義/20_機能要件/10_tier1_API要件/03_PubSub_API.md

関連要件: FR-T1-PUBSUB-001〜005
proto 構文宣言（proto3）

<a name="k1s0-tier1-pubsub-v1-BulkPublishEntry"></a>

### BulkPublishEntry

BulkPublish の個別エントリ結果

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| entry_index | [int32](#int32) |  | 入力 entries 配列内のインデックス（0 始まり） |
| offset | [int64](#int64) |  | Kafka 側のオフセット（成功時のみ意味を持つ） |
| error_code | [string](#string) |  | 失敗時のエラーコード（成功時は空文字列） |

<a name="k1s0-tier1-pubsub-v1-BulkPublishRequest"></a>

### BulkPublishRequest

BulkPublish リクエスト

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| topic | [string](#string) |  | トピック名（全エントリで共通） |
| entries | [PublishRequest](#k1s0-tier1-pubsub-v1-PublishRequest) | repeated | 公開するエントリ列 |

<a name="k1s0-tier1-pubsub-v1-BulkPublishResponse"></a>

### BulkPublishResponse

BulkPublish 応答

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| results | [BulkPublishEntry](#k1s0-tier1-pubsub-v1-BulkPublishEntry) | repeated | 各エントリの結果（失敗時は error_code に詳細） |

<a name="k1s0-tier1-pubsub-v1-Event"></a>

### Event

Event（Subscribe の stream 要素）

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| topic | [string](#string) |  | トピック名（接頭辞除去済みのテナント内名） |
| data | [bytes](#bytes) |  | イベント本文 |
| content_type | [string](#string) |  | Content-Type |
| offset | [int64](#int64) |  | Kafka 側のオフセット |
| metadata | [Event.MetadataEntry](#k1s0-tier1-pubsub-v1-Event-MetadataEntry) | repeated | メタデータ（Publish 時に付与されたヘッダがそのまま伝わる） |

<a name="k1s0-tier1-pubsub-v1-Event-MetadataEntry"></a>

### Event.MetadataEntry

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| key | [string](#string) |  |  |
| value | [string](#string) |  |  |

<a name="k1s0-tier1-pubsub-v1-PublishRequest"></a>

### PublishRequest

Publish リクエスト

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| topic | [string](#string) |  | トピック名（テナント接頭辞は tier1 が自動付与、クライアントはテナント内名のみ） |
| data | [bytes](#bytes) |  | イベント本文（bytes で透過、encoding は content_type で示す） |
| content_type | [string](#string) |  | Content-Type（application/json / application/protobuf 等） |
| idempotency_key | [string](#string) |  | 冪等性キー（重複 Publish を抑止、TTL 24h） |
| metadata | [PublishRequest.MetadataEntry](#k1s0-tier1-pubsub-v1-PublishRequest-MetadataEntry) | repeated | メタデータ（partition_key / trace_id 等の Kafka メッセージヘッダ相当） |
| context | [k1s0.tier1.common.v1.TenantContext](#k1s0-tier1-common-v1-TenantContext) |  | 呼出元コンテキスト |

<a name="k1s0-tier1-pubsub-v1-PublishRequest-MetadataEntry"></a>

### PublishRequest.MetadataEntry

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| key | [string](#string) |  |  |
| value | [string](#string) |  |  |

<a name="k1s0-tier1-pubsub-v1-PublishResponse"></a>

### PublishResponse

Publish 応答

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| offset | [int64](#int64) |  | Kafka 側のオフセット |

<a name="k1s0-tier1-pubsub-v1-SubscribeRequest"></a>

### SubscribeRequest

Subscribe リクエスト

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| topic | [string](#string) |  | トピック名 |
| consumer_group | [string](#string) |  | コンシューマグループ（テナント単位で分離） |
| context | [k1s0.tier1.common.v1.TenantContext](#k1s0-tier1-common-v1-TenantContext) |  | 呼出元コンテキスト |

<a name="k1s0-tier1-pubsub-v1-PubSubService"></a>

### PubSubService

PubSub API。Kafka をバックエンドとし、tier1 がテナント接頭辞付与と冪等性管理を行う。

| Method Name | Request Type | Response Type | Description |
| ----------- | ------------ | ------------- | ------------|
| Publish | [PublishRequest](#k1s0-tier1-pubsub-v1-PublishRequest) | [PublishResponse](#k1s0-tier1-pubsub-v1-PublishResponse) | 単発 Publish（idempotency_key で 24h 重複抑止） |
| BulkPublish | [BulkPublishRequest](#k1s0-tier1-pubsub-v1-BulkPublishRequest) | [BulkPublishResponse](#k1s0-tier1-pubsub-v1-BulkPublishResponse) | バッチ Publish（個別エントリの成否を BulkPublishEntry で返す） |
| Subscribe | [SubscribeRequest](#k1s0-tier1-pubsub-v1-SubscribeRequest) | [Event](#k1s0-tier1-pubsub-v1-Event) stream | サブスクリプション（tier2/tier3 側は HTTP コールバック登録 / gRPC ストリームのいずれか） |

<a name="k1s0_tier1_secrets_v1_secrets_service-proto"></a>
<p align="right"><a href="#top">Top</a></p>

## k1s0/tier1/secrets/v1/secrets_service.proto

本ファイルは tier1 公開 Secrets API の正式 proto。
OpenBao 経由でテナントスコープのシークレットを取得・ローテーションする。

設計正典:
  docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/04_Secrets_API.md
  docs/03_要件定義/20_機能要件/10_tier1_API要件/04_Secrets_API.md

関連要件: FR-T1-SECRETS-001〜004
proto 構文宣言（proto3）

<a name="k1s0-tier1-secrets-v1-BulkGetSecretRequest"></a>

### BulkGetSecretRequest

BulkGet リクエスト

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| context | [k1s0.tier1.common.v1.TenantContext](#k1s0-tier1-common-v1-TenantContext) |  | 呼出元コンテキスト |

<a name="k1s0-tier1-secrets-v1-BulkGetSecretResponse"></a>

### BulkGetSecretResponse

BulkGet 応答

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| results | [BulkGetSecretResponse.ResultsEntry](#k1s0-tier1-secrets-v1-BulkGetSecretResponse-ResultsEntry) | repeated | 結果マップ（シークレット名 → GetSecretResponse） |

<a name="k1s0-tier1-secrets-v1-BulkGetSecretResponse-ResultsEntry"></a>

### BulkGetSecretResponse.ResultsEntry

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| key | [string](#string) |  |  |
| value | [GetSecretResponse](#k1s0-tier1-secrets-v1-GetSecretResponse) |  |  |

<a name="k1s0-tier1-secrets-v1-GetSecretRequest"></a>

### GetSecretRequest

Get リクエスト

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| name | [string](#string) |  | シークレット名（テナント境界を超えた参照は即 PermissionDenied） |
| context | [k1s0.tier1.common.v1.TenantContext](#k1s0-tier1-common-v1-TenantContext) |  | 呼出元コンテキスト |
| version | [int32](#int32) | optional | 省略時は最新、明示で旧バージョン取得可（grace_period 中のみ） |

<a name="k1s0-tier1-secrets-v1-GetSecretResponse"></a>

### GetSecretResponse

Get 応答

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| values | [GetSecretResponse.ValuesEntry](#k1s0-tier1-secrets-v1-GetSecretResponse-ValuesEntry) | repeated | 値（Base64 エンコード必要時はクライアント側で判断、複数キーの key=value マップ） |
| version | [int32](#int32) |  | バージョン（ローテーション追跡用） |

<a name="k1s0-tier1-secrets-v1-GetSecretResponse-ValuesEntry"></a>

### GetSecretResponse.ValuesEntry

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| key | [string](#string) |  |  |
| value | [string](#string) |  |  |

<a name="k1s0-tier1-secrets-v1-RotateSecretRequest"></a>

### RotateSecretRequest

Rotate リクエスト

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| name | [string](#string) |  | ローテーション対象シークレット名 |
| context | [k1s0.tier1.common.v1.TenantContext](#k1s0-tier1-common-v1-TenantContext) |  | 呼出元コンテキスト |
| grace_period_sec | [int32](#int32) |  | 旧バージョンの猶予時間（0 は即無効、既定 3600 秒） tier2 側の接続プール drain 時間を想定 |
| policy | [string](#string) | optional | 動的シークレット（DB 資格情報等）の場合の発行ポリシー名 |
| idempotency_key | [string](#string) |  | 冪等性キー（同一キーでの再試行は同じ new_version を返す） |

<a name="k1s0-tier1-secrets-v1-RotateSecretResponse"></a>

### RotateSecretResponse

Rotate 応答

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| new_version | [int32](#int32) |  | ローテーション後の新バージョン |
| previous_version | [int32](#int32) |  | 旧バージョン（grace_period_sec まで Get 可能） |
| rotated_at_ms | [int64](#int64) |  | 新バージョン発効時刻（Unix epoch ミリ秒） |
| ttl_sec | [int32](#int32) |  | 動的シークレット時の TTL（静的シークレットでは 0） |

<a name="k1s0-tier1-secrets-v1-SecretsService"></a>

### SecretsService

Secrets API。OpenBao をバックエンドとし、tier1 が PII / アクセス制御を強制する。

| Method Name | Request Type | Response Type | Description |
| ----------- | ------------ | ------------- | ------------|
| Get | [GetSecretRequest](#k1s0-tier1-secrets-v1-GetSecretRequest) | [GetSecretResponse](#k1s0-tier1-secrets-v1-GetSecretResponse) | 単一シークレット取得（テナント越境参照は即 PermissionDenied） |
| BulkGet | [BulkGetSecretRequest](#k1s0-tier1-secrets-v1-BulkGetSecretRequest) | [BulkGetSecretResponse](#k1s0-tier1-secrets-v1-BulkGetSecretResponse) | 一括取得（テナントに割当された全シークレット） |
| Rotate | [RotateSecretRequest](#k1s0-tier1-secrets-v1-RotateSecretRequest) | [RotateSecretResponse](#k1s0-tier1-secrets-v1-RotateSecretResponse) | ローテーション実行（FR-T1-SECRETS-004） 成功時は new_version を返し、旧バージョンは grace_period_sec まで Get 可能。 失敗時は K1s0Error を返し OpenBao 側は不変（トランザクショナル）。 |

<a name="k1s0_tier1_serviceinvoke_v1_serviceinvoke_service-proto"></a>
<p align="right"><a href="#top">Top</a></p>

## k1s0/tier1/serviceinvoke/v1/serviceinvoke_service.proto

本ファイルは tier1 公開 Service Invoke API の正式 proto。
サービス間呼出を仲介する RPC を提供する（Dapr の app-to-app invoke 概念に対応）。

設計正典:
  docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/01_Service_Invoke_API.md
  docs/03_要件定義/20_機能要件/10_tier1_API要件/01_Service_Invoke_API.md

関連要件: FR-T1-INVOKE-001〜005

注: 正典 IDL では package を `k1s0.tier1.invoke.v1` と記載しているが、
    ディレクトリ設計（DS-DIR-*/ IMP-DIR-*）と SDK 生成パスが
    `serviceinvoke` で統一されているため、buf STANDARD lint の
    PACKAGE_DIRECTORY_MATCH を満たすために本パッケージは
    `k1s0.tier1.serviceinvoke.v1` とする。RPC / message / フィールドは
    IDL 正典と完全一致させる。
proto 構文宣言（proto3）

<a name="k1s0-tier1-serviceinvoke-v1-InvokeChunk"></a>

### InvokeChunk

ストリーム応答のチャンク

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| data | [bytes](#bytes) |  | チャンク本文 |
| eof | [bool](#bool) |  | ストリーム終端フラグ（true の場合は本チャンクが最終） |

<a name="k1s0-tier1-serviceinvoke-v1-InvokeRequest"></a>

### InvokeRequest

Invoke リクエスト

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| app_id | [string](#string) |  | 呼出先のアプリ識別子（Dapr app_id 互換、tier2 のサービス名に相当） |
| method | [string](#string) |  | 呼出先のメソッド名（HTTP の場合は path に相当） |
| data | [bytes](#bytes) |  | 呼出データ（bytes で透過伝搬、encoding は content_type で示す） |
| content_type | [string](#string) |  | Content-Type（application/json / application/grpc / application/protobuf 等） |
| context | [k1s0.tier1.common.v1.TenantContext](#k1s0-tier1-common-v1-TenantContext) |  | 呼出元コンテキスト（テナント識別と相関 ID） |
| timeout_ms | [int32](#int32) |  | タイムアウト（ミリ秒、省略時は 5000ms） |

<a name="k1s0-tier1-serviceinvoke-v1-InvokeResponse"></a>

### InvokeResponse

Invoke 応答

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| data | [bytes](#bytes) |  | 応答データ（bytes で透過伝搬、encoding は content_type で示す） |
| content_type | [string](#string) |  | Content-Type（呼出先が決定） |
| status | [int32](#int32) |  | HTTP ステータス相当（成功 200、失敗時は詳細を Status に載せる） |

<a name="k1s0-tier1-serviceinvoke-v1-InvokeService"></a>

### InvokeService

サービス間呼出を仲介する。tier1 が allowlist / RBAC / 監査を一括強制する。

| Method Name | Request Type | Response Type | Description |
| ----------- | ------------ | ------------- | ------------|
| Invoke | [InvokeRequest](#k1s0-tier1-serviceinvoke-v1-InvokeRequest) | [InvokeResponse](#k1s0-tier1-serviceinvoke-v1-InvokeResponse) | 任意サービスの任意メソッドを呼び出す（app_id は Dapr の app_id 概念と互換） |
| InvokeStream | [InvokeRequest](#k1s0-tier1-serviceinvoke-v1-InvokeRequest) | [InvokeChunk](#k1s0-tier1-serviceinvoke-v1-InvokeChunk) stream | ストリーミング呼出（大容量応答や段階出力のため、サーバ → クライアントの単方向ストリーム） |

<a name="k1s0_tier1_state_v1_state_service-proto"></a>
<p align="right"><a href="#top">Top</a></p>

## k1s0/tier1/state/v1/state_service.proto

本ファイルは tier1 公開 State API の正式 proto。
KV / Relational / Document 状態管理（楽観的排他とトランザクション境界付き）を提供する。

設計正典:
  docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/02_State_API.md
  docs/03_要件定義/20_機能要件/10_tier1_API要件/02_State_API.md

関連要件: FR-T1-STATE-001〜005
proto 構文宣言（proto3）

<a name="k1s0-tier1-state-v1-BulkGetRequest"></a>

### BulkGetRequest

BulkGet リクエスト

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| store | [string](#string) |  | Store 名（全キーで共通） |
| keys | [string](#string) | repeated | 取得するキー一覧 |
| context | [k1s0.tier1.common.v1.TenantContext](#k1s0-tier1-common-v1-TenantContext) |  | 呼出元コンテキスト |

<a name="k1s0-tier1-state-v1-BulkGetResponse"></a>

### BulkGetResponse

BulkGet 応答

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| results | [BulkGetResponse.ResultsEntry](#k1s0-tier1-state-v1-BulkGetResponse-ResultsEntry) | repeated | 結果マップ（キー → GetResponse、未存在キーも not_found=true で含める） |

<a name="k1s0-tier1-state-v1-BulkGetResponse-ResultsEntry"></a>

### BulkGetResponse.ResultsEntry

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| key | [string](#string) |  |  |
| value | [GetResponse](#k1s0-tier1-state-v1-GetResponse) |  |  |

<a name="k1s0-tier1-state-v1-DeleteRequest"></a>

### DeleteRequest

Delete リクエスト

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| store | [string](#string) |  | Store 名 |
| key | [string](#string) |  | キー |
| expected_etag | [string](#string) |  | 期待 ETag（空は無条件削除、指定時は楽観的排他で削除） |
| context | [k1s0.tier1.common.v1.TenantContext](#k1s0-tier1-common-v1-TenantContext) |  | 呼出元コンテキスト |

<a name="k1s0-tier1-state-v1-DeleteResponse"></a>

### DeleteResponse

Delete 応答

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| deleted | [bool](#bool) |  | 削除実行可否（未存在キーへの削除も deleted=true で返す） |

<a name="k1s0-tier1-state-v1-GetRequest"></a>

### GetRequest

Get リクエスト

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| store | [string](#string) |  | Store 名（valkey-default / postgres-tenant 等、運用側で設定） |
| key | [string](#string) |  | キー（テナント境界は tier1 が自動付与、クライアントはテナント内キーのみ指定） |
| context | [k1s0.tier1.common.v1.TenantContext](#k1s0-tier1-common-v1-TenantContext) |  | 呼出元コンテキスト |

<a name="k1s0-tier1-state-v1-GetResponse"></a>

### GetResponse

Get 応答

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| data | [bytes](#bytes) |  | 値本文（bytes で透過、encoding はクライアント責務） |
| etag | [string](#string) |  | 楽観的排他のための ETag（Set / Delete 時に expected_etag に再送する） |
| not_found | [bool](#bool) |  | キー未存在時は true（このとき data / etag は空、エラーではない） |

<a name="k1s0-tier1-state-v1-SetRequest"></a>

### SetRequest

Set リクエスト

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| store | [string](#string) |  | Store 名 |
| key | [string](#string) |  | キー |
| data | [bytes](#bytes) |  | 保存値本文 |
| expected_etag | [string](#string) |  | 期待 ETag（空は未存在前提、新規作成時は空文字列） |
| ttl_sec | [int32](#int32) |  | TTL（秒、0 は永続） |
| context | [k1s0.tier1.common.v1.TenantContext](#k1s0-tier1-common-v1-TenantContext) |  | 呼出元コンテキスト |

<a name="k1s0-tier1-state-v1-SetResponse"></a>

### SetResponse

Set 応答

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| new_etag | [string](#string) |  | 保存後の新 ETag（次回 Set / Delete 時の expected_etag に渡す） |

<a name="k1s0-tier1-state-v1-TransactOp"></a>

### TransactOp

トランザクション内の 1 操作（Set / Delete のいずれか）

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| set | [SetRequest](#k1s0-tier1-state-v1-SetRequest) |  | Set 操作 |
| delete | [DeleteRequest](#k1s0-tier1-state-v1-DeleteRequest) |  | Delete 操作 |

<a name="k1s0-tier1-state-v1-TransactRequest"></a>

### TransactRequest

Transact リクエスト

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| store | [string](#string) |  | Store 名（全操作で共通、複数 Store を跨ぐトランザクションは不可） |
| operations | [TransactOp](#k1s0-tier1-state-v1-TransactOp) | repeated | 操作列（記述順に実行、途中失敗で全ロールバック） |
| context | [k1s0.tier1.common.v1.TenantContext](#k1s0-tier1-common-v1-TenantContext) |  | 呼出元コンテキスト |

<a name="k1s0-tier1-state-v1-TransactResponse"></a>

### TransactResponse

Transact 応答

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| committed | [bool](#bool) |  | コミット成功可否（false の場合は全ロールバック済み） |

<a name="k1s0-tier1-state-v1-StateService"></a>

### StateService

状態管理 API。Store 名で valkey / postgres / minio 等のバックエンドを切り替える。

| Method Name | Request Type | Response Type | Description |
| ----------- | ------------ | ------------- | ------------|
| Get | [GetRequest](#k1s0-tier1-state-v1-GetRequest) | [GetResponse](#k1s0-tier1-state-v1-GetResponse) | キー単位の取得（未存在時は not_found=true、エラーには非該当） |
| Set | [SetRequest](#k1s0-tier1-state-v1-SetRequest) | [SetResponse](#k1s0-tier1-state-v1-SetResponse) | キー単位の保存（ETag 不一致時は FAILED_PRECONDITION でエラー） |
| Delete | [DeleteRequest](#k1s0-tier1-state-v1-DeleteRequest) | [DeleteResponse](#k1s0-tier1-state-v1-DeleteResponse) | キー単位の削除（ETag を expected_etag に渡せば楽観的排他で削除） |
| BulkGet | [BulkGetRequest](#k1s0-tier1-state-v1-BulkGetRequest) | [BulkGetResponse](#k1s0-tier1-state-v1-BulkGetResponse) | 複数キーの一括取得（部分的な未存在は not_found=true で表現、エラーにしない） |
| Transact | [TransactRequest](#k1s0-tier1-state-v1-TransactRequest) | [TransactResponse](#k1s0-tier1-state-v1-TransactResponse) | トランザクション境界付きの複数操作（全 Store で対応するわけではない） |

<a name="k1s0_tier1_telemetry_v1_telemetry_service-proto"></a>
<p align="right"><a href="#top">Top</a></p>

## k1s0/tier1/telemetry/v1/telemetry_service.proto

本ファイルは tier1 公開 Telemetry API の正式 proto。
メトリクス（Counter / Gauge / Histogram）と分散トレース Span 送信を提供する。

設計正典:
  docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/08_Telemetry_API.md
  docs/03_要件定義/20_機能要件/10_tier1_API要件/08_Telemetry_API.md

関連要件: FR-T1-TELEMETRY-001〜004
proto 構文宣言（proto3）

<a name="k1s0-tier1-telemetry-v1-EmitMetricRequest"></a>

### EmitMetricRequest

EmitMetric リクエスト

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| metrics | [Metric](#k1s0-tier1-telemetry-v1-Metric) | repeated | メトリクス列 |
| context | [k1s0.tier1.common.v1.TenantContext](#k1s0-tier1-common-v1-TenantContext) |  | 呼出元コンテキスト |

<a name="k1s0-tier1-telemetry-v1-EmitMetricResponse"></a>

### EmitMetricResponse

EmitMetric 応答

<a name="k1s0-tier1-telemetry-v1-EmitSpanRequest"></a>

### EmitSpanRequest

EmitSpan リクエスト

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| spans | [Span](#k1s0-tier1-telemetry-v1-Span) | repeated | Span 列 |
| context | [k1s0.tier1.common.v1.TenantContext](#k1s0-tier1-common-v1-TenantContext) |  | 呼出元コンテキスト |

<a name="k1s0-tier1-telemetry-v1-EmitSpanResponse"></a>

### EmitSpanResponse

EmitSpan 応答

<a name="k1s0-tier1-telemetry-v1-Metric"></a>

### Metric

単一メトリクス

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| name | [string](#string) |  | メトリクス名（OTel 慣行に従いドット区切り、例: k1s0.tier1.invoke.duration_ms） |
| kind | [MetricKind](#k1s0-tier1-telemetry-v1-MetricKind) |  | メトリクス種別 |
| value | [double](#double) |  | 値（Counter は加算、Gauge は瞬時値、Histogram は観測値） |
| labels | [Metric.LabelsEntry](#k1s0-tier1-telemetry-v1-Metric-LabelsEntry) | repeated | ラベル（service.name / env / status_code 等の OTel attribute） |
| timestamp | [google.protobuf.Timestamp](#google-protobuf-Timestamp) |  | タイムスタンプ（観測時刻、UTC） |

<a name="k1s0-tier1-telemetry-v1-Metric-LabelsEntry"></a>

### Metric.LabelsEntry

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| key | [string](#string) |  |  |
| value | [string](#string) |  |  |

<a name="k1s0-tier1-telemetry-v1-Span"></a>

### Span

単一 Span

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| trace_id | [string](#string) |  | トレース ID（W3C Trace Context、16 byte 相当の 32 文字 hex） |
| span_id | [string](#string) |  | Span ID（8 byte 相当の 16 文字 hex） |
| parent_span_id | [string](#string) |  | 親 Span ID（ルート Span は空文字列） |
| name | [string](#string) |  | Span 名（操作名、例: HTTP GET /api/v1/foo） |
| start_time | [google.protobuf.Timestamp](#google-protobuf-Timestamp) |  | 開始時刻 |
| end_time | [google.protobuf.Timestamp](#google-protobuf-Timestamp) |  | 終了時刻 |
| attributes | [Span.AttributesEntry](#k1s0-tier1-telemetry-v1-Span-AttributesEntry) | repeated | 属性（http.method / db.system 等の OTel semantic conventions） |

<a name="k1s0-tier1-telemetry-v1-Span-AttributesEntry"></a>

### Span.AttributesEntry

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| key | [string](#string) |  |  |
| value | [string](#string) |  |  |

<a name="k1s0-tier1-telemetry-v1-MetricKind"></a>

### MetricKind

メトリクス種別。
注: 正典 IDL は zero value を `COUNTER = 0` と定義しているため、
    buf STANDARD lint の ENUM_ZERO_VALUE_SUFFIX / ENUM_VALUE_PREFIX を ignore する。
buf:lint:ignore ENUM_ZERO_VALUE_SUFFIX
buf:lint:ignore ENUM_VALUE_PREFIX

| Name | Number | Description |
| ---- | ------ | ----------- |
| COUNTER | 0 | 単調増加カウンタ（Prometheus _total メトリクスに対応、既定値） |
| GAUGE | 1 | 上下する瞬時値ゲージ |
| HISTOGRAM | 2 | 分布ヒストグラム（quantile / bucket 計算用） |

<a name="k1s0-tier1-telemetry-v1-TelemetryService"></a>

### TelemetryService

Telemetry API。OTel Collector → Mimir / Tempo に転送する経路で使う。

| Method Name | Request Type | Response Type | Description |
| ----------- | ------------ | ------------- | ------------|
| EmitMetric | [EmitMetricRequest](#k1s0-tier1-telemetry-v1-EmitMetricRequest) | [EmitMetricResponse](#k1s0-tier1-telemetry-v1-EmitMetricResponse) | メトリクス送信（Counter / Gauge / Histogram の混在可） |
| EmitSpan | [EmitSpanRequest](#k1s0-tier1-telemetry-v1-EmitSpanRequest) | [EmitSpanResponse](#k1s0-tier1-telemetry-v1-EmitSpanResponse) | Span 送信（既に終了済みの Span のみ受け付ける、開始 Span は OTel SDK で） |

<a name="k1s0_tier1_workflow_v1_workflow_service-proto"></a>
<p align="right"><a href="#top">Top</a></p>

## k1s0/tier1/workflow/v1/workflow_service.proto

本ファイルは tier1 公開 Workflow API の正式 proto。
短期 Dapr Workflow / 長期 Temporal の双方を抽象化した Workflow ライフサイクル API。

設計正典:
  docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/06_Workflow_API.md
  docs/03_要件定義/20_機能要件/10_tier1_API要件/06_Workflow_API.md

関連要件: FR-T1-WORKFLOW-001〜005
proto 構文宣言（proto3）

<a name="k1s0-tier1-workflow-v1-CancelRequest"></a>

### CancelRequest

Cancel リクエスト

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| workflow_id | [string](#string) |  | 対象ワークフロー ID |
| reason | [string](#string) |  | キャンセル理由（監査用、自由記述） |
| context | [k1s0.tier1.common.v1.TenantContext](#k1s0-tier1-common-v1-TenantContext) |  | 呼出元コンテキスト |

<a name="k1s0-tier1-workflow-v1-CancelResponse"></a>

### CancelResponse

Cancel 応答

<a name="k1s0-tier1-workflow-v1-GetStatusRequest"></a>

### GetStatusRequest

GetStatus リクエスト

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| workflow_id | [string](#string) |  | 対象ワークフロー ID |
| context | [k1s0.tier1.common.v1.TenantContext](#k1s0-tier1-common-v1-TenantContext) |  | 呼出元コンテキスト |

<a name="k1s0-tier1-workflow-v1-GetStatusResponse"></a>

### GetStatusResponse

GetStatus 応答

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| status | [WorkflowStatus](#k1s0-tier1-workflow-v1-WorkflowStatus) |  | 現在状態 |
| run_id | [string](#string) |  | 直近の実行 ID（Continue-as-New 後は新 run_id を返す） |
| output | [bytes](#bytes) |  | 完了時の出力（status = COMPLETED の時のみ） |
| error | [k1s0.tier1.common.v1.ErrorDetail](#k1s0-tier1-common-v1-ErrorDetail) |  | 失敗時のエラー詳細（status = FAILED の時のみ） |

<a name="k1s0-tier1-workflow-v1-QueryRequest"></a>

### QueryRequest

Query リクエスト

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| workflow_id | [string](#string) |  | 対象ワークフロー ID |
| query_name | [string](#string) |  | クエリ名（tier2 ワークフロー実装が register しているクエリハンドラ名） |
| payload | [bytes](#bytes) |  | ペイロード |
| context | [k1s0.tier1.common.v1.TenantContext](#k1s0-tier1-common-v1-TenantContext) |  | 呼出元コンテキスト |

<a name="k1s0-tier1-workflow-v1-QueryResponse"></a>

### QueryResponse

Query 応答

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| result | [bytes](#bytes) |  | クエリ結果 |

<a name="k1s0-tier1-workflow-v1-SignalRequest"></a>

### SignalRequest

Signal リクエスト

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| workflow_id | [string](#string) |  | 対象ワークフロー ID |
| signal_name | [string](#string) |  | シグナル名（tier2 ワークフロー実装が listen している名前） |
| payload | [bytes](#bytes) |  | ペイロード（任意 bytes） |
| context | [k1s0.tier1.common.v1.TenantContext](#k1s0-tier1-common-v1-TenantContext) |  | 呼出元コンテキスト |

<a name="k1s0-tier1-workflow-v1-SignalResponse"></a>

### SignalResponse

Signal 応答（フィールド無し、成功は OK gRPC ステータスで表現）

<a name="k1s0-tier1-workflow-v1-StartRequest"></a>

### StartRequest

Start リクエスト

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| workflow_type | [string](#string) |  | ワークフロー種別（tier2 で登録されたコード名） |
| workflow_id | [string](#string) |  | 実行 ID（指定なければ tier1 が UUID を生成） |
| input | [bytes](#bytes) |  | 初期入力（任意 bytes、エンコーディングは tier2 側合意） |
| idempotent | [bool](#bool) |  | 冪等性（同一 workflow_id の重複開始は既存実行を返す） |
| context | [k1s0.tier1.common.v1.TenantContext](#k1s0-tier1-common-v1-TenantContext) |  | 呼出元コンテキスト |

<a name="k1s0-tier1-workflow-v1-StartResponse"></a>

### StartResponse

Start 応答

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| workflow_id | [string](#string) |  | ワークフロー ID（リクエストの workflow_id か、tier1 採番の UUID） |
| run_id | [string](#string) |  | 実行 ID（リトライや継続実行時に新しい値が割当） |

<a name="k1s0-tier1-workflow-v1-TerminateRequest"></a>

### TerminateRequest

Terminate リクエスト

| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| workflow_id | [string](#string) |  | 対象ワークフロー ID |
| reason | [string](#string) |  | 強制終了理由（監査用、自由記述） |
| context | [k1s0.tier1.common.v1.TenantContext](#k1s0-tier1-common-v1-TenantContext) |  | 呼出元コンテキスト |

<a name="k1s0-tier1-workflow-v1-TerminateResponse"></a>

### TerminateResponse

Terminate 応答

<a name="k1s0-tier1-workflow-v1-WorkflowStatus"></a>

### WorkflowStatus

実行状態の列挙。
注: 正典 IDL は本 enum の zero value を `RUNNING = 0` と定義しているため、
    buf STANDARD lint の ENUM_ZERO_VALUE_SUFFIX / ENUM_VALUE_PREFIX を ignore する。
buf:lint:ignore ENUM_ZERO_VALUE_SUFFIX
buf:lint:ignore ENUM_VALUE_PREFIX

| Name | Number | Description |
| ---- | ------ | ----------- |
| RUNNING | 0 | 実行中（既定値、Start 直後の状態） |
| COMPLETED | 1 | 正常完了 |
| FAILED | 2 | 失敗終了 |
| CANCELED | 3 | Cancel による正常停止 |
| TERMINATED | 4 | Terminate による強制停止 |
| CONTINUED_AS_NEW | 5 | Continue-as-New（長期ワークフローの履歴ローテーション） |

<a name="k1s0-tier1-workflow-v1-WorkflowService"></a>

### WorkflowService

Workflow API。tier2 がワークフロー種別をコード登録し、tier1 経由で実行操作する。

| Method Name | Request Type | Response Type | Description |
| ----------- | ------------ | ------------- | ------------|
| Start | [StartRequest](#k1s0-tier1-workflow-v1-StartRequest) | [StartResponse](#k1s0-tier1-workflow-v1-StartResponse) | ワークフロー開始 |
| Signal | [SignalRequest](#k1s0-tier1-workflow-v1-SignalRequest) | [SignalResponse](#k1s0-tier1-workflow-v1-SignalResponse) | シグナル送信（ワークフローへの入力イベント） |
| Query | [QueryRequest](#k1s0-tier1-workflow-v1-QueryRequest) | [QueryResponse](#k1s0-tier1-workflow-v1-QueryResponse) | クエリ（ワークフロー状態の読取り、副作用なし） |
| Cancel | [CancelRequest](#k1s0-tier1-workflow-v1-CancelRequest) | [CancelResponse](#k1s0-tier1-workflow-v1-CancelResponse) | 正常終了の依頼（キャンセル） |
| Terminate | [TerminateRequest](#k1s0-tier1-workflow-v1-TerminateRequest) | [TerminateResponse](#k1s0-tier1-workflow-v1-TerminateResponse) | 強制終了 |
| GetStatus | [GetStatusRequest](#k1s0-tier1-workflow-v1-GetStatusRequest) | [GetStatusResponse](#k1s0-tier1-workflow-v1-GetStatusResponse) | 状態取得 |

## Scalar Value Types

| .proto Type | Notes | C++ | Java | Python | Go | C# | PHP | Ruby |
| ----------- | ----- | --- | ---- | ------ | -- | -- | --- | ---- |
| <a name="double" /> double |  | double | double | float | float64 | double | float | Float |
| <a name="float" /> float |  | float | float | float | float32 | float | float | Float |
| <a name="int32" /> int32 | Uses variable-length encoding. Inefficient for encoding negative numbers – if your field is likely to have negative values, use sint32 instead. | int32 | int | int | int32 | int | integer | Bignum or Fixnum (as required) |
| <a name="int64" /> int64 | Uses variable-length encoding. Inefficient for encoding negative numbers – if your field is likely to have negative values, use sint64 instead. | int64 | long | int/long | int64 | long | integer/string | Bignum |
| <a name="uint32" /> uint32 | Uses variable-length encoding. | uint32 | int | int/long | uint32 | uint | integer | Bignum or Fixnum (as required) |
| <a name="uint64" /> uint64 | Uses variable-length encoding. | uint64 | long | int/long | uint64 | ulong | integer/string | Bignum or Fixnum (as required) |
| <a name="sint32" /> sint32 | Uses variable-length encoding. Signed int value. These more efficiently encode negative numbers than regular int32s. | int32 | int | int | int32 | int | integer | Bignum or Fixnum (as required) |
| <a name="sint64" /> sint64 | Uses variable-length encoding. Signed int value. These more efficiently encode negative numbers than regular int64s. | int64 | long | int/long | int64 | long | integer/string | Bignum |
| <a name="fixed32" /> fixed32 | Always four bytes. More efficient than uint32 if values are often greater than 2^28. | uint32 | int | int | uint32 | uint | integer | Bignum or Fixnum (as required) |
| <a name="fixed64" /> fixed64 | Always eight bytes. More efficient than uint64 if values are often greater than 2^56. | uint64 | long | int/long | uint64 | ulong | integer/string | Bignum |
| <a name="sfixed32" /> sfixed32 | Always four bytes. | int32 | int | int | int32 | int | integer | Bignum or Fixnum (as required) |
| <a name="sfixed64" /> sfixed64 | Always eight bytes. | int64 | long | int/long | int64 | long | integer/string | Bignum |
| <a name="bool" /> bool |  | bool | boolean | boolean | bool | bool | boolean | TrueClass/FalseClass |
| <a name="string" /> string | A string must always contain UTF-8 encoded or 7-bit ASCII text. | string | String | str/unicode | string | string | string | String (UTF-8) |
| <a name="bytes" /> bytes | May contain any arbitrary sequence of bytes. | string | ByteString | str | []byte | ByteString | string | String (ASCII-8BIT) |
