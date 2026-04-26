import { LivenessRequest, LivenessResponse, ReadinessRequest, ReadinessResponse } from "./health_service_pb.js";
import { MethodKind } from "@bufbuild/protobuf";
/**
 * ヘルスチェック用の最小 service。Kubernetes liveness/readiness probe からの
 * gRPC ヘルスチェックと、tier2/tier3 からの疎通確認に使う。
 *
 * @generated from service k1s0.tier1.health.v1.HealthService
 */
export declare const HealthService: {
    readonly typeName: "k1s0.tier1.health.v1.HealthService";
    readonly methods: {
        /**
         * Liveness probe: process が応答可能なら OK。依存 backend は見ない。
         *
         * @generated from rpc k1s0.tier1.health.v1.HealthService.Liveness
         */
        readonly liveness: {
            readonly name: "Liveness";
            readonly I: typeof LivenessRequest;
            readonly O: typeof LivenessResponse;
            readonly kind: MethodKind.Unary;
        };
        /**
         * Readiness probe: 依存 backend（Postgres / Kafka / OpenBao 等）が到達可能
         * かどうかも含めて判定する。詳細仕様は plan 04-16。
         *
         * @generated from rpc k1s0.tier1.health.v1.HealthService.Readiness
         */
        readonly readiness: {
            readonly name: "Readiness";
            readonly I: typeof ReadinessRequest;
            readonly O: typeof ReadinessResponse;
            readonly kind: MethodKind.Unary;
        };
    };
};
//# sourceMappingURL=health_service_connect.d.ts.map