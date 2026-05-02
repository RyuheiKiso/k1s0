// @k1s0/api-client の本体実装。
//
// tier3 web の app から BFF (portal-bff / admin-bff) を呼ぶ薄い fetch wrapper。
// REST 17 endpoint と GraphQL の汎用呼出を、型付きメソッドとして露出する。
// リリース時点 では fetch 直叩きで TanStack Query / Apollo は導入していない（呼出側で wrap する想定）。

import type { AppConfig } from '@k1s0/config';
import type {
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
  LogSendReq,
  TelemetryEmitMetricReq,
  MetricPoint,
  PiiClassifyResp,
  PiiMaskResp,
  FeatureEvaluateBooleanReq,
  FeatureEvaluateBooleanResp,
  BindingInvokeReq,
  BindingInvokeResp,
} from './types';

// ApiClient の構築に必要な依存。
export interface ApiClientOptions {
  // 設定（BFF URL / tenantId 等）。
  config: AppConfig;
  // テスト容易性のため fetch 関数を差し替え可能（既定は global fetch）。
  fetchFn?: typeof fetch;
  // 認証トークン取得関数（未指定時は Authorization ヘッダ未付与）。
  getToken?: () => string | null;
}

// 失敗時の標準エラー型（BFF が返す JSON エラーボディに揃える）。
export class ApiError extends Error {
  // HTTP status。
  status: number;
  // BFF コード（E-T3-BFF-* / E-T1-* 等、生のまま伝搬する）。
  code: string;
  // カテゴリ（VALIDATION / UPSTREAM / 等。BFF 旧仕様後方互換用、未指定可）。
  category: string;

  constructor(status: number, code: string, message: string, category = '') {
    super(message);
    this.name = 'ApiError';
    this.status = status;
    this.code = code;
    this.category = category;
  }
}

// ApiClient は BFF への呼出を集約する。
export class ApiClient {
  // 設定（BFF URL / tenantId 等）。
  private readonly config: AppConfig;
  // fetch 実装（テスト時は mock を渡す）。
  private readonly fetchFn: typeof fetch;
  // Bearer token 提供関数（null 戻りで Authorization ヘッダ未付与）。
  private readonly getToken: () => string | null;

  // ApiClient を組み立てる。
  constructor(options: ApiClientOptions) {
    this.config = options.config;
    this.fetchFn = options.fetchFn ?? fetch.bind(globalThis);
    this.getToken = options.getToken ?? (() => null);
  }

  // ---- State ----

  // POST /api/state/get（BFF REST 経由）。
  async stateGet(store: string, key: string): Promise<StateValue> {
    return await this.postJson<StateValue>('/api/state/get', { store, key });
  }

  // POST /api/state/save。
  async stateSave(store: string, key: string, data: string): Promise<StateSaveResp> {
    return await this.postJson<StateSaveResp>('/api/state/save', { store, key, data });
  }

  // POST /api/state/delete。expectedEtag が指定されれば optimistic concurrency control が効く。
  async stateDelete(store: string, key: string, expectedEtag?: string): Promise<void> {
    await this.postJson<unknown>('/api/state/delete', {
      store,
      key,
      ...(expectedEtag ? { expected_etag: expectedEtag } : {}),
    });
  }

  // ---- PubSub ----

  // POST /api/pubsub/publish。
  async pubsubPublish(req: PubSubPublishReq): Promise<PubSubPublishResp> {
    return await this.postJson<PubSubPublishResp>('/api/pubsub/publish', req);
  }

  // ---- Secrets ----

  // POST /api/secrets/get。
  async secretsGet(name: string): Promise<SecretsGetResp> {
    return await this.postJson<SecretsGetResp>('/api/secrets/get', { name });
  }

  // POST /api/secrets/rotate。
  async secretsRotate(req: SecretsRotateReq): Promise<SecretsRotateResp> {
    return await this.postJson<SecretsRotateResp>('/api/secrets/rotate', req);
  }

  // ---- Decision ----

  // POST /api/decision/evaluate。
  async decisionEvaluate(req: DecisionEvaluateReq): Promise<DecisionEvaluateResp> {
    return await this.postJson<DecisionEvaluateResp>('/api/decision/evaluate', req);
  }

  // ---- Workflow ----

  // POST /api/workflow/start。
  async workflowStart(req: WorkflowStartReq): Promise<WorkflowStartResp> {
    return await this.postJson<WorkflowStartResp>('/api/workflow/start', req);
  }

  // ---- Invoke ----

