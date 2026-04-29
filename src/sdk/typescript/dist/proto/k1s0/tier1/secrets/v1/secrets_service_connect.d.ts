import { BulkGetSecretRequest, BulkGetSecretResponse, GetDynamicSecretRequest, GetDynamicSecretResponse, GetSecretRequest, GetSecretResponse, RotateSecretRequest, RotateSecretResponse } from "./secrets_service_pb.js";
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
    };
};
//# sourceMappingURL=secrets_service_connect.d.ts.map