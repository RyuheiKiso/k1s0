// 本ファイルは tier1 公開 Workflow API の正式 proto。
// 短期 Dapr Workflow / 長期 Temporal の双方を抽象化した Workflow ライフサイクル API。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/06_Workflow_API.md
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/06_Workflow_API.md
//
// 関連要件: FR-T1-WORKFLOW-001〜005
// proto 構文宣言（proto3）
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
export var WorkflowBackend;
(function (WorkflowBackend) {
    /**
     * 既定値: tier1 が workflow_type / 期待実行時間に基づいて自動選択する。
     *
     * @generated from enum value: BACKEND_AUTO = 0;
     */
    WorkflowBackend[WorkflowBackend["BACKEND_AUTO"] = 0] = "BACKEND_AUTO";
    /**
     * 短期ワークフロー向け（Dapr Workflow building block、上限 7 日）。
     *
     * @generated from enum value: BACKEND_DAPR = 1;
     */
    WorkflowBackend[WorkflowBackend["BACKEND_DAPR"] = 1] = "BACKEND_DAPR";
    /**
     * 長期ワークフロー向け（Temporal、上限なし）。
     *
     * @generated from enum value: BACKEND_TEMPORAL = 2;
     */
    WorkflowBackend[WorkflowBackend["BACKEND_TEMPORAL"] = 2] = "BACKEND_TEMPORAL";
})(WorkflowBackend || (WorkflowBackend = {}));
// Retrieve enum metadata with: proto3.getEnumType(WorkflowBackend)
proto3.util.setEnumType(WorkflowBackend, "k1s0.tier1.workflow.v1.WorkflowBackend", [
    { no: 0, name: "BACKEND_AUTO" },
    { no: 1, name: "BACKEND_DAPR" },
    { no: 2, name: "BACKEND_TEMPORAL" },
]);
/**
 * 実行状態の列挙。
 * 注: 正典 IDL は本 enum の zero value を `RUNNING = 0` と定義しているため、
 *     buf STANDARD lint の ENUM_ZERO_VALUE_SUFFIX / ENUM_VALUE_PREFIX を ignore する。
 * buf:lint:ignore ENUM_ZERO_VALUE_SUFFIX
 * buf:lint:ignore ENUM_VALUE_PREFIX
 *
 * @generated from enum k1s0.tier1.workflow.v1.WorkflowStatus
 */
export var WorkflowStatus;
(function (WorkflowStatus) {
    /**
     * 実行中（既定値、Start 直後の状態）
     *
     * @generated from enum value: RUNNING = 0;
     */
    WorkflowStatus[WorkflowStatus["RUNNING"] = 0] = "RUNNING";
    /**
     * 正常完了
     *
     * @generated from enum value: COMPLETED = 1;
     */
    WorkflowStatus[WorkflowStatus["COMPLETED"] = 1] = "COMPLETED";
    /**
     * 失敗終了
     *
     * @generated from enum value: FAILED = 2;
     */
    WorkflowStatus[WorkflowStatus["FAILED"] = 2] = "FAILED";
    /**
     * Cancel による正常停止
     *
     * @generated from enum value: CANCELED = 3;
     */
    WorkflowStatus[WorkflowStatus["CANCELED"] = 3] = "CANCELED";
    /**
     * Terminate による強制停止
     *
     * @generated from enum value: TERMINATED = 4;
     */
    WorkflowStatus[WorkflowStatus["TERMINATED"] = 4] = "TERMINATED";
    /**
     * Continue-as-New（長期ワークフローの履歴ローテーション）
     *
     * @generated from enum value: CONTINUED_AS_NEW = 5;
     */
    WorkflowStatus[WorkflowStatus["CONTINUED_AS_NEW"] = 5] = "CONTINUED_AS_NEW";
})(WorkflowStatus || (WorkflowStatus = {}));
// Retrieve enum metadata with: proto3.getEnumType(WorkflowStatus)
proto3.util.setEnumType(WorkflowStatus, "k1s0.tier1.workflow.v1.WorkflowStatus", [
    { no: 0, name: "RUNNING" },
    { no: 1, name: "COMPLETED" },
    { no: 2, name: "FAILED" },
    { no: 3, name: "CANCELED" },
    { no: 4, name: "TERMINATED" },
    { no: 5, name: "CONTINUED_AS_NEW" },
]);
/**
 * Start リクエスト
 *
 * @generated from message k1s0.tier1.workflow.v1.StartRequest
 */
