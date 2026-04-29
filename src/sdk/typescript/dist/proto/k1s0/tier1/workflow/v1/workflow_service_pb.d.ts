import type { BinaryReadOptions, FieldList, JsonReadOptions, JsonValue, PartialMessage, PlainMessage } from "@bufbuild/protobuf";
import { Message, proto3 } from "@bufbuild/protobuf";
import { ErrorDetail, TenantContext } from "../../common/v1/common_pb.js";
/**
 * バックエンド種別（FR-T1-WORKFLOW-001 の "短期は Dapr Workflow、長期実行は Temporal" 対応）。
 * SDK の RunShort / RunLong は本 enum を BACKEND_DAPR / BACKEND_TEMPORAL に固定する。
 * 注: zero 値は AUTO（明示指定なし、tier1 が workflow_type / 期待実行時間で振り分け）。
 * buf:lint:ignore ENUM_ZERO_VALUE_SUFFIX
 *
 * @generated from enum k1s0.tier1.workflow.v1.WorkflowBackend
 */
export declare enum WorkflowBackend {
    /**
     * 既定値: tier1 が workflow_type / 期待実行時間に基づいて自動選択する。
     *
     * @generated from enum value: BACKEND_AUTO = 0;
     */
    BACKEND_AUTO = 0,
    /**
     * 短期ワークフロー向け（Dapr Workflow building block、上限 7 日）。
     *
     * @generated from enum value: BACKEND_DAPR = 1;
     */
    BACKEND_DAPR = 1,
    /**
     * 長期ワークフロー向け（Temporal、上限なし）。
     *
     * @generated from enum value: BACKEND_TEMPORAL = 2;
     */
    BACKEND_TEMPORAL = 2
}
/**
 * 実行状態の列挙。
 * 注: 正典 IDL は本 enum の zero value を `RUNNING = 0` と定義しているため、
 *     buf STANDARD lint の ENUM_ZERO_VALUE_SUFFIX / ENUM_VALUE_PREFIX を ignore する。
 * buf:lint:ignore ENUM_ZERO_VALUE_SUFFIX
 * buf:lint:ignore ENUM_VALUE_PREFIX
 *
 * @generated from enum k1s0.tier1.workflow.v1.WorkflowStatus
 */
export declare enum WorkflowStatus {
    /**
     * 実行中（既定値、Start 直後の状態）
     *
     * @generated from enum value: RUNNING = 0;
     */
    RUNNING = 0,
    /**
     * 正常完了
     *
     * @generated from enum value: COMPLETED = 1;
     */
    COMPLETED = 1,
    /**
     * 失敗終了
     *
     * @generated from enum value: FAILED = 2;
     */
    FAILED = 2,
    /**
     * Cancel による正常停止
     *
     * @generated from enum value: CANCELED = 3;
     */
    CANCELED = 3,
    /**
     * Terminate による強制停止
     *
     * @generated from enum value: TERMINATED = 4;
     */
    TERMINATED = 4,
    /**
     * Continue-as-New（長期ワークフローの履歴ローテーション）
     *
     * @generated from enum value: CONTINUED_AS_NEW = 5;
     */
    CONTINUED_AS_NEW = 5
}
/**
 * Start リクエスト
 *
 * @generated from message k1s0.tier1.workflow.v1.StartRequest
 */
export declare class StartRequest extends Message<StartRequest> {
    /**
     * ワークフロー種別（tier2 で登録されたコード名）
     *
     * @generated from field: string workflow_type = 1;
     */
    workflowType: string;
    /**
     * 実行 ID（指定なければ tier1 が UUID を生成）
     *
     * @generated from field: string workflow_id = 2;
     */
    workflowId: string;
    /**
     * 初期入力（任意 bytes、エンコーディングは tier2 側合意）
     *
     * @generated from field: bytes input = 3;
     */
    input: Uint8Array;
    /**
     * 冪等性（同一 workflow_id の重複開始は既存実行を返す）
     *
     * @generated from field: bool idempotent = 4;
     */
    idempotent: boolean;
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 5;
     */
    context?: TenantContext;
    /**
     * バックエンド hint（BACKEND_AUTO で tier1 が自動選択）。
     *
     * @generated from field: k1s0.tier1.workflow.v1.WorkflowBackend backend = 6;
     */
    backend: WorkflowBackend;
    constructor(data?: PartialMessage<StartRequest>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.workflow.v1.StartRequest";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): StartRequest;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): StartRequest;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): StartRequest;
    static equals(a: StartRequest | PlainMessage<StartRequest> | undefined, b: StartRequest | PlainMessage<StartRequest> | undefined): boolean;
}
/**
 * Start 応答
 *
 * @generated from message k1s0.tier1.workflow.v1.StartResponse
 */
export declare class StartResponse extends Message<StartResponse> {
    /**
     * ワークフロー ID（リクエストの workflow_id か、tier1 採番の UUID）
     *
     * @generated from field: string workflow_id = 1;
     */
    workflowId: string;
    /**
     * 実行 ID（リトライや継続実行時に新しい値が割当）
     *
     * @generated from field: string run_id = 2;
     */
    runId: string;
    /**
     * 実際に選択された backend（AUTO で起動された場合の解決結果が入る）。
     *
     * @generated from field: k1s0.tier1.workflow.v1.WorkflowBackend backend = 3;
     */
    backend: WorkflowBackend;
    constructor(data?: PartialMessage<StartResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.workflow.v1.StartResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): StartResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): StartResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): StartResponse;
    static equals(a: StartResponse | PlainMessage<StartResponse> | undefined, b: StartResponse | PlainMessage<StartResponse> | undefined): boolean;
}
/**
 * Signal リクエスト
 *
 * @generated from message k1s0.tier1.workflow.v1.SignalRequest
 */
