// @k1s0/api-client の型定義集。
//
// tier3 BFF (portal-bff / admin-bff) の REST endpoint と完全に対応した
// Request / Response 型を集約する。JSON フィールド命名は BFF JSON
// (snake_case) に揃え、TypeScript 側でも snake_case をそのまま使う。
//
// BFF endpoint との対応:
//   docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/04_bff配置.md
//   src/tier3/bff/internal/rest/{state,pubsub,secrets,decision,workflow,
//     invoke,audit,log,telemetry,pii,feature,binding}.go

// 共通: BFF が返す JSON エラーボディ。
export interface ErrorBody {
  // E-T3-BFF-* / E-T1-* 等のコード。
  code: string;
  // 人間可読メッセージ。
  message: string;
}

// State.
export interface StateValue {
  data: string;
  etag: string;
  found: boolean;
}
export interface StateSaveResp {
  etag: string;
}

// PubSub.
export interface PubSubPublishReq {
  topic: string;
  data: string;
  content_type?: string;
  idempotency_key?: string;
  metadata?: Record<string, string>;
}
export interface PubSubPublishResp {
  offset: number;
}

// Secrets.
export interface SecretsGetResp {
  values: Record<string, string>;
  version: number;
}
export interface SecretsRotateReq {
  name: string;
  grace_period_sec?: number;
  idempotency_key?: string;
}
export interface SecretsRotateResp {
  new_version: number;
  previous_version: number;
}

// Decision.
export interface DecisionEvaluateReq {
  rule_id: string;
  rule_version?: string;
  input_json: string;
  include_trace?: boolean;
}
export interface DecisionEvaluateResp {
  output_json: string;
  trace_json?: string;
  elapsed_us: number;
}

// Workflow.
export interface WorkflowStartReq {
  workflow_type: string;
  workflow_id: string;
  input?: string;
  idempotent?: boolean;
}
export interface WorkflowStartResp {
  workflow_id: string;
  run_id: string;
}

// Invoke.
export interface InvokeCallReq {
  app_id: string;
  method: string;
  data?: string;
  content_type?: string;
  timeout_ms?: number;
}
export interface InvokeCallResp {
  data: string;
  content_type: string;
  status: number;
}

// Audit.
export interface AuditRecordReq {
  actor: string;
  action: string;
  resource: string;
  outcome: string;
  attributes?: Record<string, string>;
  idempotency_key?: string;
}
export interface AuditRecordResp {
  audit_id: string;
}
export interface AuditQueryReq {
  // RFC3339 文字列（空文字 / 未指定なら範囲未指定として SDK にそのまま渡す）。
  from?: string;
  to?: string;
  filters?: Record<string, string>;
  limit?: number;
}
export interface AuditEvent {
  occurred_at_millis: number;
  actor: string;
  action: string;
  resource: string;
  outcome: string;
  attributes?: Record<string, string>;
}
export interface AuditQueryResp {
  events: AuditEvent[];
}

// Log.
// severity は TRACE / DEBUG / INFO / WARN / ERROR / FATAL のいずれか
// （未指定は INFO 扱い）。
export type LogSeverity = 'TRACE' | 'DEBUG' | 'INFO' | 'WARN' | 'ERROR' | 'FATAL';
export interface LogSendReq {
  severity?: LogSeverity;
  body: string;
  attributes?: Record<string, string>;
}

// Telemetry.
export interface MetricPoint {
  name: string;
  value: number;
  labels?: Record<string, string>;
}
export interface TelemetryEmitMetricReq {
  points: MetricPoint[];
}

// PII.
export interface PiiFinding {
  type: string;
  start: number;
  end: number;
  confidence: number;
}
export interface PiiClassifyResp {
  findings: PiiFinding[];
  contains_pii: boolean;
}
export interface PiiMaskResp {
  masked_text: string;
  findings: PiiFinding[];
}

// Feature.
export interface FeatureEvaluateBooleanReq {
  flag_key: string;
  eval_ctx?: Record<string, string>;
}
export interface FeatureEvaluateBooleanResp {
  value: boolean;
  variant: string;
  reason: string;
}

// Binding.
export interface BindingInvokeReq {
  name: string;
  operation: string;
  data?: string;
  metadata?: Record<string, string>;
}
export interface BindingInvokeResp {
  data?: string;
  metadata?: Record<string, string>;
}