export class StartRequest extends Message {
    /**
     * ワークフロー種別（tier2 で登録されたコード名）
     *
     * @generated from field: string workflow_type = 1;
     */
    workflowType = "";
    /**
     * 実行 ID（指定なければ tier1 が UUID を生成）
     *
     * @generated from field: string workflow_id = 2;
     */
    workflowId = "";
    /**
     * 初期入力（任意 bytes、エンコーディングは tier2 側合意）
     *
     * @generated from field: bytes input = 3;
     */
    input = new Uint8Array(0);
    /**
     * 冪等性（同一 workflow_id の重複開始は既存実行を返す）
     *
     * @generated from field: bool idempotent = 4;
     */
    idempotent = false;
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 5;
     */
    context;
    /**
     * バックエンド hint（BACKEND_AUTO で tier1 が自動選択）。
     *
     * @generated from field: k1s0.tier1.workflow.v1.WorkflowBackend backend = 6;
     */
    backend = WorkflowBackend.BACKEND_AUTO;
    /**
     * 冪等性キー（共通規約 §「冪等性と再試行」: 24h TTL の dedup）
     * 同一キーでの再試行は副作用を重複させず初回 StartResponse を返す
     *
     * @generated from field: string idempotency_key = 7;
     */
    idempotencyKey = "";
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.workflow.v1.StartRequest";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "workflow_type", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 2, name: "workflow_id", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 3, name: "input", kind: "scalar", T: 12 /* ScalarType.BYTES */ },
        { no: 4, name: "idempotent", kind: "scalar", T: 8 /* ScalarType.BOOL */ },
        { no: 5, name: "context", kind: "message", T: TenantContext },
        { no: 6, name: "backend", kind: "enum", T: proto3.getEnumType(WorkflowBackend) },
        { no: 7, name: "idempotency_key", kind: "scalar", T: 9 /* ScalarType.STRING */ },
    ]);
    static fromBinary(bytes, options) {
        return new StartRequest().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new StartRequest().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new StartRequest().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(StartRequest, a, b);
    }
}
/**
 * Start 応答
 *
 * @generated from message k1s0.tier1.workflow.v1.StartResponse
 */
export class StartResponse extends Message {
    /**
     * ワークフロー ID（リクエストの workflow_id か、tier1 採番の UUID）
     *
     * @generated from field: string workflow_id = 1;
     */
    workflowId = "";
    /**
     * 実行 ID（リトライや継続実行時に新しい値が割当）
     *
     * @generated from field: string run_id = 2;
     */
    runId = "";
    /**
     * 実際に選択された backend（AUTO で起動された場合の解決結果が入る）。
     *
     * @generated from field: k1s0.tier1.workflow.v1.WorkflowBackend backend = 3;
     */
    backend = WorkflowBackend.BACKEND_AUTO;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.workflow.v1.StartResponse";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "workflow_id", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 2, name: "run_id", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 3, name: "backend", kind: "enum", T: proto3.getEnumType(WorkflowBackend) },
    ]);
    static fromBinary(bytes, options) {
        return new StartResponse().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new StartResponse().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new StartResponse().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(StartResponse, a, b);
    }
}
/**
 * Signal リクエスト
 *
 * @generated from message k1s0.tier1.workflow.v1.SignalRequest
 */
export class SignalRequest extends Message {
    /**
     * 対象ワークフロー ID
     *
     * @generated from field: string workflow_id = 1;
     */
    workflowId = "";
    /**
     * シグナル名（tier2 ワークフロー実装が listen している名前）
     *
     * @generated from field: string signal_name = 2;
     */
    signalName = "";
    /**
     * ペイロード（任意 bytes）
     *
     * @generated from field: bytes payload = 3;
     */
    payload = new Uint8Array(0);
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 4;
     */
    context;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.workflow.v1.SignalRequest";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "workflow_id", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 2, name: "signal_name", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 3, name: "payload", kind: "scalar", T: 12 /* ScalarType.BYTES */ },
        { no: 4, name: "context", kind: "message", T: TenantContext },
    ]);
    static fromBinary(bytes, options) {
        return new SignalRequest().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new SignalRequest().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new SignalRequest().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(SignalRequest, a, b);
    }
}
/**
 * Signal 応答（フィールド無し、成功は OK gRPC ステータスで表現）
 *
 * @generated from message k1s0.tier1.workflow.v1.SignalResponse
 */
export class SignalResponse extends Message {
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.workflow.v1.SignalResponse";
    static fields = proto3.util.newFieldList(() => []);
    static fromBinary(bytes, options) {
        return new SignalResponse().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new SignalResponse().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new SignalResponse().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(SignalResponse, a, b);
    }
}
/**
 * Query リクエスト
 *
 * @generated from message k1s0.tier1.workflow.v1.QueryRequest
 */
export class QueryRequest extends Message {
    /**
     * 対象ワークフロー ID
     *
     * @generated from field: string workflow_id = 1;
     */
    workflowId = "";
    /**
     * クエリ名（tier2 ワークフロー実装が register しているクエリハンドラ名）
     *
     * @generated from field: string query_name = 2;
     */
    queryName = "";
    /**
     * ペイロード
     *
     * @generated from field: bytes payload = 3;
     */
    payload = new Uint8Array(0);
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 4;
     */
    context;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.workflow.v1.QueryRequest";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "workflow_id", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 2, name: "query_name", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 3, name: "payload", kind: "scalar", T: 12 /* ScalarType.BYTES */ },
        { no: 4, name: "context", kind: "message", T: TenantContext },
    ]);
    static fromBinary(bytes, options) {
        return new QueryRequest().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new QueryRequest().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new QueryRequest().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(QueryRequest, a, b);
    }
}
/**
 * Query 応答
 *
 * @generated from message k1s0.tier1.workflow.v1.QueryResponse
 */
