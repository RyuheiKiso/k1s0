import { BulkGetSecretRequest, BulkGetSecretResponse, DecryptRequest, DecryptResponse, EncryptRequest, EncryptResponse, GetDynamicSecretRequest, GetDynamicSecretResponse, GetSecretRequest, GetSecretResponse, RotateKeyRequest, RotateKeyResponse, RotateSecretRequest, RotateSecretResponse } from "./secrets_service_pb.js";
import { MethodKind } from "@bufbuild/protobuf";
/**
 * Secrets API。OpenBao をバックエンドとし、tier1 が PII / アクセス制御を強制する。
 *
 * @generated from service k1s0.tier1.secrets.v1.SecretsService
 */
export declare const SecretsService: {
    readonly typeName: "k1s0.tier1.secrets.v1.SecretsService";
    readonly methods: {
        /**
         * 単一シークレット取得（テナント越境参照は即 PermissionDenied、FR-T1-SECRETS-001）
         *
         * @generated from rpc k1s0.tier1.secrets.v1.SecretsService.Get
         */
        readonly get: {
            readonly name: "Get";
            readonly I: typeof GetSecretRequest;
            readonly O: typeof GetSecretResponse;
            readonly kind: MethodKind.Unary;
        };
        /**
         * 一括取得（テナントに割当された全シークレット）
         *
         * @generated from rpc k1s0.tier1.secrets.v1.SecretsService.BulkGet
         */
        readonly bulkGet: {
            readonly name: "BulkGet";
            readonly I: typeof BulkGetSecretRequest;
            readonly O: typeof BulkGetSecretResponse;
            readonly kind: MethodKind.Unary;
        };
        /**
         * 動的シークレット発行（FR-T1-SECRETS-002）。
         * engine="postgres" 等の Database Engine から TTL 付き credential を都度発行する。
         * TTL 経過後は OpenBao が backend ユーザを自動失効（drop）させる。
         *
         * @generated from rpc k1s0.tier1.secrets.v1.SecretsService.GetDynamic
         */
        readonly getDynamic: {
            readonly name: "GetDynamic";
            readonly I: typeof GetDynamicSecretRequest;
            readonly O: typeof GetDynamicSecretResponse;
            readonly kind: MethodKind.Unary;
        };
        /**
         * ローテーション実行（FR-T1-SECRETS-004）
         * 成功時は new_version を返し、旧バージョンは grace_period_sec まで Get 可能。
         * 失敗時は K1s0Error を返し OpenBao 側は不変（トランザクショナル）。
         *
         * @generated from rpc k1s0.tier1.secrets.v1.SecretsService.Rotate
         */
        readonly rotate: {
            readonly name: "Rotate";
            readonly I: typeof RotateSecretRequest;
            readonly O: typeof RotateSecretResponse;
            readonly kind: MethodKind.Unary;
        };
        /**
         * Transit 暗号化（FR-T1-SECRETS-003）。AES-256-GCM 固定。
         * 鍵名は tier1 が <tenant_id>.<key_label> で自動 prefix する。
         * 鍵バージョンは ciphertext に埋め込まれ、Decrypt 時に自動的に正しい版で復号される。
         *
         * @generated from rpc k1s0.tier1.secrets.v1.SecretsService.Encrypt
         */
        readonly encrypt: {
            readonly name: "Encrypt";
            readonly I: typeof EncryptRequest;
            readonly O: typeof EncryptResponse;
            readonly kind: MethodKind.Unary;
        };
        /**
         * Transit 復号（FR-T1-SECRETS-003）。
         * ciphertext から鍵バージョンを取り出し、対応する鍵で復号する。
         * ローテーション後の旧版 ciphertext も自動で復号できる。
         *
         * @generated from rpc k1s0.tier1.secrets.v1.SecretsService.Decrypt
         */
        readonly decrypt: {
            readonly name: "Decrypt";
            readonly I: typeof DecryptRequest;
            readonly O: typeof DecryptResponse;
            readonly kind: MethodKind.Unary;
        };
        /**
         * Transit 鍵ローテーション（FR-T1-SECRETS-003 受け入れ基準「鍵バージョン管理が自動」）。
         * 新バージョン鍵を生成して current にする。以降の Encrypt は新版で行うが、
         * 旧版 ciphertext は引き続き Decrypt 可能（旧版鍵を保持）。
         *
         * @generated from rpc k1s0.tier1.secrets.v1.SecretsService.RotateKey
         */
        readonly rotateKey: {
            readonly name: "RotateKey";
            readonly I: typeof RotateKeyRequest;
            readonly O: typeof RotateKeyResponse;
            readonly kind: MethodKind.Unary;
        };
    };
};
//# sourceMappingURL=secrets_service_connect.d.ts.map