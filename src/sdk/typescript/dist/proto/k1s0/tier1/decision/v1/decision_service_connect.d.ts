import { BatchEvaluateRequest, BatchEvaluateResponse, EvaluateRequest, EvaluateResponse, GetRuleRequest, GetRuleResponse, ListVersionsRequest, ListVersionsResponse, RegisterRuleRequest, RegisterRuleResponse } from "./decision_service_pb.js";
import { MethodKind } from "@bufbuild/protobuf";
/**
 * Decision 評価 API。tier1 内の Rust 実装（ZEN Engine 統合）にディスパッチする。
 *
 * @generated from service k1s0.tier1.decision.v1.DecisionService
 */
export declare const DecisionService: {
    readonly typeName: "k1s0.tier1.decision.v1.DecisionService";
    readonly methods: {
        /**
         * ルール評価（同期、非決定要素を含むルールは登録時に弾かれる）
         *
         * @generated from rpc k1s0.tier1.decision.v1.DecisionService.Evaluate
         */
        readonly evaluate: {
            readonly name: "Evaluate";
            readonly I: typeof EvaluateRequest;
            readonly O: typeof EvaluateResponse;
            readonly kind: MethodKind.Unary;
        };
        /**
         * バッチ評価（複数入力を一括評価、JIT 最適化対象）
         *
         * @generated from rpc k1s0.tier1.decision.v1.DecisionService.BatchEvaluate
         */
        readonly batchEvaluate: {
            readonly name: "BatchEvaluate";
            readonly I: typeof BatchEvaluateRequest;
            readonly O: typeof BatchEvaluateResponse;
            readonly kind: MethodKind.Unary;
        };
    };
};
/**
 * JDM ルール文書の登録・バージョン管理（リリース時点 で proto 追加予定）
 *
 * @generated from service k1s0.tier1.decision.v1.DecisionAdminService
 */
export declare const DecisionAdminService: {
    readonly typeName: "k1s0.tier1.decision.v1.DecisionAdminService";
    readonly methods: {
        /**
         * JDM 文書の登録（schema validator と非決定要素 linter を通過必須）
         *
         * @generated from rpc k1s0.tier1.decision.v1.DecisionAdminService.RegisterRule
         */
        readonly registerRule: {
            readonly name: "RegisterRule";
            readonly I: typeof RegisterRuleRequest;
            readonly O: typeof RegisterRuleResponse;
            readonly kind: MethodKind.Unary;
        };
        /**
         * バージョン一覧
         *
         * @generated from rpc k1s0.tier1.decision.v1.DecisionAdminService.ListVersions
         */
        readonly listVersions: {
            readonly name: "ListVersions";
            readonly I: typeof ListVersionsRequest;
            readonly O: typeof ListVersionsResponse;
            readonly kind: MethodKind.Unary;
        };
        /**
         * 特定バージョンの取得（レビュー用）
         *
         * @generated from rpc k1s0.tier1.decision.v1.DecisionAdminService.GetRule
         */
        readonly getRule: {
            readonly name: "GetRule";
            readonly I: typeof GetRuleRequest;
            readonly O: typeof GetRuleResponse;
            readonly kind: MethodKind.Unary;
        };
    };
};
//# sourceMappingURL=decision_service_connect.d.ts.map