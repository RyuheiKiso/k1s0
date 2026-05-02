// 本ファイルは tier1 公開 PII API の正式 proto。
// PII（個人情報）の検出（Classify）とマスキング（Mask）を提供する。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/10_Audit_Pii_API.md（PiiService 部）
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/10_Audit_Pii_API.md
//
// 関連要件: FR-T1-PII-001〜002
//
// 担当 Pod: t1-pii（Rust 純関数実装、ステートレス、DS-SW-COMP-009）
//
// 注: 正典 IDL では AuditService と PiiService を 1 ファイル（package
//     k1s0.tier1.audit.v1）にまとめているが、ディレクトリ設計と Pod 構成に従い、
//     本リポジトリでは pii.v1 パッケージに分離する。
// proto 構文宣言（proto3）
import { Message, proto3 } from "@bufbuild/protobuf";
import { TenantContext } from "../../common/v1/common_pb.js";
/**
 * Classify リクエスト
 *
 * @generated from message k1s0.tier1.pii.v1.ClassifyRequest
 */
export class ClassifyRequest extends Message {
    /**
     * 判定対象テキスト
     *
     * @generated from field: string text = 1;
     */
    text = "";
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
    static typeName = "k1s0.tier1.pii.v1.ClassifyRequest";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "text", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 2, name: "context", kind: "message", T: TenantContext },
    ]);
    static fromBinary(bytes, options) {
        return new ClassifyRequest().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new ClassifyRequest().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new ClassifyRequest().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(ClassifyRequest, a, b);
    }
}
/**
 * PII 検出結果の 1 件
 *
 * @generated from message k1s0.tier1.pii.v1.PiiFinding
 */
export class PiiFinding extends Message {
    /**
     * 検出された PII 種別（NAME / EMAIL / PHONE / MYNUMBER / CREDITCARD 等）
     *
     * @generated from field: string type = 1;
     */
    type = "";
    /**
     * 文字列内の開始位置（0 始まり、UTF-8 byte 単位ではなく文字単位）
     *
     * @generated from field: int32 start = 2;
     */
    start = 0;
    /**
     * 文字列内の終了位置（exclusive）
     *
     * @generated from field: int32 end = 3;
     */
    end = 0;
    /**
     * 信頼度（0.0〜1.0）
     *
     * @generated from field: double confidence = 4;
     */
    confidence = 0;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.pii.v1.PiiFinding";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "type", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 2, name: "start", kind: "scalar", T: 5 /* ScalarType.INT32 */ },
        { no: 3, name: "end", kind: "scalar", T: 5 /* ScalarType.INT32 */ },
        { no: 4, name: "confidence", kind: "scalar", T: 1 /* ScalarType.DOUBLE */ },
    ]);
    static fromBinary(bytes, options) {
        return new PiiFinding().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new PiiFinding().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new PiiFinding().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(PiiFinding, a, b);
    }
}
/**
 * Classify 応答
 *
 * @generated from message k1s0.tier1.pii.v1.ClassifyResponse
 */
export class ClassifyResponse extends Message {
    /**
     * 検出された PII 一覧（位置順）
     *
     * @generated from field: repeated k1s0.tier1.pii.v1.PiiFinding findings = 1;
     */
    findings = [];
    /**
     * PII を含むか（findings が空でなければ true）
     *
     * @generated from field: bool contains_pii = 2;
     */
    containsPii = false;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.pii.v1.ClassifyResponse";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "findings", kind: "message", T: PiiFinding, repeated: true },
        { no: 2, name: "contains_pii", kind: "scalar", T: 8 /* ScalarType.BOOL */ },
    ]);
    static fromBinary(bytes, options) {
        return new ClassifyResponse().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new ClassifyResponse().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new ClassifyResponse().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(ClassifyResponse, a, b);
    }
}
/**
 * Mask リクエスト
 *
 * @generated from message k1s0.tier1.pii.v1.MaskRequest
 */
export class MaskRequest extends Message {
    /**
     * マスキング対象テキスト
     *
     * @generated from field: string text = 1;
     */
    text = "";
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
    static typeName = "k1s0.tier1.pii.v1.MaskRequest";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "text", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 2, name: "context", kind: "message", T: TenantContext },
    ]);
    static fromBinary(bytes, options) {
        return new MaskRequest().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new MaskRequest().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new MaskRequest().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(MaskRequest, a, b);
    }
}
/**
 * Mask 応答
 *
 * @generated from message k1s0.tier1.pii.v1.MaskResponse
 */
export class MaskResponse extends Message {
    /**
     * マスク後のテキスト（氏名 → [NAME]、メール → [EMAIL] 等）
     *
     * @generated from field: string masked_text = 1;
     */
    maskedText = "";
    /**
     * 検出された PII 一覧（マスキング前の位置情報）
     *
     * @generated from field: repeated k1s0.tier1.pii.v1.PiiFinding findings = 2;
     */
    findings = [];
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.pii.v1.MaskResponse";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "masked_text", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 2, name: "findings", kind: "message", T: PiiFinding, repeated: true },
    ]);
    static fromBinary(bytes, options) {
        return new MaskResponse().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new MaskResponse().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new MaskResponse().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(MaskResponse, a, b);
    }
}
/**
 * Pseudonymize リクエスト（FR-T1-PII-002）。
 *
 * @generated from message k1s0.tier1.pii.v1.PseudonymizeRequest
 */
export class PseudonymizeRequest extends Message {
    /**
     * 仮名化対象の PII 種別（NAME / EMAIL / PHONE / MYNUMBER / ADDRESS / CREDITCARD / IPV4 等）。
     * 種別ごとに独立な仮名空間を持たせるため、HMAC 入力に prefix として混入する。
     *
     * @generated from field: string field_type = 1;
     */
    fieldType = "";
    /**
     * 仮名化対象の生値。
     *
     * @generated from field: string value = 2;
     */
    value = "";
    /**
     * 仮名空間を分離する salt。本番運用では OpenBao 等で管理し、
     * クライアントは salt 識別子のみを送る運用も許容する。空文字は不可。
     *
     * @generated from field: string salt = 3;
     */
    salt = "";
    /**
     * 呼出元コンテキスト（テナント境界の検証に必須）。
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 4;
     */
    context;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.pii.v1.PseudonymizeRequest";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "field_type", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 2, name: "value", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 3, name: "salt", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 4, name: "context", kind: "message", T: TenantContext },
    ]);
    static fromBinary(bytes, options) {
        return new PseudonymizeRequest().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new PseudonymizeRequest().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new PseudonymizeRequest().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(PseudonymizeRequest, a, b);
    }
}
/**
 * Pseudonymize 応答。
 *
 * @generated from message k1s0.tier1.pii.v1.PseudonymizeResponse
 */
export class PseudonymizeResponse extends Message {
    /**
     * 仮名化された値（URL-safe base64、padding 無し）。
     *
     * @generated from field: string pseudonym = 1;
     */
    pseudonym = "";
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.pii.v1.PseudonymizeResponse";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "pseudonym", kind: "scalar", T: 9 /* ScalarType.STRING */ },
    ]);
    static fromBinary(bytes, options) {
        return new PseudonymizeResponse().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new PseudonymizeResponse().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new PseudonymizeResponse().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(PseudonymizeResponse, a, b);
    }
}
//# sourceMappingURL=pii_service_pb.js.map