export declare class SignalRequest extends Message<SignalRequest> {
    /**
     * 対象ワークフロー ID
     *
     * @generated from field: string workflow_id = 1;
     */
    workflowId: string;
    /**
     * シグナル名（tier2 ワークフロー実装が listen している名前）
     *
     * @generated from field: string signal_name = 2;
     */
    signalName: string;
    /**
     * ペイロード（任意 bytes）
     *
     * @generated from field: bytes payload = 3;
     */
    payload: Uint8Array;
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 4;
     */
    context?: TenantContext;
    constructor(data?: PartialMessage<SignalRequest>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.workflow.v1.SignalRequest";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): SignalRequest;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): SignalRequest;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): SignalRequest;
    static equals(a: SignalRequest | PlainMessage<SignalRequest> | undefined, b: SignalRequest | PlainMessage<SignalRequest> | undefined): boolean;
}
/**
 * Signal 応答（フィールド無し、成功は OK gRPC ステータスで表現）
 *
 * @generated from message k1s0.tier1.workflow.v1.SignalResponse
 */
export declare class SignalResponse extends Message<SignalResponse> {
    constructor(data?: PartialMessage<SignalResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.workflow.v1.SignalResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): SignalResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): SignalResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): SignalResponse;
    static equals(a: SignalResponse | PlainMessage<SignalResponse> | undefined, b: SignalResponse | PlainMessage<SignalResponse> | undefined): boolean;
}
/**
 * Query リクエスト
 *
 * @generated from message k1s0.tier1.workflow.v1.QueryRequest
 */
export declare class QueryRequest extends Message<QueryRequest> {
    /**
     * 対象ワークフロー ID
     *
     * @generated from field: string workflow_id = 1;
     */
    workflowId: string;
    /**
     * クエリ名（tier2 ワークフロー実装が register しているクエリハンドラ名）
     *
     * @generated from field: string query_name = 2;
     */
    queryName: string;
    /**
     * ペイロード
     *
     * @generated from field: bytes payload = 3;
     */
    payload: Uint8Array;
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 4;
     */
    context?: TenantContext;
    constructor(data?: PartialMessage<QueryRequest>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.workflow.v1.QueryRequest";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): QueryRequest;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): QueryRequest;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): QueryRequest;
    static equals(a: QueryRequest | PlainMessage<QueryRequest> | undefined, b: QueryRequest | PlainMessage<QueryRequest> | undefined): boolean;
}
/**
 * Query 応答
 *
 * @generated from message k1s0.tier1.workflow.v1.QueryResponse
 */
export declare class QueryResponse extends Message<QueryResponse> {
    /**
     * クエリ結果
     *
     * @generated from field: bytes result = 1;
     */
    result: Uint8Array;
    constructor(data?: PartialMessage<QueryResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.workflow.v1.QueryResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): QueryResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): QueryResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): QueryResponse;
    static equals(a: QueryResponse | PlainMessage<QueryResponse> | undefined, b: QueryResponse | PlainMessage<QueryResponse> | undefined): boolean;
}
/**
 * Cancel リクエスト
 *
 * @generated from message k1s0.tier1.workflow.v1.CancelRequest
 */
