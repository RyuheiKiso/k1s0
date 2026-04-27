import type { BinaryReadOptions, FieldList, JsonReadOptions, JsonValue, PartialMessage, PlainMessage } from "@bufbuild/protobuf";
import { Message, proto3, Value } from "@bufbuild/protobuf";
import { TenantContext } from "../../common/v1/common_pb.js";
/**
 * Flag の種別（OpenFeature / k1s0 固有）。
 * 注: 正典 IDL は zero value を `RELEASE = 0` と定義しているため、
 *     buf STANDARD lint の ENUM_ZERO_VALUE_SUFFIX / ENUM_VALUE_PREFIX を ignore する。
 * buf:lint:ignore ENUM_ZERO_VALUE_SUFFIX
 * buf:lint:ignore ENUM_VALUE_PREFIX
 *
 * @generated from enum k1s0.tier1.feature.v1.FlagKind
 */
export declare enum FlagKind {
    /**
     * リリース管理用（既定値、コード経路の段階公開）
     *
     * @generated from enum value: RELEASE = 0;
     */
    RELEASE = 0,
    /**
     * A/B テスト等の実験用
     *
     * @generated from enum value: EXPERIMENT = 1;
     */
    EXPERIMENT = 1,
    /**
     * 運用上の緊急切替（Kill switch 含む）
     *
     * @generated from enum value: OPS = 2;
     */
    OPS = 2,
    /**
     * 権限制御（permission gate、Product Council 承認必須）
     *
     * @generated from enum value: PERMISSION = 3;
     */
    PERMISSION = 3
}
/**
 * Flag の戻り値型
 *
 * @generated from enum k1s0.tier1.feature.v1.FlagValueType
 */
export declare enum FlagValueType {
    /**
     * 未指定（既定値、登録時に弾かれる）
     *
     * @generated from enum value: FLAG_VALUE_UNSPECIFIED = 0;
     */
    FLAG_VALUE_UNSPECIFIED = 0,
    /**
     * boolean 型
     *
     * @generated from enum value: FLAG_VALUE_BOOLEAN = 1;
     */
    FLAG_VALUE_BOOLEAN = 1,
    /**
     * string 型
     *
     * @generated from enum value: FLAG_VALUE_STRING = 2;
     */
    FLAG_VALUE_STRING = 2,
    /**
     * number 型
     *
     * @generated from enum value: FLAG_VALUE_NUMBER = 3;
     */
    FLAG_VALUE_NUMBER = 3,
    /**
     * object 型（任意 JSON）
     *
     * @generated from enum value: FLAG_VALUE_OBJECT = 4;
     */
    FLAG_VALUE_OBJECT = 4
}
/**
 * Flag の状態
 *
 * @generated from enum k1s0.tier1.feature.v1.FlagState
 */
export declare enum FlagState {
    /**
     * 未指定（既定値、登録時に弾かれる）
     *
     * @generated from enum value: FLAG_STATE_UNSPECIFIED = 0;
     */
    UNSPECIFIED = 0,
    /**
     * 有効（評価可能）
     *
     * @generated from enum value: FLAG_STATE_ENABLED = 1;
     */
    ENABLED = 1,
    /**
     * 無効（評価は default_variant 固定）
     *
     * @generated from enum value: FLAG_STATE_DISABLED = 2;
     */
    DISABLED = 2,
    /**
     * 廃止（ListFlags から消える、Get は可能）
     *
     * @generated from enum value: FLAG_STATE_ARCHIVED = 3;
     */
    ARCHIVED = 3
}
/**
 * Flag 評価の共通入力
 *
 * @generated from message k1s0.tier1.feature.v1.EvaluateRequest
 */
