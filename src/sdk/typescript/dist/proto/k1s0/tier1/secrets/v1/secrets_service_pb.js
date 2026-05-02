// 本ファイルは tier1 公開 Secrets API の正式 proto。
// OpenBao 経由でテナントスコープのシークレットを取得・ローテーションする。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/04_Secrets_API.md
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/04_Secrets_API.md
//
// 関連要件: FR-T1-SECRETS-001〜004
// proto 構文宣言（proto3）
import { Message, proto3, protoInt64 } from "@bufbuild/protobuf";
import { TenantContext } from "../../common/v1/common_pb.js";
/**
 * Get リクエスト
 *
 * @generated from message k1s0.tier1.secrets.v1.GetSecretRequest
 */
export class GetSecretRequest extends Message {
    /**
     * シークレット名（テナント境界を超えた参照は即 PermissionDenied）
     *
     * @generated from field: string name = 1;
     */
    name = "";
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 2;
     */
    context;
    /**
     * 省略時は最新、明示で旧バージョン取得可（grace_period 中のみ）
     *
     * @generated from field: optional int32 version = 3;
     */
    version;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.secrets.v1.GetSecretRequest";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "name", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 2, name: "context", kind: "message", T: TenantContext },
        { no: 3, name: "version", kind: "scalar", T: 5 /* ScalarType.INT32 */, opt: true },
    ]);
    static fromBinary(bytes, options) {
        return new GetSecretRequest().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new GetSecretRequest().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new GetSecretRequest().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(GetSecretRequest, a, b);
    }
}
/**
 * Get 応答
 *
 * @generated from message k1s0.tier1.secrets.v1.GetSecretResponse
 */
export class GetSecretResponse extends Message {
    /**
     * 値（Base64 エンコード必要時はクライアント側で判断、複数キーの key=value マップ）
     *
     * @generated from field: map<string, string> values = 1;
     */
    values = {};
    /**
     * バージョン（ローテーション追跡用）
     *
     * @generated from field: int32 version = 2;
     */
    version = 0;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.secrets.v1.GetSecretResponse";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "values", kind: "map", K: 9 /* ScalarType.STRING */, V: { kind: "scalar", T: 9 /* ScalarType.STRING */ } },
        { no: 2, name: "version", kind: "scalar", T: 5 /* ScalarType.INT32 */ },
    ]);
    static fromBinary(bytes, options) {
        return new GetSecretResponse().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new GetSecretResponse().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new GetSecretResponse().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(GetSecretResponse, a, b);
    }
}
/**
 * BulkGet リクエスト
 *
 * @generated from message k1s0.tier1.secrets.v1.BulkGetSecretRequest
 */
export class BulkGetSecretRequest extends Message {
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 1;
     */
    context;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.secrets.v1.BulkGetSecretRequest";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "context", kind: "message", T: TenantContext },
    ]);
    static fromBinary(bytes, options) {
        return new BulkGetSecretRequest().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new BulkGetSecretRequest().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new BulkGetSecretRequest().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(BulkGetSecretRequest, a, b);
    }
}
/**
 * BulkGet 応答
 *
 * @generated from message k1s0.tier1.secrets.v1.BulkGetSecretResponse
 */
export class BulkGetSecretResponse extends Message {
    /**
     * 結果マップ（シークレット名 → GetSecretResponse）
     *
     * @generated from field: map<string, k1s0.tier1.secrets.v1.GetSecretResponse> results = 1;
     */
    results = {};
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.secrets.v1.BulkGetSecretResponse";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "results", kind: "map", K: 9 /* ScalarType.STRING */, V: { kind: "message", T: GetSecretResponse } },
    ]);
    static fromBinary(bytes, options) {
        return new BulkGetSecretResponse().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new BulkGetSecretResponse().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new BulkGetSecretResponse().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(BulkGetSecretResponse, a, b);
    }
}
/**
 * Rotate リクエスト
 *
 * @generated from message k1s0.tier1.secrets.v1.RotateSecretRequest
 */
