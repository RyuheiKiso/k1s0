import type { BinaryReadOptions, FieldList, JsonReadOptions, JsonValue, PartialMessage, PlainMessage } from "@bufbuild/protobuf";
import { Message, proto3 } from "@bufbuild/protobuf";
/**
 * 機械可読なエラーカテゴリ。docs `00_tier1_API共通規約.md` の 8 カテゴリ +
 * UNSPECIFIED + DEADLINE_EXCEEDED の計 10 値。
 * 新規追加は MINOR 版で許可、既存値の削除・意味変更は禁止。
 * 未知カテゴリを受け取った tier2/tier3 SDK は UNSPECIFIED として扱う。
 *
 * @generated from enum k1s0.tier1.common.v1.K1s0ErrorCategory
 */
export declare enum K1s0ErrorCategory {
    /**
     * 未指定（既定値、クライアントは UNKNOWN として扱う）
     *
     * @generated from enum value: K1S0_ERROR_UNSPECIFIED = 0;
     */
    K1S0_ERROR_UNSPECIFIED = 0,
    /**
     * 呼出側入力誤り（リトライ不可）
     *
     * @generated from enum value: K1S0_ERROR_INVALID_ARGUMENT = 1;
     */
    K1S0_ERROR_INVALID_ARGUMENT = 1,
    /**
     * JWT 不在・署名不正・期限切れ
     *
     * @generated from enum value: K1S0_ERROR_UNAUTHENTICATED = 2;
     */
    K1S0_ERROR_UNAUTHENTICATED = 2,
    /**
     * RBAC 拒否・テナント越境・allowlist 外
     *
     * @generated from enum value: K1S0_ERROR_PERMISSION_DENIED = 3;
     */
    K1S0_ERROR_PERMISSION_DENIED = 3,
    /**
     * キー / バージョン / リソース未存在
     *
     * @generated from enum value: K1S0_ERROR_NOT_FOUND = 4;
     */
    K1S0_ERROR_NOT_FOUND = 4,
    /**
     * ETag 不一致・冪等性キー衝突
     *
     * @generated from enum value: K1S0_ERROR_CONFLICT = 5;
     */
    K1S0_ERROR_CONFLICT = 5,
    /**
     * レート制限・クォータ超過（retry_after_ms 必須）
     *
     * @generated from enum value: K1S0_ERROR_RESOURCE_EXHAUSTED = 6;
     */
    K1S0_ERROR_RESOURCE_EXHAUSTED = 6,
    /**
     * 一時的バックエンド不能（retry_after_ms 必須、指数バックオフ）
     *
     * @generated from enum value: K1S0_ERROR_UNAVAILABLE = 7;
     */
    K1S0_ERROR_UNAVAILABLE = 7,
    /**
     * tier1 バグ・未分類（Audit に Severity 2 で記録）
     *
     * @generated from enum value: K1S0_ERROR_INTERNAL = 8;
     */
    K1S0_ERROR_INTERNAL = 8,
    /**
     * gRPC Deadline 超過（副作用未発生扱い、再試行可）
     *
     * @generated from enum value: K1S0_ERROR_DEADLINE_EXCEEDED = 9;
     */
    K1S0_ERROR_DEADLINE_EXCEEDED = 9
}
/**
 * 呼出元テナントを特定する識別子。gRPC メタデータヘッダ
 * （x-tenant-id / x-correlation-id / x-subject）と Request 側 message の
 * `context` フィールドの両方で伝搬し、interceptor で整合性を検証する。
 *
 * @generated from message k1s0.tier1.common.v1.TenantContext
 */
export declare class TenantContext extends Message<TenantContext> {
    /**
     * テナント ID（UUID v4 文字列、tier1 が JWT クレームと突き合わせて検証）
     *
     * @generated from field: string tenant_id = 1;
     */
    tenantId: string;
    /**
     * 呼出元の主体（workload_id または user_id、SPIFFE ID 互換）
     *
     * @generated from field: string subject = 2;
     */
    subject: string;
    /**
     * 相関 ID（OTel traceparent と紐付けて全 tier 横断トレース）
     *
     * @generated from field: string correlation_id = 3;
     */
    correlationId: string;
    constructor(data?: PartialMessage<TenantContext>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.common.v1.TenantContext";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): TenantContext;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): TenantContext;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): TenantContext;
    static equals(a: TenantContext | PlainMessage<TenantContext> | undefined, b: TenantContext | PlainMessage<TenantContext> | undefined): boolean;
}
/**
 * エラー詳細。tier1 は google.rpc.Status の `details` 配列に
 * 本メッセージを 1 つ埋め込んで返す。`code` で詳細検索、
 * `category` で switch 分岐、`retryable` でクライアント側再試行判定。
 *
 * @generated from message k1s0.tier1.common.v1.ErrorDetail
 */
export declare class ErrorDetail extends Message<ErrorDetail> {
    /**
     * エラーコード（E-<CATEGORY>-<MODULE>-<NUMBER> 形式、文字列）
     *
     * @generated from field: string code = 1;
     */
    code: string;
    /**
     * 機械可読カテゴリ（switch 分岐用、enum 追加時は後方互換維持）
     *
     * @generated from field: k1s0.tier1.common.v1.K1s0ErrorCategory category = 5;
     */
    category: K1s0ErrorCategory;
    /**
     * 人間可読なメッセージ（テナント表示可、PII を含めてはならない）
     *
     * @generated from field: string message = 2;
     */
    message: string;
    /**
     * 再試行可否（true の場合クライアントは指数バックオフで再試行）
     *
     * @generated from field: bool retryable = 3;
     */
    retryable: boolean;
    /**
     * 再試行までの推奨待機時間（ミリ秒、retryable=true の時のみ意味を持つ）
     *
     * @generated from field: int32 retry_after_ms = 4;
     */
    retryAfterMs: number;
    constructor(data?: PartialMessage<ErrorDetail>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.common.v1.ErrorDetail";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): ErrorDetail;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): ErrorDetail;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): ErrorDetail;
    static equals(a: ErrorDetail | PlainMessage<ErrorDetail> | undefined, b: ErrorDetail | PlainMessage<ErrorDetail> | undefined): boolean;
}
//# sourceMappingURL=common_pb.d.ts.map