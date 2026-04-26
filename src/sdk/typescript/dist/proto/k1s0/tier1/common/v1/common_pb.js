// 本ファイルは tier1 公開 12 API すべてが import する共通型を定義する。
// 個別 API の proto からは `import "k1s0/tier1/common/v1/common.proto";` で参照する。
//
// 定義範囲:
//   - TenantContext      : 全 RPC が伝搬するテナント識別コンテキスト
//   - ErrorDetail        : google.rpc.Status.details に埋め込むエラー詳細
//   - K1s0ErrorCategory  : 機械可読なエラーカテゴリ enum（10 値）
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/00_共通型定義.md
//   docs/02_構想設計/adr/ADR-TIER1-002-protobuf-grpc.md
//
// 変更ルール: 既存フィールド削除は破壊的変更、追加は MINOR 版で許可。
// proto 構文宣言（proto3）
import { Message, proto3 } from "@bufbuild/protobuf";
/**
 * 機械可読なエラーカテゴリ。docs `00_tier1_API共通規約.md` の 8 カテゴリ +
 * UNSPECIFIED + DEADLINE_EXCEEDED の計 10 値。
 * 新規追加は MINOR 版で許可、既存値の削除・意味変更は禁止。
 * 未知カテゴリを受け取った tier2/tier3 SDK は UNSPECIFIED として扱う。
 *
 * @generated from enum k1s0.tier1.common.v1.K1s0ErrorCategory
 */
export var K1s0ErrorCategory;
(function (K1s0ErrorCategory) {
    /**
     * 未指定（既定値、クライアントは UNKNOWN として扱う）
     *
     * @generated from enum value: K1S0_ERROR_UNSPECIFIED = 0;
     */
    K1s0ErrorCategory[K1s0ErrorCategory["K1S0_ERROR_UNSPECIFIED"] = 0] = "K1S0_ERROR_UNSPECIFIED";
    /**
     * 呼出側入力誤り（リトライ不可）
     *
     * @generated from enum value: K1S0_ERROR_INVALID_ARGUMENT = 1;
     */
    K1s0ErrorCategory[K1s0ErrorCategory["K1S0_ERROR_INVALID_ARGUMENT"] = 1] = "K1S0_ERROR_INVALID_ARGUMENT";
    /**
     * JWT 不在・署名不正・期限切れ
     *
     * @generated from enum value: K1S0_ERROR_UNAUTHENTICATED = 2;
     */
    K1s0ErrorCategory[K1s0ErrorCategory["K1S0_ERROR_UNAUTHENTICATED"] = 2] = "K1S0_ERROR_UNAUTHENTICATED";
    /**
     * RBAC 拒否・テナント越境・allowlist 外
     *
     * @generated from enum value: K1S0_ERROR_PERMISSION_DENIED = 3;
     */
    K1s0ErrorCategory[K1s0ErrorCategory["K1S0_ERROR_PERMISSION_DENIED"] = 3] = "K1S0_ERROR_PERMISSION_DENIED";
    /**
     * キー / バージョン / リソース未存在
     *
     * @generated from enum value: K1S0_ERROR_NOT_FOUND = 4;
     */
    K1s0ErrorCategory[K1s0ErrorCategory["K1S0_ERROR_NOT_FOUND"] = 4] = "K1S0_ERROR_NOT_FOUND";
    /**
     * ETag 不一致・冪等性キー衝突
     *
     * @generated from enum value: K1S0_ERROR_CONFLICT = 5;
     */
    K1s0ErrorCategory[K1s0ErrorCategory["K1S0_ERROR_CONFLICT"] = 5] = "K1S0_ERROR_CONFLICT";
    /**
     * レート制限・クォータ超過（retry_after_ms 必須）
     *
     * @generated from enum value: K1S0_ERROR_RESOURCE_EXHAUSTED = 6;
     */
    K1s0ErrorCategory[K1s0ErrorCategory["K1S0_ERROR_RESOURCE_EXHAUSTED"] = 6] = "K1S0_ERROR_RESOURCE_EXHAUSTED";
    /**
     * 一時的バックエンド不能（retry_after_ms 必須、指数バックオフ）
     *
     * @generated from enum value: K1S0_ERROR_UNAVAILABLE = 7;
     */
    K1s0ErrorCategory[K1s0ErrorCategory["K1S0_ERROR_UNAVAILABLE"] = 7] = "K1S0_ERROR_UNAVAILABLE";
    /**
     * tier1 バグ・未分類（Audit に Severity 2 で記録）
     *
     * @generated from enum value: K1S0_ERROR_INTERNAL = 8;
     */
    K1s0ErrorCategory[K1s0ErrorCategory["K1S0_ERROR_INTERNAL"] = 8] = "K1S0_ERROR_INTERNAL";
    /**
     * gRPC Deadline 超過（副作用未発生扱い、再試行可）
     *
     * @generated from enum value: K1S0_ERROR_DEADLINE_EXCEEDED = 9;
     */
    K1s0ErrorCategory[K1s0ErrorCategory["K1S0_ERROR_DEADLINE_EXCEEDED"] = 9] = "K1S0_ERROR_DEADLINE_EXCEEDED";
})(K1s0ErrorCategory || (K1s0ErrorCategory = {}));
// Retrieve enum metadata with: proto3.getEnumType(K1s0ErrorCategory)
proto3.util.setEnumType(K1s0ErrorCategory, "k1s0.tier1.common.v1.K1s0ErrorCategory", [
    { no: 0, name: "K1S0_ERROR_UNSPECIFIED" },
    { no: 1, name: "K1S0_ERROR_INVALID_ARGUMENT" },
    { no: 2, name: "K1S0_ERROR_UNAUTHENTICATED" },
    { no: 3, name: "K1S0_ERROR_PERMISSION_DENIED" },
    { no: 4, name: "K1S0_ERROR_NOT_FOUND" },
    { no: 5, name: "K1S0_ERROR_CONFLICT" },
    { no: 6, name: "K1S0_ERROR_RESOURCE_EXHAUSTED" },
    { no: 7, name: "K1S0_ERROR_UNAVAILABLE" },
    { no: 8, name: "K1S0_ERROR_INTERNAL" },
    { no: 9, name: "K1S0_ERROR_DEADLINE_EXCEEDED" },
]);
/**
 * 呼出元テナントを特定する識別子。gRPC メタデータヘッダ
 * （x-tenant-id / x-correlation-id / x-subject）と Request 側 message の
 * `context` フィールドの両方で伝搬し、interceptor で整合性を検証する。
 *
 * @generated from message k1s0.tier1.common.v1.TenantContext
 */
