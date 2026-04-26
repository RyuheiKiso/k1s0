// 本ファイルは tier1 公開 Feature API の正式 proto。
// flagd / OpenFeature 互換の Feature Flag 評価と管理を提供する。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/11_Feature_API.md
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/11_Feature_API.md
//
// 関連要件: FR-T1-FEATURE-001〜004
// proto 構文宣言（proto3）
import { Message, proto3, protoInt64, Value } from "@bufbuild/protobuf";
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
export var FlagKind;
(function (FlagKind) {
    /**
     * リリース管理用（既定値、コード経路の段階公開）
     *
     * @generated from enum value: RELEASE = 0;
     */
    FlagKind[FlagKind["RELEASE"] = 0] = "RELEASE";
    /**
     * A/B テスト等の実験用
     *
     * @generated from enum value: EXPERIMENT = 1;
     */
    FlagKind[FlagKind["EXPERIMENT"] = 1] = "EXPERIMENT";
    /**
     * 運用上の緊急切替（Kill switch 含む）
     *
     * @generated from enum value: OPS = 2;
     */
    FlagKind[FlagKind["OPS"] = 2] = "OPS";
    /**
     * 権限制御（permission gate、Product Council 承認必須）
     *
     * @generated from enum value: PERMISSION = 3;
     */
    FlagKind[FlagKind["PERMISSION"] = 3] = "PERMISSION";
})(FlagKind || (FlagKind = {}));
// Retrieve enum metadata with: proto3.getEnumType(FlagKind)
proto3.util.setEnumType(FlagKind, "k1s0.tier1.feature.v1.FlagKind", [
    { no: 0, name: "RELEASE" },
    { no: 1, name: "EXPERIMENT" },
    { no: 2, name: "OPS" },
    { no: 3, name: "PERMISSION" },
]);
/**
 * Flag の戻り値型
 *
 * @generated from enum k1s0.tier1.feature.v1.FlagValueType
 */
export var FlagValueType;
(function (FlagValueType) {
    /**
     * 未指定（既定値、登録時に弾かれる）
     *
     * @generated from enum value: FLAG_VALUE_UNSPECIFIED = 0;
     */
    FlagValueType[FlagValueType["FLAG_VALUE_UNSPECIFIED"] = 0] = "FLAG_VALUE_UNSPECIFIED";
    /**
     * boolean 型
     *
     * @generated from enum value: FLAG_VALUE_BOOLEAN = 1;
     */
    FlagValueType[FlagValueType["FLAG_VALUE_BOOLEAN"] = 1] = "FLAG_VALUE_BOOLEAN";
    /**
     * string 型
     *
     * @generated from enum value: FLAG_VALUE_STRING = 2;
     */
    FlagValueType[FlagValueType["FLAG_VALUE_STRING"] = 2] = "FLAG_VALUE_STRING";
    /**
     * number 型
     *
     * @generated from enum value: FLAG_VALUE_NUMBER = 3;
     */
    FlagValueType[FlagValueType["FLAG_VALUE_NUMBER"] = 3] = "FLAG_VALUE_NUMBER";
    /**
     * object 型（任意 JSON）
     *
     * @generated from enum value: FLAG_VALUE_OBJECT = 4;
     */
    FlagValueType[FlagValueType["FLAG_VALUE_OBJECT"] = 4] = "FLAG_VALUE_OBJECT";
})(FlagValueType || (FlagValueType = {}));
// Retrieve enum metadata with: proto3.getEnumType(FlagValueType)
proto3.util.setEnumType(FlagValueType, "k1s0.tier1.feature.v1.FlagValueType", [
    { no: 0, name: "FLAG_VALUE_UNSPECIFIED" },
    { no: 1, name: "FLAG_VALUE_BOOLEAN" },
    { no: 2, name: "FLAG_VALUE_STRING" },
    { no: 3, name: "FLAG_VALUE_NUMBER" },
    { no: 4, name: "FLAG_VALUE_OBJECT" },
]);
/**
 * Flag の状態
 *
 * @generated from enum k1s0.tier1.feature.v1.FlagState
 */