export class RotateSecretRequest extends Message {
    /**
     * ローテーション対象シークレット名
     *
     * @generated from field: string name = 1;
     */
    name = "";
    /**
     * 呼出元コンテキスト
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 2;
     */
    context;
    /**
     * 旧バージョンの猶予時間（0 は即無効、既定 3600 秒）
     * tier2 側の接続プール drain 時間を想定
     *
     * @generated from field: int32 grace_period_sec = 3;
     */
    gracePeriodSec = 0;
    /**
     * 動的シークレット（DB 資格情報等）の場合の発行ポリシー名
     *
     * @generated from field: optional string policy = 4;
     */
    policy;
    /**
     * 冪等性キー（同一キーでの再試行は同じ new_version を返す）
     *
     * @generated from field: string idempotency_key = 5;
     */
    idempotencyKey = "";
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.secrets.v1.RotateSecretRequest";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "name", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 2, name: "context", kind: "message", T: TenantContext },
        { no: 3, name: "grace_period_sec", kind: "scalar", T: 5 /* ScalarType.INT32 */ },
        { no: 4, name: "policy", kind: "scalar", T: 9 /* ScalarType.STRING */, opt: true },
        { no: 5, name: "idempotency_key", kind: "scalar", T: 9 /* ScalarType.STRING */ },
    ]);
    static fromBinary(bytes, options) {
        return new RotateSecretRequest().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new RotateSecretRequest().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new RotateSecretRequest().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(RotateSecretRequest, a, b);
    }
}
/**
 * Rotate 応答
 *
 * @generated from message k1s0.tier1.secrets.v1.RotateSecretResponse
 */
export class RotateSecretResponse extends Message {
    /**
     * ローテーション後の新バージョン
     *
     * @generated from field: int32 new_version = 1;
     */
    newVersion = 0;
    /**
     * 旧バージョン（grace_period_sec まで Get 可能）
     *
     * @generated from field: int32 previous_version = 2;
     */
    previousVersion = 0;
    /**
     * 新バージョン発効時刻（Unix epoch ミリ秒）
     *
     * @generated from field: int64 rotated_at_ms = 3;
     */
    rotatedAtMs = protoInt64.zero;
    /**
     * 動的シークレット時の TTL（静的シークレットでは 0）
     *
     * @generated from field: int32 ttl_sec = 4;
     */
    ttlSec = 0;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.secrets.v1.RotateSecretResponse";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "new_version", kind: "scalar", T: 5 /* ScalarType.INT32 */ },
        { no: 2, name: "previous_version", kind: "scalar", T: 5 /* ScalarType.INT32 */ },
        { no: 3, name: "rotated_at_ms", kind: "scalar", T: 3 /* ScalarType.INT64 */ },
        { no: 4, name: "ttl_sec", kind: "scalar", T: 5 /* ScalarType.INT32 */ },
    ]);
    static fromBinary(bytes, options) {
        return new RotateSecretResponse().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new RotateSecretResponse().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new RotateSecretResponse().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(RotateSecretResponse, a, b);
    }
}
/**
 * GetDynamic リクエスト（FR-T1-SECRETS-002）
 *
 * @generated from message k1s0.tier1.secrets.v1.GetDynamicSecretRequest
 */
export class GetDynamicSecretRequest extends Message {
    /**
     * 呼出元コンテキスト（テナント境界の検証に必須）
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 1;
     */
    context;
    /**
     * 発行エンジン名（"postgres" / "mysql" / "kafka" 等、OpenBao の database engine 種別）
     *
     * @generated from field: string engine = 2;
     */
    engine = "";
    /**
     * OpenBao 側で予め定義されたロール名（tenant_id でスコープされた role）
     *
     * @generated from field: string role = 3;
     */
    role = "";
    /**
     * TTL 秒数（0 = 既定 3600 秒 = 1 時間、最大 86400 秒 = 24 時間）
     *
     * @generated from field: int32 ttl_sec = 4;
     */
    ttlSec = 0;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.secrets.v1.GetDynamicSecretRequest";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "context", kind: "message", T: TenantContext },
        { no: 2, name: "engine", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 3, name: "role", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 4, name: "ttl_sec", kind: "scalar", T: 5 /* ScalarType.INT32 */ },
    ]);
    static fromBinary(bytes, options) {
        return new GetDynamicSecretRequest().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new GetDynamicSecretRequest().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new GetDynamicSecretRequest().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(GetDynamicSecretRequest, a, b);
    }
}
/**
 * GetDynamic 応答
 *
 * @generated from message k1s0.tier1.secrets.v1.GetDynamicSecretResponse
 */