export class QueryResponse extends Message {
    /**
     * クエリ結果
     *
     * @generated from field: bytes result = 1;
     */
    result = new Uint8Array(0);
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.workflow.v1.QueryResponse";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "result", kind: "scalar", T: 12 /* ScalarType.BYTES */ },
    ]);
    static fromBinary(bytes, options) {
        return new QueryResponse().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new QueryResponse().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new QueryResponse().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(QueryResponse, a, b);
    }
}
/**
 * Cancel リクエスト
 *
 * @generated from message k1s0.tier1.workflow.v1.CancelRequest
 */
export class CancelRequest extends Message {
    /**
     * 対象ワークフロー ID
     *
     * @generated from field: string workflow_id = 1;
     */
    workflowId = "";
    /**
     * キャンセル理由（監査用、自由記述）
     *
     * @generated from field: string reason = 2;
     */
    reason = "";
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 3;
     */
    context;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.workflow.v1.CancelRequest";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "workflow_id", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 2, name: "reason", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 3, name: "context", kind: "message", T: TenantContext },
    ]);
    static fromBinary(bytes, options) {
        return new CancelRequest().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new CancelRequest().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new CancelRequest().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(CancelRequest, a, b);
    }
}
/**
 * Cancel 応答
 *
 * @generated from message k1s0.tier1.workflow.v1.CancelResponse
 */
export class CancelResponse extends Message {
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.workflow.v1.CancelResponse";
    static fields = proto3.util.newFieldList(() => []);
    static fromBinary(bytes, options) {
        return new CancelResponse().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new CancelResponse().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new CancelResponse().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(CancelResponse, a, b);
    }
}
/**
 * Terminate リクエスト
 *
 * @generated from message k1s0.tier1.workflow.v1.TerminateRequest
 */
export class TerminateRequest extends Message {
    /**
     * 対象ワークフロー ID
     *
     * @generated from field: string workflow_id = 1;
     */
    workflowId = "";
    /**
     * 強制終了理由（監査用、自由記述）
     *
     * @generated from field: string reason = 2;
     */
    reason = "";
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 3;
     */
    context;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.workflow.v1.TerminateRequest";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "workflow_id", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 2, name: "reason", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 3, name: "context", kind: "message", T: TenantContext },
    ]);
    static fromBinary(bytes, options) {
        return new TerminateRequest().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new TerminateRequest().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new TerminateRequest().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(TerminateRequest, a, b);
    }
}
/**
 * Terminate 応答
 *
 * @generated from message k1s0.tier1.workflow.v1.TerminateResponse
 */
export class TerminateResponse extends Message {
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.workflow.v1.TerminateResponse";
    static fields = proto3.util.newFieldList(() => []);
    static fromBinary(bytes, options) {
        return new TerminateResponse().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new TerminateResponse().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new TerminateResponse().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(TerminateResponse, a, b);
    }
}
/**
 * GetStatus リクエスト
 *
 * @generated from message k1s0.tier1.workflow.v1.GetStatusRequest
 */
export class GetStatusRequest extends Message {
    /**
     * 対象ワークフロー ID
     *
     * @generated from field: string workflow_id = 1;
     */
    workflowId = "";
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 2;
     */
    context;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.workflow.v1.GetStatusRequest";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "workflow_id", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 2, name: "context", kind: "message", T: TenantContext },
    ]);
    static fromBinary(bytes, options) {
        return new GetStatusRequest().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new GetStatusRequest().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new GetStatusRequest().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(GetStatusRequest, a, b);
    }
}
/**
 * GetStatus 応答
 *
 * @generated from message k1s0.tier1.workflow.v1.GetStatusResponse
 */
export class GetStatusResponse extends Message {
    /**
     * 現在状態
     *
     * @generated from field: k1s0.tier1.workflow.v1.WorkflowStatus status = 1;
     */
    status = WorkflowStatus.RUNNING;
    /**
     * 直近の実行 ID（Continue-as-New 後は新 run_id を返す）
     *
     * @generated from field: string run_id = 2;
     */
    runId = "";
    /**
     * 完了時の出力（status = COMPLETED の時のみ）
     *
     * @generated from field: bytes output = 3;
     */
    output = new Uint8Array(0);
    /**
     * 失敗時のエラー詳細（status = FAILED の時のみ）
     *
     * @generated from field: k1s0.tier1.common.v1.ErrorDetail error = 4;
     */
    error;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.workflow.v1.GetStatusResponse";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "status", kind: "enum", T: proto3.getEnumType(WorkflowStatus) },
        { no: 2, name: "run_id", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 3, name: "output", kind: "scalar", T: 12 /* ScalarType.BYTES */ },
        { no: 4, name: "error", kind: "message", T: ErrorDetail },
    ]);
    static fromBinary(bytes, options) {
        return new GetStatusResponse().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new GetStatusResponse().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new GetStatusResponse().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(GetStatusResponse, a, b);
    }
}
//# sourceMappingURL=workflow_service_pb.js.map