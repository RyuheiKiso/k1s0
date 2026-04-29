import type { BinaryReadOptions, FieldList, JsonReadOptions, JsonValue, PartialMessage, PlainMessage } from "@bufbuild/protobuf";
import { Message, proto3, Timestamp } from "@bufbuild/protobuf";
import { TenantContext } from "../../common/v1/common_pb.js";
/**
 * Export のフォーマット種別。
 *
 * @generated from enum k1s0.tier1.audit.v1.ExportFormat
 */
export declare enum ExportFormat {
    /**
     * 既定（指定なし）。サーバが NDJSON にフォールバックする。
     *
     * @generated from enum value: EXPORT_FORMAT_UNSPECIFIED = 0;
     */
    UNSPECIFIED = 0,
    /**
     * CSV（RFC 4180、ヘッダ行を最初の chunk に出力）。
     *
     * @generated from enum value: EXPORT_FORMAT_CSV = 1;
     */
    CSV = 1,
    /**
     * 改行区切り JSON（1 行 = 1 event）。Splunk / fluentd 取り込み向け。
     *
     * @generated from enum value: EXPORT_FORMAT_NDJSON = 2;
     */
    NDJSON = 2,
    /**
     * 単一 JSON 配列（小規模向け、最後の chunk で `]` を閉じる）。
     *
     * @generated from enum value: EXPORT_FORMAT_JSON_ARRAY = 3;
     */
    JSON_ARRAY = 3
}
/**
 * 監査イベント
 *
 * @generated from message k1s0.tier1.audit.v1.AuditEvent
 */
export declare class AuditEvent extends Message<AuditEvent> {
    /**
     * 発生時刻（UTC）
     *
     * @generated from field: google.protobuf.Timestamp timestamp = 1;
     */
    timestamp?: Timestamp;
    /**
     * 操作主体（user_id / workload_id）
     *
     * @generated from field: string actor = 2;
     */
    actor: string;
    /**
     * 操作種別（CREATE / READ / UPDATE / DELETE / LOGIN / EXPORT）
     *
     * @generated from field: string action = 3;
     */
    action: string;
    /**
     * 対象リソース（URN 形式: k1s0:tenant:<tid>:resource:<type>/<id>）
     *
     * @generated from field: string resource = 4;
     */
    resource: string;
    /**
     * 操作結果（SUCCESS / DENIED / ERROR）
     *
     * @generated from field: string outcome = 5;
     */
    outcome: string;
    /**
     * 追加コンテキスト（ip / user_agent / request_id 等）
     *
     * @generated from field: map<string, string> attributes = 6;
     */
    attributes: {
        [key: string]: string;
    };
    constructor(data?: PartialMessage<AuditEvent>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.audit.v1.AuditEvent";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): AuditEvent;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): AuditEvent;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): AuditEvent;
    static equals(a: AuditEvent | PlainMessage<AuditEvent> | undefined, b: AuditEvent | PlainMessage<AuditEvent> | undefined): boolean;
}
/**
 * Record リクエスト
 *
 * @generated from message k1s0.tier1.audit.v1.RecordAuditRequest
 */
export declare class RecordAuditRequest extends Message<RecordAuditRequest> {
    /**
     * 記録対象イベント
     *
     * @generated from field: k1s0.tier1.audit.v1.AuditEvent event = 1;
     */
    event?: AuditEvent;
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 2;
     */
    context?: TenantContext;
    /**
     * 冪等性キー（共通規約 §「冪等性と再試行」: 24h TTL の dedup）
     * 重複 audit event 書込を防ぐ（hash chain 整合性が乱れないよう）。
     * 同一キーでの再試行は副作用を重複させず初回 audit_id を返す。
     *
     * @generated from field: string idempotency_key = 3;
     */
    idempotencyKey: string;
    constructor(data?: PartialMessage<RecordAuditRequest>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.audit.v1.RecordAuditRequest";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): RecordAuditRequest;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): RecordAuditRequest;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): RecordAuditRequest;
    static equals(a: RecordAuditRequest | PlainMessage<RecordAuditRequest> | undefined, b: RecordAuditRequest | PlainMessage<RecordAuditRequest> | undefined): boolean;
}
/**
 * Record 応答
 *
 * @generated from message k1s0.tier1.audit.v1.RecordAuditResponse
 */
export declare class RecordAuditResponse extends Message<RecordAuditResponse> {
    /**
     * WORM ストアでの固有 ID（再現性のあるハッシュ含む）
     *
     * @generated from field: string audit_id = 1;
     */
    auditId: string;
    constructor(data?: PartialMessage<RecordAuditResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.audit.v1.RecordAuditResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): RecordAuditResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): RecordAuditResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): RecordAuditResponse;
    static equals(a: RecordAuditResponse | PlainMessage<RecordAuditResponse> | undefined, b: RecordAuditResponse | PlainMessage<RecordAuditResponse> | undefined): boolean;
}
/**
 * Query リクエスト
 *
 * @generated from message k1s0.tier1.audit.v1.QueryAuditRequest
 */
