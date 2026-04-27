import type { BinaryReadOptions, FieldList, JsonReadOptions, JsonValue, PartialMessage, PlainMessage } from "@bufbuild/protobuf";
import { Message, proto3 } from "@bufbuild/protobuf";
import { TenantContext } from "../../common/v1/common_pb.js";
/**
 * Evaluate リクエスト
 *
 * @generated from message k1s0.tier1.decision.v1.EvaluateRequest
 */
export declare class EvaluateRequest extends Message<EvaluateRequest> {
    /**
     * ルール ID（tier2 で登録した JDM 文書の識別子）
     *
     * @generated from field: string rule_id = 1;
     */
    ruleId: string;
    /**
     * ルールバージョン（省略時は最新有効）
     *
     * @generated from field: string rule_version = 2;
     */
    ruleVersion: string;
    /**
     * 入力（JDM の context に相当、任意 JSON）
     *
     * @generated from field: bytes input_json = 3;
     */
    inputJson: Uint8Array;
    /**
     * trace 情報を返すか（デバッグ用、PII を含む可能性あり）
     *
     * @generated from field: bool include_trace = 4;
     */
    includeTrace: boolean;
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 5;
     */
    context?: TenantContext;
    constructor(data?: PartialMessage<EvaluateRequest>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.decision.v1.EvaluateRequest";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): EvaluateRequest;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): EvaluateRequest;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): EvaluateRequest;
    static equals(a: EvaluateRequest | PlainMessage<EvaluateRequest> | undefined, b: EvaluateRequest | PlainMessage<EvaluateRequest> | undefined): boolean;
}
/**
 * Evaluate 応答
 *
 * @generated from message k1s0.tier1.decision.v1.EvaluateResponse
 */
export declare class EvaluateResponse extends Message<EvaluateResponse> {
    /**
     * 出力（JDM 評価結果、任意 JSON）
     *
     * @generated from field: bytes output_json = 1;
     */
    outputJson: Uint8Array;
    /**
     * 評価されたノードのトレース（include_trace=true の時のみ、空 bytes）
     *
     * @generated from field: bytes trace_json = 2;
     */
    traceJson: Uint8Array;
    /**
     * 評価にかかった時間（マイクロ秒）
     *
     * @generated from field: int64 elapsed_us = 3;
     */
    elapsedUs: bigint;
    constructor(data?: PartialMessage<EvaluateResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.decision.v1.EvaluateResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): EvaluateResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): EvaluateResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): EvaluateResponse;
    static equals(a: EvaluateResponse | PlainMessage<EvaluateResponse> | undefined, b: EvaluateResponse | PlainMessage<EvaluateResponse> | undefined): boolean;
}
/**
 * BatchEvaluate リクエスト
 *
 * @generated from message k1s0.tier1.decision.v1.BatchEvaluateRequest
 */
export declare class BatchEvaluateRequest extends Message<BatchEvaluateRequest> {
    /**
     * ルール ID（全入力で共通）
     *
     * @generated from field: string rule_id = 1;
     */
    ruleId: string;
    /**
     * ルールバージョン
     *
     * @generated from field: string rule_version = 2;
     */
    ruleVersion: string;
    /**
     * 入力 JSON 列（順序を保って評価される）
     *
     * @generated from field: repeated bytes inputs_json = 3;
     */
    inputsJson: Uint8Array[];
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 4;
     */
    context?: TenantContext;
    constructor(data?: PartialMessage<BatchEvaluateRequest>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.decision.v1.BatchEvaluateRequest";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): BatchEvaluateRequest;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): BatchEvaluateRequest;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): BatchEvaluateRequest;
    static equals(a: BatchEvaluateRequest | PlainMessage<BatchEvaluateRequest> | undefined, b: BatchEvaluateRequest | PlainMessage<BatchEvaluateRequest> | undefined): boolean;
}
/**
 * BatchEvaluate 応答
 *
 * @generated from message k1s0.tier1.decision.v1.BatchEvaluateResponse
 */
export declare class BatchEvaluateResponse extends Message<BatchEvaluateResponse> {
    /**
     * 出力 JSON 列（inputs_json と同じ順序）
     *
     * @generated from field: repeated bytes outputs_json = 1;
     */
    outputsJson: Uint8Array[];
    constructor(data?: PartialMessage<BatchEvaluateResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.decision.v1.BatchEvaluateResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): BatchEvaluateResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): BatchEvaluateResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): BatchEvaluateResponse;
    static equals(a: BatchEvaluateResponse | PlainMessage<BatchEvaluateResponse> | undefined, b: BatchEvaluateResponse | PlainMessage<BatchEvaluateResponse> | undefined): boolean;
}
/**
 * RegisterRule リクエスト
 *
 * @generated from message k1s0.tier1.decision.v1.RegisterRuleRequest
 */
