import type { BinaryReadOptions, FieldList, JsonReadOptions, JsonValue, PartialMessage, PlainMessage } from "@bufbuild/protobuf";
import { Message, proto3 } from "@bufbuild/protobuf";
import { TenantContext } from "../../common/v1/common_pb.js";
/**
 * Get リクエスト
 *
 * @generated from message k1s0.tier1.secrets.v1.GetSecretRequest
 */
export declare class GetSecretRequest extends Message<GetSecretRequest> {
    /**
     * シークレット名（テナント境界を超えた参照は即 PermissionDenied）
     *
     * @generated from field: string name = 1;
     */
    name: string;
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 2;
     */
    context?: TenantContext;
    /**
     * 省略時は最新、明示で旧バージョン取得可（grace_period 中のみ）
     *
     * @generated from field: optional int32 version = 3;
     */
    version?: number;
    constructor(data?: PartialMessage<GetSecretRequest>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.secrets.v1.GetSecretRequest";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): GetSecretRequest;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): GetSecretRequest;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): GetSecretRequest;
    static equals(a: GetSecretRequest | PlainMessage<GetSecretRequest> | undefined, b: GetSecretRequest | PlainMessage<GetSecretRequest> | undefined): boolean;
}
/**
 * Get 応答
 *
 * @generated from message k1s0.tier1.secrets.v1.GetSecretResponse
 */
export declare class GetSecretResponse extends Message<GetSecretResponse> {
    /**
     * 値（Base64 エンコード必要時はクライアント側で判断、複数キーの key=value マップ）
     *
     * @generated from field: map<string, string> values = 1;
     */
    values: {
        [key: string]: string;
    };
    /**
     * バージョン（ローテーション追跡用）
     *
     * @generated from field: int32 version = 2;
     */
    version: number;
    constructor(data?: PartialMessage<GetSecretResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.secrets.v1.GetSecretResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): GetSecretResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): GetSecretResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): GetSecretResponse;
    static equals(a: GetSecretResponse | PlainMessage<GetSecretResponse> | undefined, b: GetSecretResponse | PlainMessage<GetSecretResponse> | undefined): boolean;
}
/**
 * BulkGet リクエスト
 *
 * @generated from message k1s0.tier1.secrets.v1.BulkGetSecretRequest
 */
export declare class BulkGetSecretRequest extends Message<BulkGetSecretRequest> {
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 1;
     */
    context?: TenantContext;
    constructor(data?: PartialMessage<BulkGetSecretRequest>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.secrets.v1.BulkGetSecretRequest";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): BulkGetSecretRequest;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): BulkGetSecretRequest;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): BulkGetSecretRequest;
    static equals(a: BulkGetSecretRequest | PlainMessage<BulkGetSecretRequest> | undefined, b: BulkGetSecretRequest | PlainMessage<BulkGetSecretRequest> | undefined): boolean;
}
/**
 * BulkGet 応答
 *
 * @generated from message k1s0.tier1.secrets.v1.BulkGetSecretResponse
 */
export declare class BulkGetSecretResponse extends Message<BulkGetSecretResponse> {
    /**
     * 結果マップ（シークレット名 → GetSecretResponse）
     *
     * @generated from field: map<string, k1s0.tier1.secrets.v1.GetSecretResponse> results = 1;
     */
    results: {
        [key: string]: GetSecretResponse;
    };
    constructor(data?: PartialMessage<BulkGetSecretResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.secrets.v1.BulkGetSecretResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): BulkGetSecretResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): BulkGetSecretResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): BulkGetSecretResponse;
    static equals(a: BulkGetSecretResponse | PlainMessage<BulkGetSecretResponse> | undefined, b: BulkGetSecretResponse | PlainMessage<BulkGetSecretResponse> | undefined): boolean;
}
/**
 * Rotate リクエスト
 *
 * @generated from message k1s0.tier1.secrets.v1.RotateSecretRequest
 */
