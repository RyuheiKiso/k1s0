// 本ファイルは tier1 公開 Decision API の正式 proto。
// ZEN Engine による JDM（JSON Decision Model）ルール評価とルール文書管理を提供する。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/09_Decision_API.md
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/09_Decision_API.md
//
// 関連要件: FR-T1-DECISION-001〜004
// proto 構文宣言（proto3）
import { Message, proto3, protoInt64 } from "@bufbuild/protobuf";
import { TenantContext } from "../../common/v1/common_pb.js";
/**
 * Evaluate リクエスト
 *
 * @generated from message k1s0.tier1.decision.v1.EvaluateRequest
 */
export class EvaluateRequest extends Message {
    /**
     * ルール ID（tier2 で登録した JDM 文書の識別子）
     *
     * @generated from field: string rule_id = 1;
     */
    ruleId = "";
    /**
     * ルールバージョン（省略時は最新有効）
     *
     * @generated from field: string rule_version = 2;
     */
    ruleVersion = "";
    /**
     * 入力（JDM の context に相当、任意 JSON）
     *
     * @generated from field: bytes input_json = 3;
     */
    inputJson = new Uint8Array(0);
    /**
     * trace 情報を返すか（デバッグ用、PII を含む可能性あり）
     *
     * @generated from field: bool include_trace = 4;
     */
    includeTrace = false;
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 5;
     */
    context;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.decision.v1.EvaluateRequest";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "rule_id", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 2, name: "rule_version", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 3, name: "input_json", kind: "scalar", T: 12 /* ScalarType.BYTES */ },
        { no: 4, name: "include_trace", kind: "scalar", T: 8 /* ScalarType.BOOL */ },
        { no: 5, name: "context", kind: "message", T: TenantContext },
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
 * Evaluate 応答
 *
 * @generated from message k1s0.tier1.decision.v1.EvaluateResponse
 */
export class EvaluateResponse extends Message {
    /**
     * 出力（JDM 評価結果、任意 JSON）
     *
     * @generated from field: bytes output_json = 1;
     */
    outputJson = new Uint8Array(0);
    /**
     * 評価されたノードのトレース（include_trace=true の時のみ、空 bytes）
     *
     * @generated from field: bytes trace_json = 2;
     */
    traceJson = new Uint8Array(0);
    /**
     * 評価にかかった時間（マイクロ秒）
     *
     * @generated from field: int64 elapsed_us = 3;
     */
    elapsedUs = protoInt64.zero;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.decision.v1.EvaluateResponse";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "output_json", kind: "scalar", T: 12 /* ScalarType.BYTES */ },
        { no: 2, name: "trace_json", kind: "scalar", T: 12 /* ScalarType.BYTES */ },
        { no: 3, name: "elapsed_us", kind: "scalar", T: 3 /* ScalarType.INT64 */ },
    ]);
    static fromBinary(bytes, options) {
        return new EvaluateResponse().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new EvaluateResponse().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new EvaluateResponse().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(EvaluateResponse, a, b);
    }
}
/**
 * BatchEvaluate リクエスト
 *
 * @generated from message k1s0.tier1.decision.v1.BatchEvaluateRequest
 */
export class BatchEvaluateRequest extends Message {
    /**
     * ルール ID（全入力で共通）
     *
     * @generated from field: string rule_id = 1;
     */
    ruleId = "";
    /**
     * ルールバージョン
     *
     * @generated from field: string rule_version = 2;
     */
    ruleVersion = "";
    /**
     * 入力 JSON 列（順序を保って評価される）
     *
     * @generated from field: repeated bytes inputs_json = 3;
     */
    inputsJson = [];
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
    static typeName = "k1s0.tier1.decision.v1.BatchEvaluateRequest";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "rule_id", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 2, name: "rule_version", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 3, name: "inputs_json", kind: "scalar", T: 12 /* ScalarType.BYTES */, repeated: true },
        { no: 4, name: "context", kind: "message", T: TenantContext },
    ]);
    static fromBinary(bytes, options) {
        return new BatchEvaluateRequest().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new BatchEvaluateRequest().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new BatchEvaluateRequest().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(BatchEvaluateRequest, a, b);
    }
}
/**
 * BatchEvaluate 応答
 *
 * @generated from message k1s0.tier1.decision.v1.BatchEvaluateResponse
 */