export declare class CancelRequest extends Message<CancelRequest> {
    /**
     * 対象ワークフロー ID
     *
     * @generated from field: string workflow_id = 1;
     */
    workflowId: string;
    /**
     * キャンセル理由（監査用、自由記述）
     *
     * @generated from field: string reason = 2;
     */
    reason: string;
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 3;
     */
    context?: TenantContext;
    constructor(data?: PartialMessage<CancelRequest>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.workflow.v1.CancelRequest";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): CancelRequest;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): CancelRequest;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): CancelRequest;
    static equals(a: CancelRequest | PlainMessage<CancelRequest> | undefined, b: CancelRequest | PlainMessage<CancelRequest> | undefined): boolean;
}
/**
 * Cancel 応答
 *
 * @generated from message k1s0.tier1.workflow.v1.CancelResponse
 */
export declare class CancelResponse extends Message<CancelResponse> {
    constructor(data?: PartialMessage<CancelResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.workflow.v1.CancelResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): CancelResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): CancelResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): CancelResponse;
    static equals(a: CancelResponse | PlainMessage<CancelResponse> | undefined, b: CancelResponse | PlainMessage<CancelResponse> | undefined): boolean;
}
/**
 * Terminate リクエスト
 *
 * @generated from message k1s0.tier1.workflow.v1.TerminateRequest
 */
export declare class TerminateRequest extends Message<TerminateRequest> {
    /**
     * 対象ワークフロー ID
     *
     * @generated from field: string workflow_id = 1;
     */
    workflowId: string;
    /**
     * 強制終了理由（監査用、自由記述）
     *
     * @generated from field: string reason = 2;
     */
    reason: string;
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 3;
     */
    context?: TenantContext;
    constructor(data?: PartialMessage<TerminateRequest>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.workflow.v1.TerminateRequest";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): TerminateRequest;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): TerminateRequest;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): TerminateRequest;
    static equals(a: TerminateRequest | PlainMessage<TerminateRequest> | undefined, b: TerminateRequest | PlainMessage<TerminateRequest> | undefined): boolean;
}
/**
 * Terminate 応答
 *
 * @generated from message k1s0.tier1.workflow.v1.TerminateResponse
 */
export declare class TerminateResponse extends Message<TerminateResponse> {
    constructor(data?: PartialMessage<TerminateResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.workflow.v1.TerminateResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): TerminateResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): TerminateResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): TerminateResponse;
    static equals(a: TerminateResponse | PlainMessage<TerminateResponse> | undefined, b: TerminateResponse | PlainMessage<TerminateResponse> | undefined): boolean;
}
/**
 * GetStatus リクエスト
 *
 * @generated from message k1s0.tier1.workflow.v1.GetStatusRequest
 */
export declare class GetStatusRequest extends Message<GetStatusRequest> {
    /**
     * 対象ワークフロー ID
     *
     * @generated from field: string workflow_id = 1;
     */
    workflowId: string;
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 2;
     */
    context?: TenantContext;
    constructor(data?: PartialMessage<GetStatusRequest>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.workflow.v1.GetStatusRequest";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): GetStatusRequest;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): GetStatusRequest;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): GetStatusRequest;
    static equals(a: GetStatusRequest | PlainMessage<GetStatusRequest> | undefined, b: GetStatusRequest | PlainMessage<GetStatusRequest> | undefined): boolean;
}
/**
 * GetStatus 応答
 *
 * @generated from message k1s0.tier1.workflow.v1.GetStatusResponse
 */
export declare class GetStatusResponse extends Message<GetStatusResponse> {
    /**
     * 現在状態
     *
     * @generated from field: k1s0.tier1.workflow.v1.WorkflowStatus status = 1;
     */
    status: WorkflowStatus;
    /**
     * 直近の実行 ID（Continue-as-New 後は新 run_id を返す）
     *
     * @generated from field: string run_id = 2;
     */
    runId: string;
    /**
     * 完了時の出力（status = COMPLETED の時のみ）
     *
     * @generated from field: bytes output = 3;
     */
    output: Uint8Array;
    /**
     * 失敗時のエラー詳細（status = FAILED の時のみ）
     *
     * @generated from field: k1s0.tier1.common.v1.ErrorDetail error = 4;
     */
    error?: ErrorDetail;
    constructor(data?: PartialMessage<GetStatusResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.workflow.v1.GetStatusResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): GetStatusResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): GetStatusResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): GetStatusResponse;
    static equals(a: GetStatusResponse | PlainMessage<GetStatusResponse> | undefined, b: GetStatusResponse | PlainMessage<GetStatusResponse> | undefined): boolean;
}
//# sourceMappingURL=workflow_service_pb.d.ts.map