export class GetDynamicSecretResponse extends Message {
    /**
     * 発行された credential 一式（"username" / "password" 等の key=value）
     *
     * @generated from field: map<string, string> values = 1;
     */
    values = {};
    /**
     * OpenBao 側 lease ID（renewal / revoke 用、削除時に呼び返す）
     *
     * @generated from field: string lease_id = 2;
     */
    leaseId = "";
    /**
     * 実際に付与された TTL 秒数（要求値が ceiling を超えたら短縮される）
     *
     * @generated from field: int32 ttl_sec = 3;
     */
    ttlSec = 0;
    /**
     * 発効時刻（Unix epoch ミリ秒）
     *
     * @generated from field: int64 issued_at_ms = 4;
     */
    issuedAtMs = protoInt64.zero;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.secrets.v1.GetDynamicSecretResponse";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "values", kind: "map", K: 9 /* ScalarType.STRING */, V: { kind: "scalar", T: 9 /* ScalarType.STRING */ } },
        { no: 2, name: "lease_id", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 3, name: "ttl_sec", kind: "scalar", T: 5 /* ScalarType.INT32 */ },
        { no: 4, name: "issued_at_ms", kind: "scalar", T: 3 /* ScalarType.INT64 */ },
    ]);
    static fromBinary(bytes, options) {
        return new GetDynamicSecretResponse().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new GetDynamicSecretResponse().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new GetDynamicSecretResponse().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(GetDynamicSecretResponse, a, b);
    }
}
/**
 * Encrypt リクエスト（FR-T1-SECRETS-003）。
 *
 * @generated from message k1s0.tier1.secrets.v1.EncryptRequest
 */
export class EncryptRequest extends Message {
    /**
     * 呼出元コンテキスト（テナント境界の検証に必須）。
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 1;
     */
    context;
    /**
     * 鍵ラベル（tier1 が <tenant_id>.<key_label> で自動 prefix する）。
     * 同一 tenant 内で鍵空間を分離するため、key_label は業務ドメイン名等で命名する。
     *
     * @generated from field: string key_name = 2;
     */
    keyName = "";
    /**
     * 暗号化対象の平文 bytes。
     *
     * @generated from field: bytes plaintext = 3;
     */
    plaintext = new Uint8Array(0);
    /**
     * AAD（Associated Authenticated Data）。GCM の追加認証データに渡す。
     * 通常は tenant_id + RPC 名 等を JSON encoded で詰める運用想定。空でも良い。
     *
     * @generated from field: bytes aad = 4;
     */
    aad = new Uint8Array(0);
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.secrets.v1.EncryptRequest";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "context", kind: "message", T: TenantContext },
        { no: 2, name: "key_name", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 3, name: "plaintext", kind: "scalar", T: 12 /* ScalarType.BYTES */ },
        { no: 4, name: "aad", kind: "scalar", T: 12 /* ScalarType.BYTES */ },
    ]);
    static fromBinary(bytes, options) {
        return new EncryptRequest().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new EncryptRequest().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new EncryptRequest().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(EncryptRequest, a, b);
    }
}
/**
 * Encrypt 応答。
 *
 * @generated from message k1s0.tier1.secrets.v1.EncryptResponse
 */
export class EncryptResponse extends Message {
    /**
     * 暗号文。フォーマット: [version:4 BE][nonce:12][ciphertext+tag]。
     * 鍵バージョン管理は ciphertext に埋め込まれ、Decrypt 時に自動的に解決される。
     *
     * @generated from field: bytes ciphertext = 1;
     */
    ciphertext = new Uint8Array(0);
    /**
     * 暗号化に使用した鍵バージョン（observability / audit 用）。
     *
     * @generated from field: int32 key_version = 2;
     */
    keyVersion = 0;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.secrets.v1.EncryptResponse";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "ciphertext", kind: "scalar", T: 12 /* ScalarType.BYTES */ },
        { no: 2, name: "key_version", kind: "scalar", T: 5 /* ScalarType.INT32 */ },
    ]);
    static fromBinary(bytes, options) {
        return new EncryptResponse().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new EncryptResponse().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new EncryptResponse().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(EncryptResponse, a, b);
    }
}
/**
 * Decrypt リクエスト（FR-T1-SECRETS-003）。
 *
 * @generated from message k1s0.tier1.secrets.v1.DecryptRequest
 */