export class BatchEvaluateResponse extends Message {
    /**
     * 出力 JSON 列（inputs_json と同じ順序）
     *
     * @generated from field: repeated bytes outputs_json = 1;
     */
    outputsJson = [];
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.decision.v1.BatchEvaluateResponse";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "outputs_json", kind: "scalar", T: 12 /* ScalarType.BYTES */, repeated: true },
    ]);
    static fromBinary(bytes, options) {
        return new BatchEvaluateResponse().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new BatchEvaluateResponse().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new BatchEvaluateResponse().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(BatchEvaluateResponse, a, b);
    }
}
/**
 * RegisterRule リクエスト
 *
 * @generated from message k1s0.tier1.decision.v1.RegisterRuleRequest
 */
export class RegisterRuleRequest extends Message {
    /**
     * ルール ID（tenant 内で一意）
     *
     * @generated from field: string rule_id = 1;
     */
    ruleId = "";
    /**
     * JDM 文書（前節 JSON Schema に準拠、UTF-8 JSON）
     *
     * @generated from field: bytes jdm_document = 2;
     */
    jdmDocument = new Uint8Array(0);
    /**
     * Sigstore 署名（ADR-RULE-001、registry に登録する署名）
     *
     * @generated from field: bytes sigstore_signature = 3;
     */
    sigstoreSignature = new Uint8Array(0);
    /**
     * コミット ID（Git commit hash、JDM バージョン追跡用）
     *
     * @generated from field: string commit_hash = 4;
     */
    commitHash = "";
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 5;
     */
    context;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.decision.v1.RegisterRuleRequest";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "rule_id", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 2, name: "jdm_document", kind: "scalar", T: 12 /* ScalarType.BYTES */ },
        { no: 3, name: "sigstore_signature", kind: "scalar", T: 12 /* ScalarType.BYTES */ },
        { no: 4, name: "commit_hash", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 5, name: "context", kind: "message", T: TenantContext },
    ]);
    static fromBinary(bytes, options) {
        return new RegisterRuleRequest().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new RegisterRuleRequest().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new RegisterRuleRequest().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(RegisterRuleRequest, a, b);
    }
}
/**
 * RegisterRule 応答
 *
 * @generated from message k1s0.tier1.decision.v1.RegisterRuleResponse
 */
export class RegisterRuleResponse extends Message {
    /**
     * 採番されたバージョン（tenant + rule_id 内で一意、単調増加）
     *
     * @generated from field: string rule_version = 1;
     */
    ruleVersion = "";
    /**
     * 発効可能となる時刻（即時なら registered_at と同じ、Unix epoch ミリ秒）
     *
     * @generated from field: int64 effective_at_ms = 2;
     */
    effectiveAtMs = protoInt64.zero;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.decision.v1.RegisterRuleResponse";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "rule_version", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 2, name: "effective_at_ms", kind: "scalar", T: 3 /* ScalarType.INT64 */ },
    ]);
    static fromBinary(bytes, options) {
        return new RegisterRuleResponse().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new RegisterRuleResponse().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new RegisterRuleResponse().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(RegisterRuleResponse, a, b);
    }
}
/**
 * ListVersions リクエスト
 *
 * @generated from message k1s0.tier1.decision.v1.ListVersionsRequest
 */
export class ListVersionsRequest extends Message {
    /**
     * 対象ルール ID
     *
     * @generated from field: string rule_id = 1;
     */
    ruleId = "";
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
    static typeName = "k1s0.tier1.decision.v1.ListVersionsRequest";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "rule_id", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 2, name: "context", kind: "message", T: TenantContext },
    ]);
    static fromBinary(bytes, options) {
        return new ListVersionsRequest().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new ListVersionsRequest().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new ListVersionsRequest().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(ListVersionsRequest, a, b);
    }
}
/**
 * ListVersions 応答
 *
 * @generated from message k1s0.tier1.decision.v1.ListVersionsResponse
 */