export declare class QueryAuditRequest extends Message<QueryAuditRequest> {
    /**
     * 範囲開始
     *
     * @generated from field: google.protobuf.Timestamp from = 1;
     */
    from?: Timestamp;
    /**
     * 範囲終了
     *
     * @generated from field: google.protobuf.Timestamp to = 2;
     */
    to?: Timestamp;
    /**
     * フィルタ（任意の attributes 等価一致、AND 結合）
     *
     * @generated from field: map<string, string> filters = 3;
     */
    filters: {
        [key: string]: string;
    };
    /**
     * 件数上限（既定 100、最大 1000）
     *
     * @generated from field: int32 limit = 4;
     */
    limit: number;
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 5;
     */
    context?: TenantContext;
    constructor(data?: PartialMessage<QueryAuditRequest>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.audit.v1.QueryAuditRequest";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): QueryAuditRequest;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): QueryAuditRequest;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): QueryAuditRequest;
    static equals(a: QueryAuditRequest | PlainMessage<QueryAuditRequest> | undefined, b: QueryAuditRequest | PlainMessage<QueryAuditRequest> | undefined): boolean;
}
/**
 * Query 応答
 *
 * @generated from message k1s0.tier1.audit.v1.QueryAuditResponse
 */
export declare class QueryAuditResponse extends Message<QueryAuditResponse> {
    /**
     * 検索結果（時刻昇順、出力時に PII Mask 自動適用）
     *
     * @generated from field: repeated k1s0.tier1.audit.v1.AuditEvent events = 1;
     */
    events: AuditEvent[];
    constructor(data?: PartialMessage<QueryAuditResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.audit.v1.QueryAuditResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): QueryAuditResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): QueryAuditResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): QueryAuditResponse;
    static equals(a: QueryAuditResponse | PlainMessage<QueryAuditResponse> | undefined, b: QueryAuditResponse | PlainMessage<QueryAuditResponse> | undefined): boolean;
}
/**
 * VerifyChain リクエスト（FR-T1-AUDIT-002）
 *
 * @generated from message k1s0.tier1.audit.v1.VerifyChainRequest
 */