export declare class RotateSecretRequest extends Message<RotateSecretRequest> {
    /**
     * ローテーション対象シークレット名
     *
     * @generated from field: string name = 1;
     */
    name: string;
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 2;
     */
    context?: TenantContext;
    /**
     * 旧バージョンの猶予時間（0 は即無効、既定 3600 秒）
     * tier2 側の接続プール drain 時間を想定
     *
     * @generated from field: int32 grace_period_sec = 3;
     */
    gracePeriodSec: number;
    /**
     * 動的シークレット（DB 資格情報等）の場合の発行ポリシー名
     *
     * @generated from field: optional string policy = 4;
     */
    policy?: string;
    /**
     * 冪等性キー（同一キーでの再試行は同じ new_version を返す）
     *
     * @generated from field: string idempotency_key = 5;
     */
    idempotencyKey: string;
    constructor(data?: PartialMessage<RotateSecretRequest>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.secrets.v1.RotateSecretRequest";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): RotateSecretRequest;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): RotateSecretRequest;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): RotateSecretRequest;
    static equals(a: RotateSecretRequest | PlainMessage<RotateSecretRequest> | undefined, b: RotateSecretRequest | PlainMessage<RotateSecretRequest> | undefined): boolean;
}
/**
 * Rotate 応答
 *
 * @generated from message k1s0.tier1.secrets.v1.RotateSecretResponse
 */
export declare class RotateSecretResponse extends Message<RotateSecretResponse> {
    /**
     * ローテーション後の新バージョン
     *
     * @generated from field: int32 new_version = 1;
     */
    newVersion: number;
    /**
     * 旧バージョン（grace_period_sec まで Get 可能）
     *
     * @generated from field: int32 previous_version = 2;
     */
    previousVersion: number;
    /**
     * 新バージョン発効時刻（Unix epoch ミリ秒）
     *
     * @generated from field: int64 rotated_at_ms = 3;
     */
    rotatedAtMs: bigint;
    /**
     * 動的シークレット時の TTL（静的シークレットでは 0）
     *
     * @generated from field: int32 ttl_sec = 4;
     */
    ttlSec: number;
    constructor(data?: PartialMessage<RotateSecretResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.secrets.v1.RotateSecretResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): RotateSecretResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): RotateSecretResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): RotateSecretResponse;
    static equals(a: RotateSecretResponse | PlainMessage<RotateSecretResponse> | undefined, b: RotateSecretResponse | PlainMessage<RotateSecretResponse> | undefined): boolean;
}
/**
 * GetDynamic リクエスト（FR-T1-SECRETS-002）
 *
 * @generated from message k1s0.tier1.secrets.v1.GetDynamicSecretRequest
 */
export declare class GetDynamicSecretRequest extends Message<GetDynamicSecretRequest> {
    /**
     * 呼出元コンテキスト（テナント境界の検証に必須）
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 1;
     */
    context?: TenantContext;
    /**
     * 発行エンジン名（"postgres" / "mysql" / "kafka" 等、OpenBao の database engine 種別）
     *
     * @generated from field: string engine = 2;
     */
    engine: string;
    /**
     * OpenBao 側で予め定義されたロール名（tenant_id でスコープされた role）
     *
     * @generated from field: string role = 3;
     */
    role: string;
    /**
     * TTL 秒数（0 = 既定 3600 秒 = 1 時間、最大 86400 秒 = 24 時間）
     *
     * @generated from field: int32 ttl_sec = 4;
     */
    ttlSec: number;
    constructor(data?: PartialMessage<GetDynamicSecretRequest>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.secrets.v1.GetDynamicSecretRequest";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): GetDynamicSecretRequest;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): GetDynamicSecretRequest;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): GetDynamicSecretRequest;
    static equals(a: GetDynamicSecretRequest | PlainMessage<GetDynamicSecretRequest> | undefined, b: GetDynamicSecretRequest | PlainMessage<GetDynamicSecretRequest> | undefined): boolean;
}
/**
 * GetDynamic 応答
 *
 * @generated from message k1s0.tier1.secrets.v1.GetDynamicSecretResponse
 */
export declare class GetDynamicSecretResponse extends Message<GetDynamicSecretResponse> {
    /**
     * 発行された credential 一式（"username" / "password" 等の key=value）
     *
     * @generated from field: map<string, string> values = 1;
     */
    values: {
        [key: string]: string;
    };
    /**
     * OpenBao 側 lease ID（renewal / revoke 用、削除時に呼び返す）
     *
     * @generated from field: string lease_id = 2;
     */
    leaseId: string;
    /**
     * 実際に付与された TTL 秒数（要求値が ceiling を超えたら短縮される）
     *
     * @generated from field: int32 ttl_sec = 3;
     */
    ttlSec: number;
    /**
     * 発効時刻（Unix epoch ミリ秒）
     *
     * @generated from field: int64 issued_at_ms = 4;
     */
    issuedAtMs: bigint;
    constructor(data?: PartialMessage<GetDynamicSecretResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.secrets.v1.GetDynamicSecretResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): GetDynamicSecretResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): GetDynamicSecretResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): GetDynamicSecretResponse;
    static equals(a: GetDynamicSecretResponse | PlainMessage<GetDynamicSecretResponse> | undefined, b: GetDynamicSecretResponse | PlainMessage<GetDynamicSecretResponse> | undefined): boolean;
}
/**
 * Encrypt リクエスト（FR-T1-SECRETS-003）。
 *
 * @generated from message k1s0.tier1.secrets.v1.EncryptRequest
 */
