import type { BinaryReadOptions, FieldList, JsonReadOptions, JsonValue, PartialMessage, PlainMessage } from "@bufbuild/protobuf";
import { Message, proto3 } from "@bufbuild/protobuf";
import { TenantContext } from "../../common/v1/common_pb.js";
/**
 * Invoke リクエスト
 *
 * @generated from message k1s0.tier1.binding.v1.InvokeBindingRequest
 */
export declare class InvokeBindingRequest extends Message<InvokeBindingRequest> {
    /**
     * バインディング名（運用側で事前設定、例: s3-archive / smtp-notify）
     *
     * @generated from field: string name = 1;
     */
    name: string;
    /**
     * 操作種別（create / get / list / delete / send 等、バインディング型依存）
     *
     * @generated from field: string operation = 2;
     */
    operation: string;
    /**
     * 操作データ本文
     *
     * @generated from field: bytes data = 3;
     */
    data: Uint8Array;
    /**
     * メタデータ（content-type / to / subject 等、バインディング型依存）
     *
     * @generated from field: map<string, string> metadata = 4;
     */
    metadata: {
        [key: string]: string;
    };
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 5;
     */
    context?: TenantContext;
    /**
     * 冪等性キー（共通規約 §「冪等性と再試行」: 24h TTL の dedup）
     * 外部送信（SMTP / S3 等）の重複防止に必須。同一キーでの再試行は初回 response を返す。
     *
     * @generated from field: string idempotency_key = 6;
     */
    idempotencyKey: string;
    constructor(data?: PartialMessage<InvokeBindingRequest>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.binding.v1.InvokeBindingRequest";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): InvokeBindingRequest;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): InvokeBindingRequest;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): InvokeBindingRequest;
    static equals(a: InvokeBindingRequest | PlainMessage<InvokeBindingRequest> | undefined, b: InvokeBindingRequest | PlainMessage<InvokeBindingRequest> | undefined): boolean;
}
/**
 * Invoke 応答
 *
 * @generated from message k1s0.tier1.binding.v1.InvokeBindingResponse
 */
export declare class InvokeBindingResponse extends Message<InvokeBindingResponse> {
    /**
     * 応答本文（操作種別とバインディング型に依存）
     *
     * @generated from field: bytes data = 1;
     */
    data: Uint8Array;
    /**
     * メタデータ（外部システムから返るヘッダ等）
     *
     * @generated from field: map<string, string> metadata = 2;
     */
    metadata: {
        [key: string]: string;
    };
    constructor(data?: PartialMessage<InvokeBindingResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.binding.v1.InvokeBindingResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): InvokeBindingResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): InvokeBindingResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): InvokeBindingResponse;
    static equals(a: InvokeBindingResponse | PlainMessage<InvokeBindingResponse> | undefined, b: InvokeBindingResponse | PlainMessage<InvokeBindingResponse> | undefined): boolean;
}
//# sourceMappingURL=binding_service_pb.d.ts.map