  // POST /api/invoke/call。
  async invokeCall(req: InvokeCallReq): Promise<InvokeCallResp> {
    return await this.postJson<InvokeCallResp>('/api/invoke/call', req);
  }

  // ---- Audit ----

  // POST /api/audit/record。
  async auditRecord(req: AuditRecordReq): Promise<AuditRecordResp> {
    return await this.postJson<AuditRecordResp>('/api/audit/record', req);
  }

  // POST /api/audit/query。
  async auditQuery(req: AuditQueryReq): Promise<AuditQueryResp> {
    return await this.postJson<AuditQueryResp>('/api/audit/query', req);
  }

  // ---- Log ----

  // POST /api/log/send。
  async logSend(req: LogSendReq): Promise<void> {
    await this.postJson<unknown>('/api/log/send', req);
  }

  // ---- Telemetry ----

  // POST /api/telemetry/emit-metric。
  async telemetryEmitMetric(points: MetricPoint[]): Promise<void> {
    const body: TelemetryEmitMetricReq = { points };
    await this.postJson<unknown>('/api/telemetry/emit-metric', body);
  }

  // ---- PII ----

  // POST /api/pii/classify。
  async piiClassify(text: string): Promise<PiiClassifyResp> {
    return await this.postJson<PiiClassifyResp>('/api/pii/classify', { text });
  }

  // POST /api/pii/mask。
  async piiMask(text: string): Promise<PiiMaskResp> {
    return await this.postJson<PiiMaskResp>('/api/pii/mask', { text });
  }

  // ---- Feature ----

  // POST /api/feature/evaluate-boolean。
  async featureEvaluateBoolean(req: FeatureEvaluateBooleanReq): Promise<FeatureEvaluateBooleanResp> {
    return await this.postJson<FeatureEvaluateBooleanResp>('/api/feature/evaluate-boolean', req);
  }

  // ---- Binding ----

  // POST /api/binding/invoke。
  async bindingInvoke(req: BindingInvokeReq): Promise<BindingInvokeResp> {
    return await this.postJson<BindingInvokeResp>('/api/binding/invoke', req);
  }

  // ---- GraphQL ----

  // POST /graphql でクエリを送る汎用ヘルパ。
  async graphql<T>(query: string, variables?: Record<string, unknown>): Promise<T> {
    const body = await this.postJson<{ data?: T; errors?: { message: string }[] }>(
      '/graphql',
      { query, variables: variables ?? {} },
    );
    if (body.errors && body.errors.length > 0) {
      throw new ApiError(200, 'E-T3-BFF-GQL', body.errors[0]?.message ?? 'graphql error', 'UPSTREAM');
    }
    if (!body.data) {
      throw new ApiError(200, 'E-T3-BFF-GQL-EMPTY', 'graphql returned no data', 'UPSTREAM');
    }
    return body.data;
  }

  // ---- internal ----

  // 共通 POST JSON ヘルパ。BFF の error JSON は ApiError に詰め直して throw する。
  private async postJson<T>(path: string, body: unknown): Promise<T> {
    // 完全 URL を組み立てる。
    const url = `${this.config.bffUrl.replace(/\/$/, '')}${path}`;
    // Authorization / X-Tenant-Id ヘッダ。
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      'X-Tenant-Id': this.config.tenantId,
    };
    const token = this.getToken();
    if (token) {
      headers['Authorization'] = `Bearer ${token}`;
    }
    // fetch 実行。
    const res = await this.fetchFn(url, {
      method: 'POST',
      headers,
      body: JSON.stringify(body),
    });
    if (!res.ok) {
      // BFF の JSON エラーボディを試行的に取り出す。
      let errBody: { error?: ErrorBody & { category?: string }; code?: string; message?: string } = {};
      try {
        errBody = (await res.json()) as typeof errBody;
      } catch (_e) {
        // 非 JSON 応答は無視して汎用エラーへ。
      }
      // 旧形式 (error: { code, message, category }) と 新形式 (code, message) の両方を受ける。
      const code = errBody.error?.code ?? errBody.code ?? `HTTP_${res.status}`;
      const message = errBody.error?.message ?? errBody.message ?? `request failed: ${res.status}`;
      const category = errBody.error?.category ?? '';
      throw new ApiError(res.status, code, message, category);
    }
    return (await res.json()) as T;
  }
}

// 利便性のための factory（DI フレームワーク不要にする）。
export function createApiClient(options: ApiClientOptions): ApiClient {
  return new ApiClient(options);
}
