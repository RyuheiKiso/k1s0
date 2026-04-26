// 本ファイルは tier1 公開 State API の正式 proto。
// KV / Relational / Document 状態管理（楽観的排他とトランザクション境界付き）を提供する。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/02_State_API.md
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/02_State_API.md
//
// 関連要件: FR-T1-STATE-001〜005
// proto 構文宣言（proto3）
import { Message, proto3 } from "@bufbuild/protobuf";
import { TenantContext } from "../../common/v1/common_pb.js";
/**
 * Get リクエスト
 *
 * @generated from message k1s0.tier1.state.v1.GetRequest
 */
export class GetRequest extends Message {
    /**
     * Store 名（valkey-default / postgres-tenant 等、運用側で設定）
     *
     * @generated from field: string store = 1;
     */
    store = "";
    /**
     * キー（テナント境界は tier1 が自動付与、クライアントはテナント内キーのみ指定）
     *
     * @generated from field: string key = 2;
     */
    key = "";
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
    static typeName = "k1s0.tier1.state.v1.GetRequest";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "store", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 2, name: "key", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 3, name: "context", kind: "message", T: TenantContext },
    ]);
    static fromBinary(bytes, options) {
        return new GetRequest().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new GetRequest().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new GetRequest().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(GetRequest, a, b);
    }
}
/**
 * Get 応答
 *
 * @generated from message k1s0.tier1.state.v1.GetResponse
 */
export class GetResponse extends Message {
    /**
     * 値本文（bytes で透過、encoding はクライアント責務）
     *
     * @generated from field: bytes data = 1;
     */
    data = new Uint8Array(0);
    /**
     * 楽観的排他のための ETag（Set / Delete 時に expected_etag に再送する）
     *
     * @generated from field: string etag = 2;
     */
    etag = "";
    /**
     * キー未存在時は true（このとき data / etag は空、エラーではない）
     *
     * @generated from field: bool not_found = 3;
     */
    notFound = false;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.state.v1.GetResponse";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "data", kind: "scalar", T: 12 /* ScalarType.BYTES */ },
        { no: 2, name: "etag", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 3, name: "not_found", kind: "scalar", T: 8 /* ScalarType.BOOL */ },
    ]);
    static fromBinary(bytes, options) {
        return new GetResponse().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new GetResponse().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new GetResponse().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(GetResponse, a, b);
    }
}
/**
 * Set リクエスト
 *
 * @generated from message k1s0.tier1.state.v1.SetRequest
 */
export class SetRequest extends Message {
    /**
     * Store 名
     *
     * @generated from field: string store = 1;
     */
    store = "";
    /**
     * キー
     *
     * @generated from field: string key = 2;
     */
    key = "";
    /**
     * 保存値本文
     *
     * @generated from field: bytes data = 3;
     */
    data = new Uint8Array(0);
    /**
     * 期待 ETag（空は未存在前提、新規作成時は空文字列）
     *
     * @generated from field: string expected_etag = 4;
     */
    expectedEtag = "";
    /**
     * TTL（秒、0 は永続）
     *
     * @generated from field: int32 ttl_sec = 5;
     */
    ttlSec = 0;
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 6;
     */
    context;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.state.v1.SetRequest";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "store", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 2, name: "key", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 3, name: "data", kind: "scalar", T: 12 /* ScalarType.BYTES */ },
        { no: 4, name: "expected_etag", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 5, name: "ttl_sec", kind: "scalar", T: 5 /* ScalarType.INT32 */ },
        { no: 6, name: "context", kind: "message", T: TenantContext },
    ]);
    static fromBinary(bytes, options) {
        return new SetRequest().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new SetRequest().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new SetRequest().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(SetRequest, a, b);
    }
}
/**
 * Set 応答
 *
 * @generated from message k1s0.tier1.state.v1.SetResponse
 */
export class SetResponse extends Message {
    /**
     * 保存後の新 ETag（次回 Set / Delete 時の expected_etag に渡す）
     *
     * @generated from field: string new_etag = 1;
     */
    newEtag = "";
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.state.v1.SetResponse";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "new_etag", kind: "scalar", T: 9 /* ScalarType.STRING */ },
    ]);
    static fromBinary(bytes, options) {
        return new SetResponse().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new SetResponse().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new SetResponse().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(SetResponse, a, b);
    }
}
/**
 * Delete リクエスト
 *
 * @generated from message k1s0.tier1.state.v1.DeleteRequest
 */
