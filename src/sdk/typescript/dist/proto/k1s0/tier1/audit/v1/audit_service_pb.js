// 本ファイルは tier1 公開 Audit API の正式 proto。
// 監査イベントの WORM ストア記録と検索を提供する。
// PII API（Classify / Mask）は別ファイル（pii/v1/pii_service.proto）に分離。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/10_Audit_Pii_API.md（AuditService 部）
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/10_Audit_Pii_API.md
//
// 関連要件: FR-T1-AUDIT-001〜003
//
// 注: 正典 IDL では AuditService と PiiService を 1 ファイル（package
//     k1s0.tier1.audit.v1）にまとめているが、ディレクトリ設計（DS-DIR-* /
//     IMP-DIR-*）と Pod 構成（t1-audit / t1-pii の 2 Pod 独立）に従い、
//     本リポジトリでは 2 ファイル / 2 パッケージに分割する。
//     RPC / message / フィールドは IDL と完全一致。
// proto 構文宣言（proto3）
import { Message, proto3, Timestamp } from "@bufbuild/protobuf";
import { TenantContext } from "../../common/v1/common_pb.js";
/**
 * 監査イベント
 *
 * @generated from message k1s0.tier1.audit.v1.AuditEvent
 */
export class AuditEvent extends Message {
    /**
     * 発生時刻（UTC）
     *
     * @generated from field: google.protobuf.Timestamp timestamp = 1;
     */
    timestamp;
    /**
     * 操作主体（user_id / workload_id）
     *
     * @generated from field: string actor = 2;
     */
    actor = "";
    /**
     * 操作種別（CREATE / READ / UPDATE / DELETE / LOGIN / EXPORT）
     *
     * @generated from field: string action = 3;
     */
    action = "";
    /**
     * 対象リソース（URN 形式: k1s0:tenant:<tid>:resource:<type>/<id>）
     *
     * @generated from field: string resource = 4;
     */
    resource = "";
    /**
     * 操作結果（SUCCESS / DENIED / ERROR）
     *
     * @generated from field: string outcome = 5;
     */
    outcome = "";
    /**
     * 追加コンテキスト（ip / user_agent / request_id 等）
     *
     * @generated from field: map<string, string> attributes = 6;
     */
    attributes = {};
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.audit.v1.AuditEvent";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "timestamp", kind: "message", T: Timestamp },
        { no: 2, name: "actor", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 3, name: "action", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 4, name: "resource", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 5, name: "outcome", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 6, name: "attributes", kind: "map", K: 9 /* ScalarType.STRING */, V: { kind: "scalar", T: 9 /* ScalarType.STRING */ } },
    ]);
    static fromBinary(bytes, options) {
        return new AuditEvent().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new AuditEvent().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new AuditEvent().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(AuditEvent, a, b);
    }
}
/**
 * Record リクエスト
 *
 * @generated from message k1s0.tier1.audit.v1.RecordAuditRequest
 */
export class RecordAuditRequest extends Message {
    /**
     * 記録対象イベント
     *
     * @generated from field: k1s0.tier1.audit.v1.AuditEvent event = 1;
     */
    event;
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
    static typeName = "k1s0.tier1.audit.v1.RecordAuditRequest";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "event", kind: "message", T: AuditEvent },
        { no: 2, name: "context", kind: "message", T: TenantContext },
    ]);
    static fromBinary(bytes, options) {
        return new RecordAuditRequest().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new RecordAuditRequest().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new RecordAuditRequest().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(RecordAuditRequest, a, b);
    }
}
/**
 * Record 応答
 *
 * @generated from message k1s0.tier1.audit.v1.RecordAuditResponse
 */
export class RecordAuditResponse extends Message {
    /**
     * WORM ストアでの固有 ID（再現性のあるハッシュ含む）
     *
     * @generated from field: string audit_id = 1;
     */
    auditId = "";
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.audit.v1.RecordAuditResponse";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "audit_id", kind: "scalar", T: 9 /* ScalarType.STRING */ },
    ]);
    static fromBinary(bytes, options) {
        return new RecordAuditResponse().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new RecordAuditResponse().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new RecordAuditResponse().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(RecordAuditResponse, a, b);
    }
}
/**
 * Query リクエスト
 *
 * @generated from message k1s0.tier1.audit.v1.QueryAuditRequest
 */
export class QueryAuditRequest extends Message {
    /**
     * 範囲開始
     *
     * @generated from field: google.protobuf.Timestamp from = 1;
     */
    from;
    /**
     * 範囲終了
     *
     * @generated from field: google.protobuf.Timestamp to = 2;
     */
    to;
    /**
     * フィルタ（任意の attributes 等価一致、AND 結合）
     *
     * @generated from field: map<string, string> filters = 3;
     */
    filters = {};
    /**
     * 件数上限（既定 100、最大 1000）
     *
     * @generated from field: int32 limit = 4;
     */
    limit = 0;
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
    static typeName = "k1s0.tier1.audit.v1.QueryAuditRequest";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "from", kind: "message", T: Timestamp },
        { no: 2, name: "to", kind: "message", T: Timestamp },
        { no: 3, name: "filters", kind: "map", K: 9 /* ScalarType.STRING */, V: { kind: "scalar", T: 9 /* ScalarType.STRING */ } },
        { no: 4, name: "limit", kind: "scalar", T: 5 /* ScalarType.INT32 */ },
        { no: 5, name: "context", kind: "message", T: TenantContext },
    ]);
    static fromBinary(bytes, options) {
        return new QueryAuditRequest().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new QueryAuditRequest().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new QueryAuditRequest().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(QueryAuditRequest, a, b);
    }
}
/**
 * Query 応答
 *
 * @generated from message k1s0.tier1.audit.v1.QueryAuditResponse
 */
export class QueryAuditResponse extends Message {
    /**
     * 検索結果（時刻昇順、出力時に PII Mask 自動適用）
     *
     * @generated from field: repeated k1s0.tier1.audit.v1.AuditEvent events = 1;
     */
    events = [];
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.audit.v1.QueryAuditResponse";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "events", kind: "message", T: AuditEvent, repeated: true },
    ]);
    static fromBinary(bytes, options) {
        return new QueryAuditResponse().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new QueryAuditResponse().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new QueryAuditResponse().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(QueryAuditResponse, a, b);
    }
}
//# sourceMappingURL=audit_service_pb.js.map