export class TenantContext extends Message {
    /**
     * テナント ID（UUID v4 文字列、tier1 が JWT クレームと突き合わせて検証）
     *
     * @generated from field: string tenant_id = 1;
     */
    tenantId = "";
    /**
     * 呼出元の主体（workload_id または user_id、SPIFFE ID 互換）
     *
     * @generated from field: string subject = 2;
     */
    subject = "";
    /**
     * 相関 ID（OTel traceparent と紐付けて全 tier 横断トレース）
     *
     * @generated from field: string correlation_id = 3;
     */
    correlationId = "";
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.common.v1.TenantContext";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "tenant_id", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 2, name: "subject", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 3, name: "correlation_id", kind: "scalar", T: 9 /* ScalarType.STRING */ },
    ]);
    static fromBinary(bytes, options) {
        return new TenantContext().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new TenantContext().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new TenantContext().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(TenantContext, a, b);
    }
}
/**
 * エラー詳細。tier1 は google.rpc.Status の `details` 配列に
 * 本メッセージを 1 つ埋め込んで返す。`code` で詳細検索、
 * `category` で switch 分岐、`retryable` でクライアント側再試行判定。
 *
 * @generated from message k1s0.tier1.common.v1.ErrorDetail
 */
export class ErrorDetail extends Message {
    /**
     * エラーコード（E-<CATEGORY>-<MODULE>-<NUMBER> 形式、文字列）
     *
     * @generated from field: string code = 1;
     */
    code = "";
    /**
     * 機械可読カテゴリ（switch 分岐用、enum 追加時は後方互換維持）
     *
     * @generated from field: k1s0.tier1.common.v1.K1s0ErrorCategory category = 5;
     */
    category = K1s0ErrorCategory.K1S0_ERROR_UNSPECIFIED;
    /**
     * 人間可読なメッセージ（テナント表示可、PII を含めてはならない）
     *
     * @generated from field: string message = 2;
     */
    message = "";
    /**
     * 再試行可否（true の場合クライアントは指数バックオフで再試行）
     *
     * @generated from field: bool retryable = 3;
     */
    retryable = false;
    /**
     * 再試行までの推奨待機時間（ミリ秒、retryable=true の時のみ意味を持つ）
     *
     * @generated from field: int32 retry_after_ms = 4;
     */
    retryAfterMs = 0;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.common.v1.ErrorDetail";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "code", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 5, name: "category", kind: "enum", T: proto3.getEnumType(K1s0ErrorCategory) },
        { no: 2, name: "message", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 3, name: "retryable", kind: "scalar", T: 8 /* ScalarType.BOOL */ },
        { no: 4, name: "retry_after_ms", kind: "scalar", T: 5 /* ScalarType.INT32 */ },
    ]);
    static fromBinary(bytes, options) {
        return new ErrorDetail().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new ErrorDetail().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new ErrorDetail().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(ErrorDetail, a, b);
    }
}
//# sourceMappingURL=common_pb.js.map