export class DeleteRequest extends Message {
    /**
     * Store 名
     *
     * @generated from field: string store = 1;
     */
    store = "";
    /**
     * キー
     *
     * @generated from field: string key = 2;
     */
    key = "";
    /**
     * 期待 ETag（空は無条件削除、指定時は楽観的排他で削除）
     *
     * @generated from field: string expected_etag = 3;
     */
    expectedEtag = "";
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
    static typeName = "k1s0.tier1.state.v1.DeleteRequest";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "store", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 2, name: "key", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 3, name: "expected_etag", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 4, name: "context", kind: "message", T: TenantContext },
    ]);
    static fromBinary(bytes, options) {
        return new DeleteRequest().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new DeleteRequest().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new DeleteRequest().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(DeleteRequest, a, b);
    }
}
/**
 * Delete 応答
 *
 * @generated from message k1s0.tier1.state.v1.DeleteResponse
 */
export class DeleteResponse extends Message {
    /**
     * 削除実行可否（未存在キーへの削除も deleted=true で返す）
     *
     * @generated from field: bool deleted = 1;
     */
    deleted = false;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.state.v1.DeleteResponse";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "deleted", kind: "scalar", T: 8 /* ScalarType.BOOL */ },
    ]);
    static fromBinary(bytes, options) {
        return new DeleteResponse().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new DeleteResponse().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new DeleteResponse().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(DeleteResponse, a, b);
    }
}
/**
 * BulkGet リクエスト
 *
 * @generated from message k1s0.tier1.state.v1.BulkGetRequest
 */
export class BulkGetRequest extends Message {
    /**
     * Store 名（全キーで共通）
     *
     * @generated from field: string store = 1;
     */
    store = "";
    /**
     * 取得するキー一覧
     *
     * @generated from field: repeated string keys = 2;
     */
    keys = [];
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
    static typeName = "k1s0.tier1.state.v1.BulkGetRequest";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "store", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 2, name: "keys", kind: "scalar", T: 9 /* ScalarType.STRING */, repeated: true },
        { no: 3, name: "context", kind: "message", T: TenantContext },
    ]);
    static fromBinary(bytes, options) {
        return new BulkGetRequest().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new BulkGetRequest().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new BulkGetRequest().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(BulkGetRequest, a, b);
    }
}
/**
 * BulkGet 応答
 *
 * @generated from message k1s0.tier1.state.v1.BulkGetResponse
 */
export class BulkGetResponse extends Message {
    /**
     * 結果マップ（キー → GetResponse、未存在キーも not_found=true で含める）
     *
     * @generated from field: map<string, k1s0.tier1.state.v1.GetResponse> results = 1;
     */
    results = {};
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.state.v1.BulkGetResponse";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "results", kind: "map", K: 9 /* ScalarType.STRING */, V: { kind: "message", T: GetResponse } },
    ]);
    static fromBinary(bytes, options) {
        return new BulkGetResponse().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new BulkGetResponse().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new BulkGetResponse().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(BulkGetResponse, a, b);
    }
}
/**
 * トランザクション内の 1 操作（Set / Delete のいずれか）
 *
 * @generated from message k1s0.tier1.state.v1.TransactOp
 */
export class TransactOp extends Message {
    /**
     * 操作種別（oneof で Set または Delete を排他選択）
     *
     * @generated from oneof k1s0.tier1.state.v1.TransactOp.op
     */
    op = { case: undefined };
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.state.v1.TransactOp";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "set", kind: "message", T: SetRequest, oneof: "op" },
        { no: 2, name: "delete", kind: "message", T: DeleteRequest, oneof: "op" },
    ]);
    static fromBinary(bytes, options) {
        return new TransactOp().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new TransactOp().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new TransactOp().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(TransactOp, a, b);
    }
}
/**
 * Transact リクエスト
 *
 * @generated from message k1s0.tier1.state.v1.TransactRequest
 */
export class TransactRequest extends Message {
    /**
     * Store 名（全操作で共通、複数 Store を跨ぐトランザクションは不可）
     *
     * @generated from field: string store = 1;
     */
    store = "";
    /**
     * 操作列（記述順に実行、途中失敗で全ロールバック）
     *
     * @generated from field: repeated k1s0.tier1.state.v1.TransactOp operations = 2;
     */
    operations = [];
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
    static typeName = "k1s0.tier1.state.v1.TransactRequest";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "store", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 2, name: "operations", kind: "message", T: TransactOp, repeated: true },
        { no: 3, name: "context", kind: "message", T: TenantContext },
    ]);
    static fromBinary(bytes, options) {
        return new TransactRequest().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new TransactRequest().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new TransactRequest().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(TransactRequest, a, b);
    }
}
/**
 * Transact 応答
 *
 * @generated from message k1s0.tier1.state.v1.TransactResponse
 */
export class TransactResponse extends Message {
    /**
     * コミット成功可否（false の場合は全ロールバック済み）
     *
     * @generated from field: bool committed = 1;
     */
    committed = false;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.state.v1.TransactResponse";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "committed", kind: "scalar", T: 8 /* ScalarType.BOOL */ },
    ]);
    static fromBinary(bytes, options) {
        return new TransactResponse().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new TransactResponse().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new TransactResponse().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(TransactResponse, a, b);
    }
}
//# sourceMappingURL=state_service_pb.js.map