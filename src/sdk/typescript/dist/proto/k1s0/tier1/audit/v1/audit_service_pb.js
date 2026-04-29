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
import { Message, proto3, protoInt64, Timestamp } from "@bufbuild/protobuf";
import { TenantContext } from "../../common/v1/common_pb.js";
/**
 * Export のフォーマット種別。
 *
 * @generated from enum k1s0.tier1.audit.v1.ExportFormat
 */
export var ExportFormat;
(function (ExportFormat) {
    /**
     * 既定（指定なし）。サーバが NDJSON にフォールバックする。
     *
     * @generated from enum value: EXPORT_FORMAT_UNSPECIFIED = 0;
     */
    ExportFormat[ExportFormat["UNSPECIFIED"] = 0] = "UNSPECIFIED";
    /**
     * CSV（RFC 4180、ヘッダ行を最初の chunk に出力）。
     *
     * @generated from enum value: EXPORT_FORMAT_CSV = 1;
     */
    ExportFormat[ExportFormat["CSV"] = 1] = "CSV";
    /**
     * 改行区切り JSON（1 行 = 1 event）。Splunk / fluentd 取り込み向け。
     *
     * @generated from enum value: EXPORT_FORMAT_NDJSON = 2;
     */
    ExportFormat[ExportFormat["NDJSON"] = 2] = "NDJSON";
    /**
     * 単一 JSON 配列（小規模向け、最後の chunk で `]` を閉じる）。
     *
     * @generated from enum value: EXPORT_FORMAT_JSON_ARRAY = 3;
     */
    ExportFormat[ExportFormat["JSON_ARRAY"] = 3] = "JSON_ARRAY";
})(ExportFormat || (ExportFormat = {}));
// Retrieve enum metadata with: proto3.getEnumType(ExportFormat)
proto3.util.setEnumType(ExportFormat, "k1s0.tier1.audit.v1.ExportFormat", [
    { no: 0, name: "EXPORT_FORMAT_UNSPECIFIED" },
    { no: 1, name: "EXPORT_FORMAT_CSV" },
    { no: 2, name: "EXPORT_FORMAT_NDJSON" },
    { no: 3, name: "EXPORT_FORMAT_JSON_ARRAY" },
]);
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
    /**
     * 冪等性キー（共通規約 §「冪等性と再試行」: 24h TTL の dedup）
     * 重複 audit event 書込を防ぐ（hash chain 整合性が乱れないよう）。
     * 同一キーでの再試行は副作用を重複させず初回 audit_id を返す。
     *
     * @generated from field: string idempotency_key = 3;
     */
    idempotencyKey = "";
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.audit.v1.RecordAuditRequest";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "event", kind: "message", T: AuditEvent },
        { no: 2, name: "context", kind: "message", T: TenantContext },
        { no: 3, name: "idempotency_key", kind: "scalar", T: 9 /* ScalarType.STRING */ },
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
/**
 * VerifyChain リクエスト（FR-T1-AUDIT-002）
 *
 * @generated from message k1s0.tier1.audit.v1.VerifyChainRequest
 */
export class VerifyChainRequest extends Message {
    /**
     * 範囲開始（任意）。未指定（zero）はテナント配下の全履歴を対象。
     *
     * @generated from field: google.protobuf.Timestamp from = 1;
     */
    from;
    /**
     * 範囲終了（任意）。未指定（zero）は最新まで。
     *
     * @generated from field: google.protobuf.Timestamp to = 2;
     */
    to;
    /**
     * 呼出元コンテキスト（テナント境界の検証に必須）
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 3;
     */
    context;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.audit.v1.VerifyChainRequest";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "from", kind: "message", T: Timestamp },
        { no: 2, name: "to", kind: "message", T: Timestamp },
        { no: 3, name: "context", kind: "message", T: TenantContext },
    ]);
    static fromBinary(bytes, options) {
        return new VerifyChainRequest().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new VerifyChainRequest().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new VerifyChainRequest().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(VerifyChainRequest, a, b);
    }
}
/**
 * VerifyChain 応答
 *
 * @generated from message k1s0.tier1.audit.v1.VerifyChainResponse
 */
