// @k1s0/api-client エントリポイント。
//
// tier3 web の app から BFF (portal-bff / admin-bff) を呼ぶ薄い fetch wrapper。
// 14 サービス分の REST endpoint と GraphQL 汎用呼出を ApiClient クラスに集約し、
// 型は types.ts に分離する。
//
// 利用例:
//   import { createApiClient } from '@k1s0/api-client';
//   const client = createApiClient({ config });
//   const value = await client.stateGet('postgres', 'user/1');
//   const flag = await client.featureEvaluateBoolean({ flag_key: 'new-ui', eval_ctx: { tenant: 't' } });

// 本体実装の re-export。
export { ApiClient, ApiError, createApiClient } from './client';
export type { ApiClientOptions } from './client';

// 型の re-export。
export type {
  ErrorBody,
  StateValue,
  StateSaveResp,
  PubSubPublishReq,
  PubSubPublishResp,
  SecretsGetResp,
  SecretsRotateReq,
  SecretsRotateResp,
  DecisionEvaluateReq,
  DecisionEvaluateResp,
  WorkflowStartReq,
  WorkflowStartResp,
  InvokeCallReq,
  InvokeCallResp,
  AuditRecordReq,
  AuditRecordResp,
  AuditQueryReq,
  AuditQueryResp,
  AuditEvent,
  LogSeverity,
  LogSendReq,
  MetricPoint,
  TelemetryEmitMetricReq,
  PiiFinding,
  PiiClassifyResp,
  PiiMaskResp,
  FeatureEvaluateBooleanReq,
  FeatureEvaluateBooleanResp,
  BindingInvokeReq,
  BindingInvokeResp,
} from './types';