export var FlagState;
(function (FlagState) {
    /**
     * 未指定（既定値、登録時に弾かれる）
     *
     * @generated from enum value: FLAG_STATE_UNSPECIFIED = 0;
     */
    FlagState[FlagState["UNSPECIFIED"] = 0] = "UNSPECIFIED";
    /**
     * 有効（評価可能）
     *
     * @generated from enum value: FLAG_STATE_ENABLED = 1;
     */
    FlagState[FlagState["ENABLED"] = 1] = "ENABLED";
    /**
     * 無効（評価は default_variant 固定）
     *
     * @generated from enum value: FLAG_STATE_DISABLED = 2;
     */
    FlagState[FlagState["DISABLED"] = 2] = "DISABLED";
    /**
     * 廃止（ListFlags から消える、Get は可能）
     *
     * @generated from enum value: FLAG_STATE_ARCHIVED = 3;
     */
    FlagState[FlagState["ARCHIVED"] = 3] = "ARCHIVED";
})(FlagState || (FlagState = {}));
// Retrieve enum metadata with: proto3.getEnumType(FlagState)
proto3.util.setEnumType(FlagState, "k1s0.tier1.feature.v1.FlagState", [
    { no: 0, name: "FLAG_STATE_UNSPECIFIED" },
    { no: 1, name: "FLAG_STATE_ENABLED" },
    { no: 2, name: "FLAG_STATE_DISABLED" },
    { no: 3, name: "FLAG_STATE_ARCHIVED" },
]);
/**
 * Flag 評価の共通入力
 *
 * @generated from message k1s0.tier1.feature.v1.EvaluateRequest
 */
export class EvaluateRequest extends Message {
    /**
     * Flag キー（命名規則: <tenant>.<component>.<feature>）
     *
     * @generated from field: string flag_key = 1;
     */
    flagKey = "";
    /**
     * 評価コンテキスト（targetingKey は subject と同一）
     *
     * @generated from field: map<string, string> evaluation_context = 2;
     */
    evaluationContext = {};
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
    static typeName = "k1s0.tier1.feature.v1.EvaluateRequest";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "flag_key", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 2, name: "evaluation_context", kind: "map", K: 9 /* ScalarType.STRING */, V: { kind: "scalar", T: 9 /* ScalarType.STRING */ } },
        { no: 3, name: "context", kind: "message", T: TenantContext },
    ]);
    static fromBinary(bytes, options) {
        return new EvaluateRequest().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new EvaluateRequest().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new EvaluateRequest().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(EvaluateRequest, a, b);
    }
}
/**
 * Flag 評価のメタ情報（OpenFeature の EvaluationDetails と整合）
 *
 * @generated from message k1s0.tier1.feature.v1.FlagMetadata
 */
export class FlagMetadata extends Message {
    /**
     * Flag 種別
     *
     * @generated from field: k1s0.tier1.feature.v1.FlagKind kind = 1;
     */
    kind = FlagKind.RELEASE;
    /**
     * バリアント名（有効化理由の参考）
     *
     * @generated from field: string variant = 2;
     */
    variant = "";
    /**
     * 評価の理由（DEFAULT / TARGETING_MATCH / SPLIT / ERROR）
     *
     * @generated from field: string reason = 3;
     */
    reason = "";
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.feature.v1.FlagMetadata";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "kind", kind: "enum", T: proto3.getEnumType(FlagKind) },
        { no: 2, name: "variant", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 3, name: "reason", kind: "scalar", T: 9 /* ScalarType.STRING */ },
    ]);
    static fromBinary(bytes, options) {
        return new FlagMetadata().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new FlagMetadata().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new FlagMetadata().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(FlagMetadata, a, b);
    }
}
/**
 * Boolean 評価応答
 *
 * @generated from message k1s0.tier1.feature.v1.BooleanResponse
 */
export class BooleanResponse extends Message {
    /**
     * 評価値
     *
     * @generated from field: bool value = 1;
     */
    value = false;
    /**
     * メタ情報
     *
     * @generated from field: k1s0.tier1.feature.v1.FlagMetadata metadata = 2;
     */
    metadata;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.feature.v1.BooleanResponse";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "value", kind: "scalar", T: 8 /* ScalarType.BOOL */ },
        { no: 2, name: "metadata", kind: "message", T: FlagMetadata },
    ]);
    static fromBinary(bytes, options) {
        return new BooleanResponse().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new BooleanResponse().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new BooleanResponse().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(BooleanResponse, a, b);
    }
}
/**
 * String 評価応答
 *
 * @generated from message k1s0.tier1.feature.v1.StringResponse
 */