export declare class VerifyChainRequest extends Message<VerifyChainRequest> {
    /**
     * 範囲開始（任意）。未指定（zero）はテナント配下の全履歴を対象。
     *
     * @generated from field: google.protobuf.Timestamp from = 1;
     */
    from?: Timestamp;
    /**
     * 範囲終了（任意）。未指定（zero）は最新まで。
     *
     * @generated from field: google.protobuf.Timestamp to = 2;
     */
    to?: Timestamp;
    /**
     * 呼出元コンテキスト（テナント境界の検証に必須）
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 3;
     */
    context?: TenantContext;
    constructor(data?: PartialMessage<VerifyChainRequest>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.audit.v1.VerifyChainRequest";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): VerifyChainRequest;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): VerifyChainRequest;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): VerifyChainRequest;
    static equals(a: VerifyChainRequest | PlainMessage<VerifyChainRequest> | undefined, b: VerifyChainRequest | PlainMessage<VerifyChainRequest> | undefined): boolean;
}
/**
 * VerifyChain 応答
 *
 * @generated from message k1s0.tier1.audit.v1.VerifyChainResponse
 */
export declare class VerifyChainResponse extends Message<VerifyChainResponse> {
    /**
     * チェーン全体の整合性が取れていれば true。
     *
     * @generated from field: bool valid = 1;
     */
    valid: boolean;
    /**
     * 検証対象だったイベント件数（valid に関わらず set される）。
     *
     * @generated from field: int64 checked_count = 2;
     */
    checkedCount: bigint;
    /**
     * valid=false 時のみ意味あり。最初に不整合を検出した sequence_number。
     * valid=true 時は 0。
     *
     * @generated from field: int64 first_bad_sequence = 3;
     */
    firstBadSequence: bigint;
    /**
     * 不整合の理由（"prev_hash mismatch" / "event_hash mismatch" / "tenant boundary" 等）。
     * valid=true 時は空文字。
     *
     * @generated from field: string reason = 4;
     */
    reason: string;
    constructor(data?: PartialMessage<VerifyChainResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.audit.v1.VerifyChainResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): VerifyChainResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): VerifyChainResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): VerifyChainResponse;
    static equals(a: VerifyChainResponse | PlainMessage<VerifyChainResponse> | undefined, b: VerifyChainResponse | PlainMessage<VerifyChainResponse> | undefined): boolean;
}
/**
 * Export リクエスト（FR-T1-AUDIT-002 疑似 IF "Audit.Export"）
 *
 * @generated from message k1s0.tier1.audit.v1.ExportAuditRequest
 */
export declare class ExportAuditRequest extends Message<ExportAuditRequest> {
    /**
     * 範囲開始（任意）。未指定（zero）は全履歴の先頭。
     *
     * @generated from field: google.protobuf.Timestamp from = 1;
     */
    from?: Timestamp;
    /**
     * 範囲終了（任意）。未指定（zero）は最新まで。
     *
     * @generated from field: google.protobuf.Timestamp to = 2;
     */
    to?: Timestamp;
    /**
     * 出力フォーマット。EXPORT_FORMAT_UNSPECIFIED は NDJSON 扱い。
     *
     * @generated from field: k1s0.tier1.audit.v1.ExportFormat format = 3;
     */
    format: ExportFormat;
    /**
     * 1 chunk あたりの最大バイト数（既定 65536、上限 1048576）。
     *
     * @generated from field: int32 chunk_bytes = 4;
     */
    chunkBytes: number;
    /**
     * 呼出元コンテキスト（テナント境界の検証に必須）。
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 5;
     */
    context?: TenantContext;
    constructor(data?: PartialMessage<ExportAuditRequest>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.audit.v1.ExportAuditRequest";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): ExportAuditRequest;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): ExportAuditRequest;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): ExportAuditRequest;
    static equals(a: ExportAuditRequest | PlainMessage<ExportAuditRequest> | undefined, b: ExportAuditRequest | PlainMessage<ExportAuditRequest> | undefined): boolean;
}
/**
 * Export 応答（server-streaming）の 1 チャンク
 *
 * @generated from message k1s0.tier1.audit.v1.ExportAuditChunk
 */
export declare class ExportAuditChunk extends Message<ExportAuditChunk> {
    /**
     * フォーマット済みデータの 1 部分（バイナリ透過、UTF-8 を想定）。
     *
     * @generated from field: bytes data = 1;
     */
    data: Uint8Array;
    /**
     * 0 起点のチャンク連番（再構成 / 監査時の参照用）。
     *
     * @generated from field: int64 sequence = 2;
     */
    sequence: bigint;
    /**
     * この chunk に含まれる event 数（chunk_bytes ベースの場合は variable）。
     *
     * @generated from field: int64 event_count = 3;
     */
    eventCount: bigint;
    /**
     * ストリーム末尾の chunk なら true。最後の "]" や EOF newline を含む。
     *
     * @generated from field: bool is_last = 4;
     */
    isLast: boolean;
    constructor(data?: PartialMessage<ExportAuditChunk>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.audit.v1.ExportAuditChunk";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): ExportAuditChunk;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): ExportAuditChunk;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): ExportAuditChunk;
    static equals(a: ExportAuditChunk | PlainMessage<ExportAuditChunk> | undefined, b: ExportAuditChunk | PlainMessage<ExportAuditChunk> | undefined): boolean;
}
//# sourceMappingURL=audit_service_pb.d.ts.map