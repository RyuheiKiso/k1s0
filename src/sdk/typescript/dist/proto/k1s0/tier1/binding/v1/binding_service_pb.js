// 本ファイルは tier1 公開 Binding API の正式 proto。
// 外部システム（HTTP / SMTP / S3 等）への出力バインディング呼出を提供する。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/05_Binding_API.md
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/05_Binding_API.md
//
// 関連要件: FR-T1-BINDING-001〜004
// proto 構文宣言（proto3）
import { Message, proto3 } from "@bufbuild/protobuf";
import { TenantContext } from "../../common/v1/common_pb.js";
/**
 * Invoke リクエスト
 *
 * @generated from message k1s0.tier1.binding.v1.InvokeBindingRequest
 */
export class InvokeBindingRequest extends Message {
    /**
     * バインディング名（運用側で事前設定、例: s3-archive / smtp-notify）
     *
     * @generated from field: string name = 1;
     */
    name = "";
    /**
     * 操作種別（create / get / list / delete / send 等、バインディング型依存）
     *
     * @generated from field: string operation = 2;
     */
    operation = "";
    /**
     * 操作データ本文
     *
     * @generated from field: bytes data = 3;
     */
    data = new Uint8Array(0);
    /**
     * メタデータ（content-type / to / subject 等、バインディング型依存）
     *
     * @generated from field: map<string, string> metadata = 4;
     */
    metadata = {};
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 5;
     */
    context;
    /**
     * 冪等性キー（共通規約 §「冪等性と再試行」: 24h TTL の dedup）
     * 外部送信（SMTP / S3 等）の重複防止に必須。同一キーでの再試行は初回 response を返す。
     *
     * @generated from field: string idempotency_key = 6;
     */
    idempotencyKey = "";
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.binding.v1.InvokeBindingRequest";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "name", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 2, name: "operation", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 3, name: "data", kind: "scalar", T: 12 /* ScalarType.BYTES */ },
        { no: 4, name: "metadata", kind: "map", K: 9 /* ScalarType.STRING */, V: { kind: "scalar", T: 9 /* ScalarType.STRING */ } },
        { no: 5, name: "context", kind: "message", T: TenantContext },
        { no: 6, name: "idempotency_key", kind: "scalar", T: 9 /* ScalarType.STRING */ },
    ]);
    static fromBinary(bytes, options) {
        return new InvokeBindingRequest().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new InvokeBindingRequest().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new InvokeBindingRequest().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(InvokeBindingRequest, a, b);
    }
}
/**
 * Invoke 応答
 *
 * @generated from message k1s0.tier1.binding.v1.InvokeBindingResponse
 */
export class InvokeBindingResponse extends Message {
    /**
     * 応答本文（操作種別とバインディング型に依存）
     *
     * @generated from field: bytes data = 1;
     */
    data = new Uint8Array(0);
    /**
     * メタデータ（外部システムから返るヘッダ等）
     *
     * @generated from field: map<string, string> metadata = 2;
     */
    metadata = {};
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.binding.v1.InvokeBindingResponse";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "data", kind: "scalar", T: 12 /* ScalarType.BYTES */ },
        { no: 2, name: "metadata", kind: "map", K: 9 /* ScalarType.STRING */, V: { kind: "scalar", T: 9 /* ScalarType.STRING */ } },
    ]);
    static fromBinary(bytes, options) {
        return new InvokeBindingResponse().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new InvokeBindingResponse().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new InvokeBindingResponse().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(InvokeBindingResponse, a, b);
    }
}
//# sourceMappingURL=binding_service_pb.js.map