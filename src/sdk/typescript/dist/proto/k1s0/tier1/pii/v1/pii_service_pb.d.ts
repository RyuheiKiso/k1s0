import type { BinaryReadOptions, FieldList, JsonReadOptions, JsonValue, PartialMessage, PlainMessage } from "@bufbuild/protobuf";
import { Message, proto3 } from "@bufbuild/protobuf";
import { TenantContext } from "../../common/v1/common_pb.js";
/**
 * Classify リクエスト
 *
 * @generated from message k1s0.tier1.pii.v1.ClassifyRequest
 */
export declare class ClassifyRequest extends Message<ClassifyRequest> {
    /**
     * 判定対象テキスト
     *
     * @generated from field: string text = 1;
     */
    text: string;
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 2;
     */
    context?: TenantContext;
    constructor(data?: PartialMessage<ClassifyRequest>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.pii.v1.ClassifyRequest";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): ClassifyRequest;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): ClassifyRequest;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): ClassifyRequest;
    static equals(a: ClassifyRequest | PlainMessage<ClassifyRequest> | undefined, b: ClassifyRequest | PlainMessage<ClassifyRequest> | undefined): boolean;
}
/**
 * PII 検出結果の 1 件
 *
 * @generated from message k1s0.tier1.pii.v1.PiiFinding
 */
export declare class PiiFinding extends Message<PiiFinding> {
    /**
     * 検出された PII 種別（NAME / EMAIL / PHONE / MYNUMBER / CREDITCARD 等）
     *
     * @generated from field: string type = 1;
     */
    type: string;
    /**
     * 文字列内の開始位置（0 始まり、UTF-8 byte 単位ではなく文字単位）
     *
     * @generated from field: int32 start = 2;
     */
    start: number;
    /**
     * 文字列内の終了位置（exclusive）
     *
     * @generated from field: int32 end = 3;
     */
    end: number;
    /**
     * 信頼度（0.0〜1.0）
     *
     * @generated from field: double confidence = 4;
     */
    confidence: number;
    constructor(data?: PartialMessage<PiiFinding>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.pii.v1.PiiFinding";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): PiiFinding;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): PiiFinding;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): PiiFinding;
    static equals(a: PiiFinding | PlainMessage<PiiFinding> | undefined, b: PiiFinding | PlainMessage<PiiFinding> | undefined): boolean;
}
/**
 * Classify 応答
 *
 * @generated from message k1s0.tier1.pii.v1.ClassifyResponse
 */
export declare class ClassifyResponse extends Message<ClassifyResponse> {
    /**
     * 検出された PII 一覧（位置順）
     *
     * @generated from field: repeated k1s0.tier1.pii.v1.PiiFinding findings = 1;
     */
    findings: PiiFinding[];
    /**
     * PII を含むか（findings が空でなければ true）
     *
     * @generated from field: bool contains_pii = 2;
     */
    containsPii: boolean;
    constructor(data?: PartialMessage<ClassifyResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.pii.v1.ClassifyResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): ClassifyResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): ClassifyResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): ClassifyResponse;
    static equals(a: ClassifyResponse | PlainMessage<ClassifyResponse> | undefined, b: ClassifyResponse | PlainMessage<ClassifyResponse> | undefined): boolean;
}
/**
 * Mask リクエスト
 *
 * @generated from message k1s0.tier1.pii.v1.MaskRequest
 */
export declare class MaskRequest extends Message<MaskRequest> {
    /**
     * マスキング対象テキスト
     *
     * @generated from field: string text = 1;
     */
    text: string;
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 2;
     */
    context?: TenantContext;
    constructor(data?: PartialMessage<MaskRequest>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.pii.v1.MaskRequest";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): MaskRequest;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): MaskRequest;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): MaskRequest;
    static equals(a: MaskRequest | PlainMessage<MaskRequest> | undefined, b: MaskRequest | PlainMessage<MaskRequest> | undefined): boolean;
}
/**
 * Mask 応答
 *
 * @generated from message k1s0.tier1.pii.v1.MaskResponse
 */
export declare class MaskResponse extends Message<MaskResponse> {
    /**
     * マスク後のテキスト（氏名 → [NAME]、メール → [EMAIL] 等）
     *
     * @generated from field: string masked_text = 1;
     */
    maskedText: string;
    /**
     * 検出された PII 一覧（マスキング前の位置情報）
     *
     * @generated from field: repeated k1s0.tier1.pii.v1.PiiFinding findings = 2;
     */
    findings: PiiFinding[];
    constructor(data?: PartialMessage<MaskResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.pii.v1.MaskResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): MaskResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): MaskResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): MaskResponse;
    static equals(a: MaskResponse | PlainMessage<MaskResponse> | undefined, b: MaskResponse | PlainMessage<MaskResponse> | undefined): boolean;
}
//# sourceMappingURL=pii_service_pb.d.ts.map