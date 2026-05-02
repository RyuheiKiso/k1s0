import { ClassifyRequest, ClassifyResponse, MaskRequest, MaskResponse, PseudonymizeRequest, PseudonymizeResponse } from "./pii_service_pb.js";
import { MethodKind } from "@bufbuild/protobuf";
/**
 * PII API。t1-pii Pod は純関数（ステートレス）で副作用なし。
 *
 * @generated from service k1s0.tier1.pii.v1.PiiService
 */
export declare const PiiService: {
    readonly typeName: "k1s0.tier1.pii.v1.PiiService";
    readonly methods: {
        /**
         * PII 種別の検出（テキスト → findings 列）
         *
         * @generated from rpc k1s0.tier1.pii.v1.PiiService.Classify
         */
        readonly classify: {
            readonly name: "Classify";
            readonly I: typeof ClassifyRequest;
            readonly O: typeof ClassifyResponse;
            readonly kind: MethodKind.Unary;
        };
        /**
         * マスキング（テキスト → 置換後テキスト + findings）
         *
         * @generated from rpc k1s0.tier1.pii.v1.PiiService.Mask
         */
        readonly mask: {
            readonly name: "Mask";
            readonly I: typeof MaskRequest;
            readonly O: typeof MaskResponse;
            readonly kind: MethodKind.Unary;
        };
        /**
         * 仮名化（FR-T1-PII-002）。HMAC-SHA256(salt, value) を URL-safe base64 で返す。
         * 同一 salt + 同一入力で同一出力（決定論的）。salt は OpenBao 等で管理し直接露出しない。
         *
         * @generated from rpc k1s0.tier1.pii.v1.PiiService.Pseudonymize
         */
        readonly pseudonymize: {
            readonly name: "Pseudonymize";
            readonly I: typeof PseudonymizeRequest;
            readonly O: typeof PseudonymizeResponse;
            readonly kind: MethodKind.Unary;
        };
    };
};
//# sourceMappingURL=pii_service_connect.d.ts.map