import { BooleanResponse, EvaluateRequest, GetFlagRequest, GetFlagResponse, ListFlagsRequest, ListFlagsResponse, NumberResponse, ObjectResponse, RegisterFlagRequest, RegisterFlagResponse, StringResponse } from "./feature_service_pb.js";
import { MethodKind } from "@bufbuild/protobuf";
/**
 * Feature Flag 評価 API。OpenFeature 互換、flagd 仕様準拠。
 *
 * @generated from service k1s0.tier1.feature.v1.FeatureService
 */
export declare const FeatureService: {
    readonly typeName: "k1s0.tier1.feature.v1.FeatureService";
    readonly methods: {
        /**
         * Boolean Flag 評価
         *
         * @generated from rpc k1s0.tier1.feature.v1.FeatureService.EvaluateBoolean
         */
        readonly evaluateBoolean: {
            readonly name: "EvaluateBoolean";
            readonly I: typeof EvaluateRequest;
            readonly O: typeof BooleanResponse;
            readonly kind: MethodKind.Unary;
        };
        /**
         * String Flag 評価（Variant）
         *
         * @generated from rpc k1s0.tier1.feature.v1.FeatureService.EvaluateString
         */
        readonly evaluateString: {
            readonly name: "EvaluateString";
            readonly I: typeof EvaluateRequest;
            readonly O: typeof StringResponse;
            readonly kind: MethodKind.Unary;
        };
        /**
         * 数値 Flag 評価
         *
         * @generated from rpc k1s0.tier1.feature.v1.FeatureService.EvaluateNumber
         */
        readonly evaluateNumber: {
            readonly name: "EvaluateNumber";
            readonly I: typeof EvaluateRequest;
            readonly O: typeof NumberResponse;
            readonly kind: MethodKind.Unary;
        };
        /**
         * JSON オブジェクト Flag 評価
         *
         * @generated from rpc k1s0.tier1.feature.v1.FeatureService.EvaluateObject
         */
        readonly evaluateObject: {
            readonly name: "EvaluateObject";
            readonly I: typeof EvaluateRequest;
            readonly O: typeof ObjectResponse;
            readonly kind: MethodKind.Unary;
        };
    };
};
/**
 * Flag 定義の登録・更新（リリース時点 提供）
 *
 * @generated from service k1s0.tier1.feature.v1.FeatureAdminService
 */
export declare const FeatureAdminService: {
    readonly typeName: "k1s0.tier1.feature.v1.FeatureAdminService";
    readonly methods: {
        /**
         * Flag 定義の登録（permission 種別は approval_id 必須）
         *
         * @generated from rpc k1s0.tier1.feature.v1.FeatureAdminService.RegisterFlag
         */
        readonly registerFlag: {
            readonly name: "RegisterFlag";
            readonly I: typeof RegisterFlagRequest;
            readonly O: typeof RegisterFlagResponse;
            readonly kind: MethodKind.Unary;
        };
        /**
         * Flag 定義の取得
         *
         * @generated from rpc k1s0.tier1.feature.v1.FeatureAdminService.GetFlag
         */
        readonly getFlag: {
            readonly name: "GetFlag";
            readonly I: typeof GetFlagRequest;
            readonly O: typeof GetFlagResponse;
            readonly kind: MethodKind.Unary;
        };
        /**
         * Flag 定義の一覧
         *
         * @generated from rpc k1s0.tier1.feature.v1.FeatureAdminService.ListFlags
         */
        readonly listFlags: {
            readonly name: "ListFlags";
            readonly I: typeof ListFlagsRequest;
            readonly O: typeof ListFlagsResponse;
            readonly kind: MethodKind.Unary;
        };
    };
};
//# sourceMappingURL=feature_service_connect.d.ts.map