export class StringResponse extends Message {
    /**
     * 評価値
     *
     * @generated from field: string value = 1;
     */
    value = "";
    /**
     * メタ情報
     *
     * @generated from field: k1s0.tier1.feature.v1.FlagMetadata metadata = 2;
     */
    metadata;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.feature.v1.StringResponse";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "value", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 2, name: "metadata", kind: "message", T: FlagMetadata },
    ]);
    static fromBinary(bytes, options) {
        return new StringResponse().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new StringResponse().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new StringResponse().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(StringResponse, a, b);
    }
}
/**
 * Number 評価応答
 *
 * @generated from message k1s0.tier1.feature.v1.NumberResponse
 */
export class NumberResponse extends Message {
    /**
     * 評価値
     *
     * @generated from field: double value = 1;
     */
    value = 0;
    /**
     * メタ情報
     *
     * @generated from field: k1s0.tier1.feature.v1.FlagMetadata metadata = 2;
     */
    metadata;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.feature.v1.NumberResponse";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "value", kind: "scalar", T: 1 /* ScalarType.DOUBLE */ },
        { no: 2, name: "metadata", kind: "message", T: FlagMetadata },
    ]);
    static fromBinary(bytes, options) {
        return new NumberResponse().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new NumberResponse().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new NumberResponse().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(NumberResponse, a, b);
    }
}
/**
 * Object 評価応答
 *
 * @generated from message k1s0.tier1.feature.v1.ObjectResponse
 */
export class ObjectResponse extends Message {
    /**
     * 評価値（JSON シリアライズ済み bytes）
     *
     * @generated from field: bytes value_json = 1;
     */
    valueJson = new Uint8Array(0);
    /**
     * メタ情報
     *
     * @generated from field: k1s0.tier1.feature.v1.FlagMetadata metadata = 2;
     */
    metadata;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.feature.v1.ObjectResponse";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "value_json", kind: "scalar", T: 12 /* ScalarType.BYTES */ },
        { no: 2, name: "metadata", kind: "message", T: FlagMetadata },
    ]);
    static fromBinary(bytes, options) {
        return new ObjectResponse().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new ObjectResponse().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new ObjectResponse().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(ObjectResponse, a, b);
    }
}
/**
 * flagd 互換の Flag 定義。k1s0 は OpenFeature / flagd 仕様に準拠。
 *
 * @generated from message k1s0.tier1.feature.v1.FlagDefinition
 */
export class FlagDefinition extends Message {
    /**
     * Flag キー（命名規則: <tenant>.<component>.<feature>）
     *
     * @generated from field: string flag_key = 1;
     */
    flagKey = "";
    /**
     * Flag 種別（RELEASE / EXPERIMENT / OPS / PERMISSION）
     *
     * @generated from field: k1s0.tier1.feature.v1.FlagKind kind = 2;
     */
    kind = FlagKind.RELEASE;
    /**
     * 戻り値型（boolean / string / number / object）
     *
     * @generated from field: k1s0.tier1.feature.v1.FlagValueType value_type = 3;
     */
    valueType = FlagValueType.FLAG_VALUE_UNSPECIFIED;
    /**
     * デフォルト variant の名前（下記 variants にキーが存在すること）
     *
     * @generated from field: string default_variant = 4;
     */
    defaultVariant = "";
    /**
     * variants 定義: variant 名 → 値（value_type に応じた JSON literal）
     *
     * @generated from field: map<string, google.protobuf.Value> variants = 5;
     */
    variants = {};
    /**
     * targeting ルール（先頭から評価、最初に match したもの採用）
     *
     * @generated from field: repeated k1s0.tier1.feature.v1.TargetingRule targeting = 6;
     */
    targeting = [];
    /**
     * 状態（ENABLED / DISABLED / ARCHIVED）
     *
     * @generated from field: k1s0.tier1.feature.v1.FlagState state = 7;
     */
    state = FlagState.UNSPECIFIED;
    /**
     * 説明（監査・運用者向け）
     *
     * @generated from field: string description = 8;
     */
    description = "";
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.feature.v1.FlagDefinition";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "flag_key", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 2, name: "kind", kind: "enum", T: proto3.getEnumType(FlagKind) },
        { no: 3, name: "value_type", kind: "enum", T: proto3.getEnumType(FlagValueType) },
        { no: 4, name: "default_variant", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 5, name: "variants", kind: "map", K: 9 /* ScalarType.STRING */, V: { kind: "message", T: Value } },
        { no: 6, name: "targeting", kind: "message", T: TargetingRule, repeated: true },
        { no: 7, name: "state", kind: "enum", T: proto3.getEnumType(FlagState) },
        { no: 8, name: "description", kind: "scalar", T: 9 /* ScalarType.STRING */ },
    ]);
    static fromBinary(bytes, options) {
        return new FlagDefinition().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new FlagDefinition().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new FlagDefinition().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(FlagDefinition, a, b);
    }
}
/**
 * targeting ルール（JsonLogic 互換、flagd 仕様準拠）。
 * 例: { "if": [ { "==": [{ "var": "userRole" }, "admin"] }, "blue-variant", "red-variant" ] }
 *
 * @generated from message k1s0.tier1.feature.v1.TargetingRule
 */
