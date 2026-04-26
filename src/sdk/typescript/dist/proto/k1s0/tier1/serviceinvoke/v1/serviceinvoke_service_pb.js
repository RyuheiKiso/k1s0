// 本ファイルは tier1 公開 Service Invoke API の正式 proto。
// サービス間呼出を仲介する RPC を提供する（Dapr の app-to-app invoke 概念に対応）。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/01_Service_Invoke_API.md
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/01_Service_Invoke_API.md
//
// 関連要件: FR-T1-INVOKE-001〜005
//
// 注: 正典 IDL では package を `k1s0.tier1.invoke.v1` と記載しているが、
//     ディレクトリ設計（DS-DIR-* / IMP-DIR-*）と SDK 生成パスが
//     `serviceinvoke` で統一されているため、buf STANDARD lint の
//     PACKAGE_DIRECTORY_MATCH を満たすために本パッケージは
//     `k1s0.tier1.serviceinvoke.v1` とする。RPC / message / フィールドは
//     IDL 正典と完全一致させる。
// proto 構文宣言（proto3）
import { Message, proto3 } from "@bufbuild/protobuf";
import { TenantContext } from "../../common/v1/common_pb.js";
/**
 * Invoke リクエスト
 *
 * @generated from message k1s0.tier1.serviceinvoke.v1.InvokeRequest
 */
export class InvokeRequest extends Message {
    /**
     * 呼出先のアプリ識別子（Dapr app_id 互換、tier2 のサービス名に相当）
     *
     * @generated from field: string app_id = 1;
     */
    appId = "";
    /**
     * 呼出先のメソッド名（HTTP の場合は path に相当）
     *
     * @generated from field: string method = 2;
     */
    method = "";
    /**
     * 呼出データ（bytes で透過伝搬、encoding は content_type で示す）
     *
     * @generated from field: bytes data = 3;
     */
    data = new Uint8Array(0);
    /**
     * Content-Type（application/json / application/grpc / application/protobuf 等）
     *
     * @generated from field: string content_type = 4;
     */
    contentType = "";
    /**
     * 呼出元コンテキスト（テナント識別と相関 ID）
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 5;
     */
    context;
    /**
     * タイムアウト（ミリ秒、省略時は 5000ms）
     *
     * @generated from field: int32 timeout_ms = 6;
     */
    timeoutMs = 0;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.serviceinvoke.v1.InvokeRequest";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "app_id", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 2, name: "method", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 3, name: "data", kind: "scalar", T: 12 /* ScalarType.BYTES */ },
        { no: 4, name: "content_type", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 5, name: "context", kind: "message", T: TenantContext },
        { no: 6, name: "timeout_ms", kind: "scalar", T: 5 /* ScalarType.INT32 */ },
    ]);
    static fromBinary(bytes, options) {
        return new InvokeRequest().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new InvokeRequest().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new InvokeRequest().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(InvokeRequest, a, b);
    }
}
/**
 * Invoke 応答
 *
 * @generated from message k1s0.tier1.serviceinvoke.v1.InvokeResponse
 */
export class InvokeResponse extends Message {
    /**
     * 応答データ（bytes で透過伝搬、encoding は content_type で示す）
     *
     * @generated from field: bytes data = 1;
     */
    data = new Uint8Array(0);
    /**
     * Content-Type（呼出先が決定）
     *
     * @generated from field: string content_type = 2;
     */
    contentType = "";
    /**
     * HTTP ステータス相当（成功 200、失敗時は詳細を Status に載せる）
     *
     * @generated from field: int32 status = 3;
     */
    status = 0;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.serviceinvoke.v1.InvokeResponse";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "data", kind: "scalar", T: 12 /* ScalarType.BYTES */ },
        { no: 2, name: "content_type", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 3, name: "status", kind: "scalar", T: 5 /* ScalarType.INT32 */ },
    ]);
    static fromBinary(bytes, options) {
        return new InvokeResponse().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new InvokeResponse().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new InvokeResponse().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(InvokeResponse, a, b);
    }
}
/**
 * ストリーム応答のチャンク
 *
 * @generated from message k1s0.tier1.serviceinvoke.v1.InvokeChunk
 */
export class InvokeChunk extends Message {
    /**
     * チャンク本文
     *
     * @generated from field: bytes data = 1;
     */
    data = new Uint8Array(0);
    /**
     * ストリーム終端フラグ（true の場合は本チャンクが最終）
     *
     * @generated from field: bool eof = 2;
     */
    eof = false;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.serviceinvoke.v1.InvokeChunk";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "data", kind: "scalar", T: 12 /* ScalarType.BYTES */ },
        { no: 2, name: "eof", kind: "scalar", T: 8 /* ScalarType.BOOL */ },
    ]);
    static fromBinary(bytes, options) {
        return new InvokeChunk().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new InvokeChunk().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new InvokeChunk().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(InvokeChunk, a, b);
    }
}
//# sourceMappingURL=serviceinvoke_service_pb.js.map