export class DecryptRequest extends Message {
    /**
     * 呼出元コンテキスト（テナント境界の検証に必須）。
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 1;
     */
    context;
    /**
     * 鍵ラベル（Encrypt 時と同じ key_label を渡す）。
     *
     * @generated from field: string key_name = 2;
     */
    keyName = "";
    /**
     * 暗号文（Encrypt の出力をそのまま渡す）。
     *
     * @generated from field: bytes ciphertext = 3;
     */
    ciphertext = new Uint8Array(0);
    /**
     * AAD（Encrypt 時と同じ値が必須、GCM の整合性検証に使用）。
     *
     * @generated from field: bytes aad = 4;
     */
    aad = new Uint8Array(0);
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.secrets.v1.DecryptRequest";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "context", kind: "message", T: TenantContext },
        { no: 2, name: "key_name", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 3, name: "ciphertext", kind: "scalar", T: 12 /* ScalarType.BYTES */ },
        { no: 4, name: "aad", kind: "scalar", T: 12 /* ScalarType.BYTES */ },
    ]);
    static fromBinary(bytes, options) {
        return new DecryptRequest().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new DecryptRequest().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new DecryptRequest().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(DecryptRequest, a, b);
    }
}
/**
 * Decrypt 応答。
 *
 * @generated from message k1s0.tier1.secrets.v1.DecryptResponse
 */
export class DecryptResponse extends Message {
    /**
     * 復号された平文。
     *
     * @generated from field: bytes plaintext = 1;
     */
    plaintext = new Uint8Array(0);
    /**
     * 復号に使用した鍵バージョン（旧版鍵で暗号化された場合の追跡用）。
     *
     * @generated from field: int32 key_version = 2;
     */
    keyVersion = 0;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.secrets.v1.DecryptResponse";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "plaintext", kind: "scalar", T: 12 /* ScalarType.BYTES */ },
        { no: 2, name: "key_version", kind: "scalar", T: 5 /* ScalarType.INT32 */ },
    ]);
    static fromBinary(bytes, options) {
        return new DecryptResponse().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new DecryptResponse().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new DecryptResponse().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(DecryptResponse, a, b);
    }
}
/**
 * RotateKey リクエスト（FR-T1-SECRETS-003 受け入れ基準「鍵バージョン管理が自動」）。
 *
 * @generated from message k1s0.tier1.secrets.v1.RotateKeyRequest
 */
export class RotateKeyRequest extends Message {
    /**
     * 呼出元コンテキスト。
     *
     * @generated from field: k1s0.tier1.common.v1.TenantContext context = 1;
     */
    context;
    /**
     * 鍵ラベル（tier1 が <tenant_id>.<key_label> で自動 prefix する）。
     *
     * @generated from field: string key_name = 2;
     */
    keyName = "";
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.secrets.v1.RotateKeyRequest";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "context", kind: "message", T: TenantContext },
        { no: 2, name: "key_name", kind: "scalar", T: 9 /* ScalarType.STRING */ },
    ]);
    static fromBinary(bytes, options) {
        return new RotateKeyRequest().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new RotateKeyRequest().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new RotateKeyRequest().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(RotateKeyRequest, a, b);
    }
}
/**
 * RotateKey 応答。
 *
 * @generated from message k1s0.tier1.secrets.v1.RotateKeyResponse
 */
export class RotateKeyResponse extends Message {
    /**
     * ローテーション後の新バージョン番号（既存版は保持される、Decrypt 引き続き可）。
     *
     * @generated from field: int32 new_version = 1;
     */
    newVersion = 0;
    /**
     * ローテーション直前の旧バージョン（最大 = new_version - 1）。
     *
     * @generated from field: int32 previous_version = 2;
     */
    previousVersion = 0;
    /**
     * ローテーション時刻（Unix epoch ミリ秒）。
     *
     * @generated from field: int64 rotated_at_ms = 3;
     */
    rotatedAtMs = protoInt64.zero;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.secrets.v1.RotateKeyResponse";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "new_version", kind: "scalar", T: 5 /* ScalarType.INT32 */ },
        { no: 2, name: "previous_version", kind: "scalar", T: 5 /* ScalarType.INT32 */ },
        { no: 3, name: "rotated_at_ms", kind: "scalar", T: 3 /* ScalarType.INT64 */ },
    ]);
    static fromBinary(bytes, options) {
        return new RotateKeyResponse().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new RotateKeyResponse().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new RotateKeyResponse().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(RotateKeyResponse, a, b);
    }
}
//# sourceMappingURL=secrets_service_pb.js.map