export class ListVersionsResponse extends Message {
    /**
     * バージョン一覧（登録時刻昇順）
     *
     * @generated from field: repeated k1s0.tier1.decision.v1.RuleVersionMeta versions = 1;
     */
    versions = [];
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.decision.v1.ListVersionsResponse";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "versions", kind: "message", T: RuleVersionMeta, repeated: true },
    ]);
    static fromBinary(bytes, options) {
        return new ListVersionsResponse().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new ListVersionsResponse().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new ListVersionsResponse().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(ListVersionsResponse, a, b);
    }
}
/**
 * ルールバージョンのメタ情報
 *
 * @generated from message k1s0.tier1.decision.v1.RuleVersionMeta
 */
export class RuleVersionMeta extends Message {
    /**
     * バージョン文字列
     *
     * @generated from field: string rule_version = 1;
     */
    ruleVersion = "";
    /**
     * Git commit hash
     *
     * @generated from field: string commit_hash = 2;
     */
    commitHash = "";
    /**
     * 登録時刻（Unix epoch ミリ秒）
     *
     * @generated from field: int64 registered_at_ms = 3;
     */
    registeredAtMs = protoInt64.zero;
    /**
     * 登録者（subject 相当）
     *
     * @generated from field: string registered_by = 4;
     */
    registeredBy = "";
    /**
     * DEPRECATED 状態（非推奨のみ true、廃止後は ListVersions から消える）
     *
     * @generated from field: bool deprecated = 5;
     */
    deprecated = false;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.decision.v1.RuleVersionMeta";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "rule_version", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 2, name: "commit_hash", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 3, name: "registered_at_ms", kind: "scalar", T: 3 /* ScalarType.INT64 */ },
        { no: 4, name: "registered_by", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 5, name: "deprecated", kind: "scalar", T: 8 /* ScalarType.BOOL */ },
    ]);
    static fromBinary(bytes, options) {
        return new RuleVersionMeta().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new RuleVersionMeta().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new RuleVersionMeta().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(RuleVersionMeta, a, b);
    }
}
/**
 * GetRule リクエスト
 *
 * @generated from message k1s0.tier1.decision.v1.GetRuleRequest
 */
export class GetRuleRequest extends Message {
    /**
     * 対象ルール ID
     *
     * @generated from field: string rule_id = 1;
     */
    ruleId = "";
    /**
     * 取得バージョン
     *
     * @generated from field: string rule_version = 2;
     */
    ruleVersion = "";
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
    static typeName = "k1s0.tier1.decision.v1.GetRuleRequest";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "rule_id", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 2, name: "rule_version", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 3, name: "context", kind: "message", T: TenantContext },
    ]);
    static fromBinary(bytes, options) {
        return new GetRuleRequest().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new GetRuleRequest().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new GetRuleRequest().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(GetRuleRequest, a, b);
    }
}
/**
 * GetRule 応答
 *
 * @generated from message k1s0.tier1.decision.v1.GetRuleResponse
 */
export class GetRuleResponse extends Message {
    /**
     * JDM 文書本体
     *
     * @generated from field: bytes jdm_document = 1;
     */
    jdmDocument = new Uint8Array(0);
    /**
     * メタ情報
     *
     * @generated from field: k1s0.tier1.decision.v1.RuleVersionMeta meta = 2;
     */
    meta;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.decision.v1.GetRuleResponse";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "jdm_document", kind: "scalar", T: 12 /* ScalarType.BYTES */ },
        { no: 2, name: "meta", kind: "message", T: RuleVersionMeta },
    ]);
    static fromBinary(bytes, options) {
        return new GetRuleResponse().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new GetRuleResponse().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new GetRuleResponse().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(GetRuleResponse, a, b);
    }
}
//# sourceMappingURL=decision_service_pb.js.map