export class TargetingRule extends Message {
    /**
     * ルール ID（監査用、tenant+flag 内で一意）
     *
     * @generated from field: string rule_id = 1;
     */
    ruleId = "";
    /**
     * JsonLogic 式（bytes で保持、登録時に schema validator 通過必須）
     *
     * @generated from field: bytes json_logic_expr = 2;
     */
    jsonLogicExpr = new Uint8Array(0);
    /**
     * 評価成立時に返す variant 名
     *
     * @generated from field: string variant_if_match = 3;
     */
    variantIfMatch = "";
    /**
     * Fractional split（A/B テスト用、weights 合計 100 必須）
     *
     * @generated from field: repeated k1s0.tier1.feature.v1.FractionalSplit fractional = 4;
     */
    fractional = [];
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.feature.v1.TargetingRule";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "rule_id", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 2, name: "json_logic_expr", kind: "scalar", T: 12 /* ScalarType.BYTES */ },
        { no: 3, name: "variant_if_match", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 4, name: "fractional", kind: "message", T: FractionalSplit, repeated: true },
    ]);
    static fromBinary(bytes, options) {
        return new TargetingRule().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new TargetingRule().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new TargetingRule().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(TargetingRule, a, b);
    }
}
/**
 * Experiment 種別の Flag で A/B 比率を指定
 *
 * @generated from message k1s0.tier1.feature.v1.FractionalSplit
 */
export class FractionalSplit extends Message {
    /**
     * バリアント名
     *
     * @generated from field: string variant = 1;
     */
    variant = "";
    /**
     * 重み（0〜100、全エントリ合計 100 必須）
     *
     * @generated from field: int32 weight = 2;
     */
    weight = 0;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.feature.v1.FractionalSplit";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "variant", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 2, name: "weight", kind: "scalar", T: 5 /* ScalarType.INT32 */ },
    ]);
    static fromBinary(bytes, options) {
        return new FractionalSplit().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new FractionalSplit().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new FractionalSplit().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(FractionalSplit, a, b);
    }
}
/**
 * RegisterFlag リクエスト
 *
 * @generated from message k1s0.tier1.feature.v1.RegisterFlagRequest
 */
export class RegisterFlagRequest extends Message {
    /**
     * Flag 定義
     *
     * @generated from field: k1s0.tier1.feature.v1.FlagDefinition flag = 1;
     */
    flag;
    /**
     * 変更理由（permission 種別 Flag の場合 Product Council 承認番号必須）
     *
     * @generated from field: string change_reason = 2;
     */
    changeReason = "";
    /**
     * permission 種別時の承認番号（空値は permission 種別で reject）
     *
     * @generated from field: string approval_id = 3;
     */
    approvalId = "";
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
    static typeName = "k1s0.tier1.feature.v1.RegisterFlagRequest";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "flag", kind: "message", T: FlagDefinition },
        { no: 2, name: "change_reason", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 3, name: "approval_id", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 4, name: "context", kind: "message", T: TenantContext },
    ]);
    static fromBinary(bytes, options) {
        return new RegisterFlagRequest().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new RegisterFlagRequest().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new RegisterFlagRequest().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(RegisterFlagRequest, a, b);
    }
}
/**
 * RegisterFlag 応答
 *
 * @generated from message k1s0.tier1.feature.v1.RegisterFlagResponse
 */