export declare class RegisterRuleRequest extends Message<RegisterRuleRequest> {
    /**
     * ルール ID（tenant 内で一意）
     *
     * @generated from field: string rule_id = 1;
     */
    ruleId: string;
    /**
     * JDM 文書（前節 JSON Schema に準拠、UTF-8 JSON）
     *
     * @generated from field: bytes jdm_document = 2;
     */
    jdmDocument: Uint8Array;
    /**
     * Sigstore 署名（ADR-RULE-001、registry に登録する署名）
     *
     * @generated from field: bytes sigstore_signature = 3;
     */
    sigstoreSignature: Uint8Array;
    /**
     * コミット ID（Git commit hash、JDM バージョン追跡用）
     *
     * @generated from field: string commit_hash = 4;
     */
    commitHash: string;
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 5;
     */
    context?: TenantContext;
    constructor(data?: PartialMessage<RegisterRuleRequest>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.decision.v1.RegisterRuleRequest";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): RegisterRuleRequest;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): RegisterRuleRequest;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): RegisterRuleRequest;
    static equals(a: RegisterRuleRequest | PlainMessage<RegisterRuleRequest> | undefined, b: RegisterRuleRequest | PlainMessage<RegisterRuleRequest> | undefined): boolean;
}
/**
 * RegisterRule 応答
 *
 * @generated from message k1s0.tier1.decision.v1.RegisterRuleResponse
 */
export declare class RegisterRuleResponse extends Message<RegisterRuleResponse> {
    /**
     * 採番されたバージョン（tenant + rule_id 内で一意、単調増加）
     *
     * @generated from field: string rule_version = 1;
     */
    ruleVersion: string;
    /**
     * 発効可能となる時刻（即時なら registered_at と同じ、Unix epoch ミリ秒）
     *
     * @generated from field: int64 effective_at_ms = 2;
     */
    effectiveAtMs: bigint;
    constructor(data?: PartialMessage<RegisterRuleResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.decision.v1.RegisterRuleResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): RegisterRuleResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): RegisterRuleResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): RegisterRuleResponse;
    static equals(a: RegisterRuleResponse | PlainMessage<RegisterRuleResponse> | undefined, b: RegisterRuleResponse | PlainMessage<RegisterRuleResponse> | undefined): boolean;
}
/**
 * ListVersions リクエスト
 *
 * @generated from message k1s0.tier1.decision.v1.ListVersionsRequest
 */