export declare class EvaluateRequest extends Message<EvaluateRequest> {
    /**
     * Flag キー（命名規則: <tenant>.<component>.<feature>）
     *
     * @generated from field: string flag_key = 1;
     */
    flagKey: string;
    /**
     * 評価コンテキスト（targetingKey は subject と同一）
     *
     * @generated from field: map<string, string> evaluation_context = 2;
     */
    evaluationContext: {
        [key: string]: string;
    };
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 3;
     */
    context?: TenantContext;
    constructor(data?: PartialMessage<EvaluateRequest>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.feature.v1.EvaluateRequest";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): EvaluateRequest;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): EvaluateRequest;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): EvaluateRequest;
    static equals(a: EvaluateRequest | PlainMessage<EvaluateRequest> | undefined, b: EvaluateRequest | PlainMessage<EvaluateRequest> | undefined): boolean;
}
/**
 * Flag 評価のメタ情報（OpenFeature の EvaluationDetails と整合）
 *
 * @generated from message k1s0.tier1.feature.v1.FlagMetadata
 */
export declare class FlagMetadata extends Message<FlagMetadata> {
    /**
     * Flag 種別
     *
     * @generated from field: k1s0.tier1.feature.v1.FlagKind kind = 1;
     */
    kind: FlagKind;
    /**
     * バリアント名（有効化理由の参考）
     *
     * @generated from field: string variant = 2;
     */
    variant: string;
    /**
     * 評価の理由（DEFAULT / TARGETING_MATCH / SPLIT / ERROR）
     *
     * @generated from field: string reason = 3;
     */
    reason: string;
    constructor(data?: PartialMessage<FlagMetadata>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.feature.v1.FlagMetadata";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): FlagMetadata;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): FlagMetadata;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): FlagMetadata;
    static equals(a: FlagMetadata | PlainMessage<FlagMetadata> | undefined, b: FlagMetadata | PlainMessage<FlagMetadata> | undefined): boolean;
}
/**
 * Boolean 評価応答
 *
 * @generated from message k1s0.tier1.feature.v1.BooleanResponse
 */
export declare class BooleanResponse extends Message<BooleanResponse> {
    /**
     * 評価値
     *
     * @generated from field: bool value = 1;
     */
    value: boolean;
    /**
     * メタ情報
     *
     * @generated from field: k1s0.tier1.feature.v1.FlagMetadata metadata = 2;
     */
    metadata?: FlagMetadata;
    constructor(data?: PartialMessage<BooleanResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.feature.v1.BooleanResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): BooleanResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): BooleanResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): BooleanResponse;
    static equals(a: BooleanResponse | PlainMessage<BooleanResponse> | undefined, b: BooleanResponse | PlainMessage<BooleanResponse> | undefined): boolean;
}
/**
 * String 評価応答
 *
 * @generated from message k1s0.tier1.feature.v1.StringResponse
 */
export declare class StringResponse extends Message<StringResponse> {
    /**
     * 評価値
     *
     * @generated from field: string value = 1;
     */
    value: string;
    /**
     * メタ情報
     *
     * @generated from field: k1s0.tier1.feature.v1.FlagMetadata metadata = 2;
     */
    metadata?: FlagMetadata;
    constructor(data?: PartialMessage<StringResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.feature.v1.StringResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): StringResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): StringResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): StringResponse;
    static equals(a: StringResponse | PlainMessage<StringResponse> | undefined, b: StringResponse | PlainMessage<StringResponse> | undefined): boolean;
}
/**
 * Number 評価応答
 *
 * @generated from message k1s0.tier1.feature.v1.NumberResponse
 */
export declare class NumberResponse extends Message<NumberResponse> {
    /**
     * 評価値
     *
     * @generated from field: double value = 1;
     */
    value: number;
    /**
     * メタ情報
     *
     * @generated from field: k1s0.tier1.feature.v1.FlagMetadata metadata = 2;
     */
    metadata?: FlagMetadata;
    constructor(data?: PartialMessage<NumberResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.feature.v1.NumberResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): NumberResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): NumberResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): NumberResponse;
    static equals(a: NumberResponse | PlainMessage<NumberResponse> | undefined, b: NumberResponse | PlainMessage<NumberResponse> | undefined): boolean;
}
/**
 * Object 評価応答
 *
 * @generated from message k1s0.tier1.feature.v1.ObjectResponse
 */
export declare class ObjectResponse extends Message<ObjectResponse> {
    /**
     * 評価値（JSON シリアライズ済み bytes）
     *
     * @generated from field: bytes value_json = 1;
     */
    valueJson: Uint8Array;
    /**
     * メタ情報
     *
     * @generated from field: k1s0.tier1.feature.v1.FlagMetadata metadata = 2;
     */
    metadata?: FlagMetadata;
    constructor(data?: PartialMessage<ObjectResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.feature.v1.ObjectResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): ObjectResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): ObjectResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): ObjectResponse;
    static equals(a: ObjectResponse | PlainMessage<ObjectResponse> | undefined, b: ObjectResponse | PlainMessage<ObjectResponse> | undefined): boolean;
}
/**
 * flagd 互換の Flag 定義。k1s0 は OpenFeature / flagd 仕様に準拠。
 *
 * @generated from message k1s0.tier1.feature.v1.FlagDefinition
 */