export declare class EncryptRequest extends Message<EncryptRequest> {
    /**
     * 呼出元コンテキスト（テナント境界の検証に必須）。
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 1;
     */
    context?: TenantContext;
    /**
     * 鍵ラベル（tier1 が <tenant_id>.<key_label> で自動 prefix する）。
     * 同一 tenant 内で鍵空間を分離するため、key_label は業務ドメイン名等で命名する。
     *
     * @generated from field: string key_name = 2;
     */
    keyName: string;
    /**
     * 暗号化対象の平文 bytes。
     *
     * @generated from field: bytes plaintext = 3;
     */
    plaintext: Uint8Array;
    /**
     * AAD（Associated Authenticated Data）。GCM の追加認証データに渡す。
     * 通常は tenant_id + RPC 名 等を JSON encoded で詰める運用想定。空でも良い。
     *
     * @generated from field: bytes aad = 4;
     */
    aad: Uint8Array;
    constructor(data?: PartialMessage<EncryptRequest>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.secrets.v1.EncryptRequest";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): EncryptRequest;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): EncryptRequest;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): EncryptRequest;
    static equals(a: EncryptRequest | PlainMessage<EncryptRequest> | undefined, b: EncryptRequest | PlainMessage<EncryptRequest> | undefined): boolean;
}
/**
 * Encrypt 応答。
 *
 * @generated from message k1s0.tier1.secrets.v1.EncryptResponse
 */
export declare class EncryptResponse extends Message<EncryptResponse> {
    /**
     * 暗号文。フォーマット: [version:4 BE][nonce:12][ciphertext+tag]。
     * 鍵バージョン管理は ciphertext に埋め込まれ、Decrypt 時に自動的に解決される。
     *
     * @generated from field: bytes ciphertext = 1;
     */
    ciphertext: Uint8Array;
    /**
     * 暗号化に使用した鍵バージョン（observability / audit 用）。
     *
     * @generated from field: int32 key_version = 2;
     */
    keyVersion: number;
    constructor(data?: PartialMessage<EncryptResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.secrets.v1.EncryptResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): EncryptResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): EncryptResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): EncryptResponse;
    static equals(a: EncryptResponse | PlainMessage<EncryptResponse> | undefined, b: EncryptResponse | PlainMessage<EncryptResponse> | undefined): boolean;
}
/**
 * Decrypt リクエスト（FR-T1-SECRETS-003）。
 *
 * @generated from message k1s0.tier1.secrets.v1.DecryptRequest
 */