export class VerifyChainResponse extends Message {
    /**
     * チェーン全体の整合性が取れていれば true。
     *
     * @generated from field: bool valid = 1;
     */
    valid = false;
    /**
     * 検証対象だったイベント件数（valid に関わらず set される）。
     *
     * @generated from field: int64 checked_count = 2;
     */
    checkedCount = protoInt64.zero;
    /**
     * valid=false 時のみ意味あり。最初に不整合を検出した sequence_number。
     * valid=true 時は 0。
     *
     * @generated from field: int64 first_bad_sequence = 3;
     */
    firstBadSequence = protoInt64.zero;
    /**
     * 不整合の理由（"prev_hash mismatch" / "event_hash mismatch" / "tenant boundary" 等）。
     * valid=true 時は空文字。
     *
     * @generated from field: string reason = 4;
     */
    reason = "";
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.audit.v1.VerifyChainResponse";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "valid", kind: "scalar", T: 8 /* ScalarType.BOOL */ },
        { no: 2, name: "checked_count", kind: "scalar", T: 3 /* ScalarType.INT64 */ },
        { no: 3, name: "first_bad_sequence", kind: "scalar", T: 3 /* ScalarType.INT64 */ },
        { no: 4, name: "reason", kind: "scalar", T: 9 /* ScalarType.STRING */ },
    ]);
    static fromBinary(bytes, options) {
        return new VerifyChainResponse().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new VerifyChainResponse().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new VerifyChainResponse().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(VerifyChainResponse, a, b);
    }
}
/**
 * Export リクエスト（FR-T1-AUDIT-002 疑似 IF "Audit.Export"）
 *
 * @generated from message k1s0.tier1.audit.v1.ExportAuditRequest
 */
export class ExportAuditRequest extends Message {
    /**
     * 範囲開始（任意）。未指定（zero）は全履歴の先頭。
     *
     * @generated from field: google.protobuf.Timestamp from = 1;
     */
    from;
    /**
     * 範囲終了（任意）。未指定（zero）は最新まで。
     *
     * @generated from field: google.protobuf.Timestamp to = 2;
     */
    to;
    /**
     * 出力フォーマット。EXPORT_FORMAT_UNSPECIFIED は NDJSON 扱い。
     *
     * @generated from field: k1s0.tier1.audit.v1.ExportFormat format = 3;
     */
    format = ExportFormat.UNSPECIFIED;
    /**
     * 1 chunk あたりの最大バイト数（既定 65536、上限 1048576）。
     *
     * @generated from field: int32 chunk_bytes = 4;
     */
    chunkBytes = 0;
    /**
     * 呼出元コンテキスト（テナント境界の検証に必須）。
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 5;
     */
    context;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.audit.v1.ExportAuditRequest";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "from", kind: "message", T: Timestamp },
        { no: 2, name: "to", kind: "message", T: Timestamp },
        { no: 3, name: "format", kind: "enum", T: proto3.getEnumType(ExportFormat) },
        { no: 4, name: "chunk_bytes", kind: "scalar", T: 5 /* ScalarType.INT32 */ },
        { no: 5, name: "context", kind: "message", T: TenantContext },
    ]);
    static fromBinary(bytes, options) {
        return new ExportAuditRequest().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new ExportAuditRequest().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new ExportAuditRequest().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(ExportAuditRequest, a, b);
    }
}
/**
 * Export 応答（server-streaming）の 1 チャンク
 *
 * @generated from message k1s0.tier1.audit.v1.ExportAuditChunk
 */
export class ExportAuditChunk extends Message {
    /**
     * フォーマット済みデータの 1 部分（バイナリ透過、UTF-8 を想定）。
     *
     * @generated from field: bytes data = 1;
     */
    data = new Uint8Array(0);
    /**
     * 0 起点のチャンク連番（再構成 / 監査時の参照用）。
     *
     * @generated from field: int64 sequence = 2;
     */
    sequence = protoInt64.zero;
    /**
     * この chunk に含まれる event 数（chunk_bytes ベースの場合は variable）。
     *
     * @generated from field: int64 event_count = 3;
     */
    eventCount = protoInt64.zero;
    /**
     * ストリーム末尾の chunk なら true。最後の "]" や EOF newline を含む。
     *
     * @generated from field: bool is_last = 4;
     */
    isLast = false;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.audit.v1.ExportAuditChunk";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "data", kind: "scalar", T: 12 /* ScalarType.BYTES */ },
        { no: 2, name: "sequence", kind: "scalar", T: 3 /* ScalarType.INT64 */ },
        { no: 3, name: "event_count", kind: "scalar", T: 3 /* ScalarType.INT64 */ },
        { no: 4, name: "is_last", kind: "scalar", T: 8 /* ScalarType.BOOL */ },
    ]);
    static fromBinary(bytes, options) {
        return new ExportAuditChunk().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new ExportAuditChunk().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new ExportAuditChunk().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(ExportAuditChunk, a, b);
    }
}
//# sourceMappingURL=audit_service_pb.js.map