export class RegisterFlagResponse extends Message {
    /**
     * バージョン（flag_key 内で単調増加）
     *
     * @generated from field: int64 version = 1;
     */
    version = protoInt64.zero;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.feature.v1.RegisterFlagResponse";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "version", kind: "scalar", T: 3 /* ScalarType.INT64 */ },
    ]);
    static fromBinary(bytes, options) {
        return new RegisterFlagResponse().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new RegisterFlagResponse().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new RegisterFlagResponse().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(RegisterFlagResponse, a, b);
    }
}
/**
 * GetFlag リクエスト
 *
 * @generated from message k1s0.tier1.feature.v1.GetFlagRequest
 */
export class GetFlagRequest extends Message {
    /**
     * 対象 Flag キー
     *
     * @generated from field: string flag_key = 1;
     */
    flagKey = "";
    /**
     * バージョン（省略時は最新）
     *
     * @generated from field: optional int64 version = 2;
     */
    version;
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
    static typeName = "k1s0.tier1.feature.v1.GetFlagRequest";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "flag_key", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 2, name: "version", kind: "scalar", T: 3 /* ScalarType.INT64 */, opt: true },
        { no: 3, name: "context", kind: "message", T: TenantContext },
    ]);
    static fromBinary(bytes, options) {
        return new GetFlagRequest().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new GetFlagRequest().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new GetFlagRequest().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(GetFlagRequest, a, b);
    }
}
/**
 * GetFlag 応答
 *
 * @generated from message k1s0.tier1.feature.v1.GetFlagResponse
 */
export class GetFlagResponse extends Message {
    /**
     * Flag 定義
     *
     * @generated from field: k1s0.tier1.feature.v1.FlagDefinition flag = 1;
     */
    flag;
    /**
     * バージョン
     *
     * @generated from field: int64 version = 2;
     */
    version = protoInt64.zero;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.feature.v1.GetFlagResponse";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "flag", kind: "message", T: FlagDefinition },
        { no: 2, name: "version", kind: "scalar", T: 3 /* ScalarType.INT64 */ },
    ]);
    static fromBinary(bytes, options) {
        return new GetFlagResponse().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new GetFlagResponse().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new GetFlagResponse().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(GetFlagResponse, a, b);
    }
}
/**
 * ListFlags リクエスト
 *
 * @generated from message k1s0.tier1.feature.v1.ListFlagsRequest
 */
export class ListFlagsRequest extends Message {
    /**
     * 種別フィルタ（省略で全種別）
     *
     * @generated from field: optional k1s0.tier1.feature.v1.FlagKind kind = 1;
     */
    kind;
    /**
     * 状態フィルタ（省略で ENABLED のみ）
     *
     * @generated from field: optional k1s0.tier1.feature.v1.FlagState state = 2;
     */
    state;
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
    static typeName = "k1s0.tier1.feature.v1.ListFlagsRequest";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "kind", kind: "enum", T: proto3.getEnumType(FlagKind), opt: true },
        { no: 2, name: "state", kind: "enum", T: proto3.getEnumType(FlagState), opt: true },
        { no: 3, name: "context", kind: "message", T: TenantContext },
    ]);
    static fromBinary(bytes, options) {
        return new ListFlagsRequest().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new ListFlagsRequest().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new ListFlagsRequest().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(ListFlagsRequest, a, b);
    }
}
/**
 * ListFlags 応答
 *
 * @generated from message k1s0.tier1.feature.v1.ListFlagsResponse
 */
export class ListFlagsResponse extends Message {
    /**
     * Flag 定義列
     *
     * @generated from field: repeated k1s0.tier1.feature.v1.FlagDefinition flags = 1;
     */
    flags = [];
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.feature.v1.ListFlagsResponse";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "flags", kind: "message", T: FlagDefinition, repeated: true },
    ]);
    static fromBinary(bytes, options) {
        return new ListFlagsResponse().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new ListFlagsResponse().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new ListFlagsResponse().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(ListFlagsResponse, a, b);
    }
}
//# sourceMappingURL=feature_service_pb.js.map