export declare class DecryptRequest extends Message<DecryptRequest> {
    /**
     * 呼出元コンテキスト（テナント境界の検証に必須）。
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 1;
     */
    context?: TenantContext;
    /**
     * 鍵ラベル（Encrypt 時と同じ key_label を渡す）。
     *
     * @generated from field: string key_name = 2;
     */
    keyName: string;
    /**
     * 暗号文（Encrypt の出力をそのまま渡す）。
     *
     * @generated from field: bytes ciphertext = 3;
     */
    ciphertext: Uint8Array;
    /**
     * AAD（Encrypt 時と同じ値が必須、GCM の整合性検証に使用）。
     *
     * @generated from field: bytes aad = 4;
     */
    aad: Uint8Array;
    constructor(data?: PartialMessage<DecryptRequest>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.secrets.v1.DecryptRequest";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): DecryptRequest;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): DecryptRequest;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): DecryptRequest;
    static equals(a: DecryptRequest | PlainMessage<DecryptRequest> | undefined, b: DecryptRequest | PlainMessage<DecryptRequest> | undefined): boolean;
}
/**
 * Decrypt 応答。
 *
 * @generated from message k1s0.tier1.secrets.v1.DecryptResponse
 */
export declare class DecryptResponse extends Message<DecryptResponse> {
    /**
     * 復号された平文。
     *
     * @generated from field: bytes plaintext = 1;
     */
    plaintext: Uint8Array;
    /**
     * 復号に使用した鍵バージョン（旧版鍵で暗号化された場合の追跡用）。
     *
     * @generated from field: int32 key_version = 2;
     */
    keyVersion: number;
    constructor(data?: PartialMessage<DecryptResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.secrets.v1.DecryptResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): DecryptResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): DecryptResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): DecryptResponse;
    static equals(a: DecryptResponse | PlainMessage<DecryptResponse> | undefined, b: DecryptResponse | PlainMessage<DecryptResponse> | undefined): boolean;
}
/**
 * RotateKey リクエスト（FR-T1-SECRETS-003 受け入れ基準「鍵バージョン管理が自動」）。
 *
 * @generated from message k1s0.tier1.secrets.v1.RotateKeyRequest
 */
export declare class RotateKeyRequest extends Message<RotateKeyRequest> {
    /**
     * 呼出元コンテキスト。
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 1;
     */
    context?: TenantContext;
    /**
     * 鍵ラベル（tier1 が <tenant_id>.<key_label> で自動 prefix する）。
     *
     * @generated from field: string key_name = 2;
     */
    keyName: string;
    constructor(data?: PartialMessage<RotateKeyRequest>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.secrets.v1.RotateKeyRequest";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): RotateKeyRequest;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): RotateKeyRequest;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): RotateKeyRequest;
    static equals(a: RotateKeyRequest | PlainMessage<RotateKeyRequest> | undefined, b: RotateKeyRequest | PlainMessage<RotateKeyRequest> | undefined): boolean;
}
/**
 * RotateKey 応答。
 *
 * @generated from message k1s0.tier1.secrets.v1.RotateKeyResponse
 */
export declare class RotateKeyResponse extends Message<RotateKeyResponse> {
    /**
     * ローテーション後の新バージョン番号（既存版は保持される、Decrypt 引き続き可）。
     *
     * @generated from field: int32 new_version = 1;
     */
    newVersion: number;
    /**
     * ローテーション直前の旧バージョン（最大 = new_version - 1）。
     *
     * @generated from field: int32 previous_version = 2;
     */
    previousVersion: number;
    /**
     * ローテーション時刻（Unix epoch ミリ秒）。
     *
     * @generated from field: int64 rotated_at_ms = 3;
     */
    rotatedAtMs: bigint;
    constructor(data?: PartialMessage<RotateKeyResponse>);
    static readonly runtime: typeof proto3;
    static readonly typeName = "k1s0.tier1.secrets.v1.RotateKeyResponse";
    static readonly fields: FieldList;
    static fromBinary(bytes: Uint8Array, options?: Partial<BinaryReadOptions>): RotateKeyResponse;
    static fromJson(jsonValue: JsonValue, options?: Partial<JsonReadOptions>): RotateKeyResponse;
    static fromJsonString(jsonString: string, options?: Partial<JsonReadOptions>): RotateKeyResponse;
    static equals(a: RotateKeyResponse | PlainMessage<RotateKeyResponse> | undefined, b: RotateKeyResponse | PlainMessage<RotateKeyResponse> | undefined): boolean;
}
//# sourceMappingURL=secrets_service_pb.d.ts.map