export declare class FlagDefinition extends Message<FlagDefinition> {
    /**
     * Flag キー（命名規則: <tenant>.<component>.<feature>）
     *
     * @generated from field: string flag_key = 1;
     */
    flagKey: string;
    /**
     * Flag 種別（RELEASE / EXPERIMENT / OPS / PERMISSION）
     *
     * @generated from field: k1s0.tier1.feature.v1.FlagKind kind = 2;
     */
    kind: FlagKind;
    /**
     * 戻り値型（boolean / string / number / object）
     *
     * @generated from field: k1s0.tier1.feature.v1.FlagValueType value_type = 3;
     */
    valueType: FlagValueType;
    /**
     * デフォルト variant の名前（下記 variants にキーが存在すること）
     *
     * @generated from field: string default_variant = 4;
     */
    defaultVariant: string;
    /**
     * variants 定義: variant 名 → 値（value_type に応じた JSON literal）
     *
     * @generated from field: map<string, google.protobuf.Value> variants = 5;
     */
    variants: {
        [key: string]: Value;
    };
    /**
     * targeting ルール（先頭から評価、最初に match したもの採用）
     *
     * @generated from field: repeated k1s0.tier1.feature.v1.TargetingRule targeting = 6;
     */
    targeting: TargetingRule[];
    /**
     * 状態（ENABLED / DISABLED / ARCHIVED）
     *
     * @generated from field: k1s0.tier1.feature.v1.FlagState state = 7;
     */
    state: FlagState;
    /**
     * 説明（監査・運用者向け）
     *
     * @generated from field: string description = 8;
     */
    description: string;
    constructor(data?: PartialMessage<FlagDefinition>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.feature.v1.FlagDefinition";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): FlagDefinition;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): FlagDefinition;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): FlagDefinition;
    static equals(a: FlagDefinition | PlainMessage<FlagDefinition> | undefined, b: FlagDefinition | PlainMessage<FlagDefinition> | undefined): boolean;
}
/**
 * targeting ルール（JsonLogic 互換、flagd 仕様準拠）。
 * 例: { "if": [ { "==": [{ "var": "userRole" }, "admin"] }, "blue-variant", "red-variant" ] }
 *
 * @generated from message k1s0.tier1.feature.v1.TargetingRule
 */
export declare class TargetingRule extends Message<TargetingRule> {
    /**
     * ルール ID（監査用、tenant+flag 内で一意）
     *
     * @generated from field: string rule_id = 1;
     */
    ruleId: string;
    /**
     * JsonLogic 式（bytes で保持、登録時に schema validator 通過必須）
     *
     * @generated from field: bytes json_logic_expr = 2;
     */
    jsonLogicExpr: Uint8Array;
    /**
     * 評価成立時に返す variant 名
     *
     * @generated from field: string variant_if_match = 3;
     */
    variantIfMatch: string;
    /**
     * Fractional split（A/B テスト用、weights 合計 100 必須）
     *
     * @generated from field: repeated k1s0.tier1.feature.v1.FractionalSplit fractional = 4;
     */
    fractional: FractionalSplit[];
    constructor(data?: PartialMessage<TargetingRule>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.feature.v1.TargetingRule";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): TargetingRule;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): TargetingRule;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): TargetingRule;
    static equals(a: TargetingRule | PlainMessage<TargetingRule> | undefined, b: TargetingRule | PlainMessage<TargetingRule> | undefined): boolean;
}
/**
 * Experiment 種別の Flag で A/B 比率を指定
 *
 * @generated from message k1s0.tier1.feature.v1.FractionalSplit
 */
export declare class FractionalSplit extends Message<FractionalSplit> {
    /**
     * バリアント名
     *
     * @generated from field: string variant = 1;
     */
    variant: string;
    /**
     * 重み（0〜100、全エントリ合計 100 必須）
     *
     * @generated from field: int32 weight = 2;
     */
    weight: number;
    constructor(data?: PartialMessage<FractionalSplit>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.feature.v1.FractionalSplit";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): FractionalSplit;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): FractionalSplit;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): FractionalSplit;
    static equals(a: FractionalSplit | PlainMessage<FractionalSplit> | undefined, b: FractionalSplit | PlainMessage<FractionalSplit> | undefined): boolean;
}
/**
 * RegisterFlag リクエスト
 *
 * @generated from message k1s0.tier1.feature.v1.RegisterFlagRequest
 */
export declare class RegisterFlagRequest extends Message<RegisterFlagRequest> {
    /**
     * Flag 定義
     *
     * @generated from field: k1s0.tier1.feature.v1.FlagDefinition flag = 1;
     */
    flag?: FlagDefinition;
    /**
     * 変更理由（permission 種別 Flag の場合 Product Council 承認番号必須）
     *
     * @generated from field: string change_reason = 2;
     */
    changeReason: string;
    /**
     * permission 種別時の承認番号（空値は permission 種別で reject）
     *
     * @generated from field: string approval_id = 3;
     */
    approvalId: string;
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 4;
     */
    context?: TenantContext;
    constructor(data?: PartialMessage<RegisterFlagRequest>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.feature.v1.RegisterFlagRequest";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): RegisterFlagRequest;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): RegisterFlagRequest;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): RegisterFlagRequest;
    static equals(a: RegisterFlagRequest | PlainMessage<RegisterFlagRequest> | undefined, b: RegisterFlagRequest | PlainMessage<RegisterFlagRequest> | undefined): boolean;
}
/**
 * RegisterFlag 応答
 *
 * @generated from message k1s0.tier1.feature.v1.RegisterFlagResponse
 */
export declare class RegisterFlagResponse extends Message<RegisterFlagResponse> {
    /**
     * バージョン（flag_key 内で単調増加）
     *
     * @generated from field: int64 version = 1;
     */
    version: bigint;
    constructor(data?: PartialMessage<RegisterFlagResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.feature.v1.RegisterFlagResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): RegisterFlagResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): RegisterFlagResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): RegisterFlagResponse;
    static equals(a: RegisterFlagResponse | PlainMessage<RegisterFlagResponse> | undefined, b: RegisterFlagResponse | PlainMessage<RegisterFlagResponse> | undefined): boolean;
}
/**
 * GetFlag リクエスト
 *
 * @generated from message k1s0.tier1.feature.v1.GetFlagRequest
 */
export declare class GetFlagRequest extends Message<GetFlagRequest> {
    /**
     * 対象 Flag キー
     *
     * @generated from field: string flag_key = 1;
     */
    flagKey: string;
    /**
     * バージョン（省略時は最新）
     *
     * @generated from field: optional int64 version = 2;
     */
    version?: bigint;
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 3;
     */
    context?: TenantContext;
    constructor(data?: PartialMessage<GetFlagRequest>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.feature.v1.GetFlagRequest";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): GetFlagRequest;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): GetFlagRequest;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): GetFlagRequest;
    static equals(a: GetFlagRequest | PlainMessage<GetFlagRequest> | undefined, b: GetFlagRequest | PlainMessage<GetFlagRequest> | undefined): boolean;
}
/**
 * GetFlag 応答
 *
 * @generated from message k1s0.tier1.feature.v1.GetFlagResponse
 */
export declare class GetFlagResponse extends Message<GetFlagResponse> {
    /**
     * Flag 定義
     *
     * @generated from field: k1s0.tier1.feature.v1.FlagDefinition flag = 1;
     */
    flag?: FlagDefinition;
    /**
     * バージョン
     *
     * @generated from field: int64 version = 2;
     */
    version: bigint;
    constructor(data?: PartialMessage<GetFlagResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.feature.v1.GetFlagResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): GetFlagResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): GetFlagResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): GetFlagResponse;
    static equals(a: GetFlagResponse | PlainMessage<GetFlagResponse> | undefined, b: GetFlagResponse | PlainMessage<GetFlagResponse> | undefined): boolean;
}
/**
 * ListFlags リクエスト
 *
 * @generated from message k1s0.tier1.feature.v1.ListFlagsRequest
 */
export declare class ListFlagsRequest extends Message<ListFlagsRequest> {
    /**
     * 種別フィルタ（省略で全種別）
     *
     * @generated from field: optional k1s0.tier1.feature.v1.FlagKind kind = 1;
     */
    kind?: FlagKind;
    /**
     * 状態フィルタ（省略で ENABLED のみ）
     *
     * @generated from field: optional k1s0.tier1.feature.v1.FlagState state = 2;
     */
    state?: FlagState;
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 3;
     */
    context?: TenantContext;
    constructor(data?: PartialMessage<ListFlagsRequest>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.feature.v1.ListFlagsRequest";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): ListFlagsRequest;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): ListFlagsRequest;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): ListFlagsRequest;
    static equals(a: ListFlagsRequest | PlainMessage<ListFlagsRequest> | undefined, b: ListFlagsRequest | PlainMessage<ListFlagsRequest> | undefined): boolean;
}
/**
 * ListFlags 応答
 *
 * @generated from message k1s0.tier1.feature.v1.ListFlagsResponse
 */
export declare class ListFlagsResponse extends Message<ListFlagsResponse> {
    /**
     * Flag 定義列
     *
     * @generated from field: repeated k1s0.tier1.feature.v1.FlagDefinition flags = 1;
     */
    flags: FlagDefinition[];
    constructor(data?: PartialMessage<ListFlagsResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.feature.v1.ListFlagsResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): ListFlagsResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): ListFlagsResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): ListFlagsResponse;
    static equals(a: ListFlagsResponse | PlainMessage<ListFlagsResponse> | undefined, b: ListFlagsResponse | PlainMessage<ListFlagsResponse> | undefined): boolean;
}
//# sourceMappingURL=feature_service_pb.d.ts.map