export declare class ListVersionsRequest extends Message<ListVersionsRequest> {
    /**
     * 対象ルール ID
     *
     * @generated from field: string rule_id = 1;
     */
    ruleId: string;
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 2;
     */
    context?: TenantContext;
    constructor(data?: PartialMessage<ListVersionsRequest>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.decision.v1.ListVersionsRequest";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): ListVersionsRequest;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): ListVersionsRequest;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): ListVersionsRequest;
    static equals(a: ListVersionsRequest | PlainMessage<ListVersionsRequest> | undefined, b: ListVersionsRequest | PlainMessage<ListVersionsRequest> | undefined): boolean;
}
/**
 * ListVersions 応答
 *
 * @generated from message k1s0.tier1.decision.v1.ListVersionsResponse
 */
export declare class ListVersionsResponse extends Message<ListVersionsResponse> {
    /**
     * バージョン一覧（登録時刻昇順）
     *
     * @generated from field: repeated k1s0.tier1.decision.v1.RuleVersionMeta versions = 1;
     */
    versions: RuleVersionMeta[];
    constructor(data?: PartialMessage<ListVersionsResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.decision.v1.ListVersionsResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): ListVersionsResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): ListVersionsResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): ListVersionsResponse;
    static equals(a: ListVersionsResponse | PlainMessage<ListVersionsResponse> | undefined, b: ListVersionsResponse | PlainMessage<ListVersionsResponse> | undefined): boolean;
}
/**
 * ルールバージョンのメタ情報
 *
 * @generated from message k1s0.tier1.decision.v1.RuleVersionMeta
 */
export declare class RuleVersionMeta extends Message<RuleVersionMeta> {
    /**
     * バージョン文字列
     *
     * @generated from field: string rule_version = 1;
     */
    ruleVersion: string;
    /**
     * Git commit hash
     *
     * @generated from field: string commit_hash = 2;
     */
    commitHash: string;
    /**
     * 登録時刻（Unix epoch ミリ秒）
     *
     * @generated from field: int64 registered_at_ms = 3;
     */
    registeredAtMs: bigint;
    /**
     * 登録者（subject 相当）
     *
     * @generated from field: string registered_by = 4;
     */
    registeredBy: string;
    /**
     * DEPRECATED 状態（非推奨のみ true、廃止後は ListVersions から消える）
     *
     * @generated from field: bool deprecated = 5;
     */
    deprecated: boolean;
    constructor(data?: PartialMessage<RuleVersionMeta>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.decision.v1.RuleVersionMeta";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): RuleVersionMeta;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): RuleVersionMeta;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): RuleVersionMeta;
    static equals(a: RuleVersionMeta | PlainMessage<RuleVersionMeta> | undefined, b: RuleVersionMeta | PlainMessage<RuleVersionMeta> | undefined): boolean;
}
/**
 * GetRule リクエスト
 *
 * @generated from message k1s0.tier1.decision.v1.GetRuleRequest
 */
export declare class GetRuleRequest extends Message<GetRuleRequest> {
    /**
     * 対象ルール ID
     *
     * @generated from field: string rule_id = 1;
     */
    ruleId: string;
    /**
     * 取得バージョン
     *
     * @generated from field: string rule_version = 2;
     */
    ruleVersion: string;
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 3;
     */
    context?: TenantContext;
    constructor(data?: PartialMessage<GetRuleRequest>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.decision.v1.GetRuleRequest";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): GetRuleRequest;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): GetRuleRequest;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): GetRuleRequest;
    static equals(a: GetRuleRequest | PlainMessage<GetRuleRequest> | undefined, b: GetRuleRequest | PlainMessage<GetRuleRequest> | undefined): boolean;
}
/**
 * GetRule 応答
 *
 * @generated from message k1s0.tier1.decision.v1.GetRuleResponse
 */
export declare class GetRuleResponse extends Message<GetRuleResponse> {
    /**
     * JDM 文書本体
     *
     * @generated from field: bytes jdm_document = 1;
     */
    jdmDocument: Uint8Array;
    /**
     * メタ情報
     *
     * @generated from field: k1s0.tier1.decision.v1.RuleVersionMeta meta = 2;
     */
    meta?: RuleVersionMeta;
    constructor(data?: PartialMessage<GetRuleResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.decision.v1.GetRuleResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): GetRuleResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): GetRuleResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): GetRuleResponse;
    static equals(a: GetRuleResponse | PlainMessage<GetRuleResponse> | undefined, b: GetRuleResponse | PlainMessage<GetRuleResponse> | undefined): boolean;
}
//# sourceMappingURL=decision_service_pb.d.ts.map