import { QueryAuditRequest, QueryAuditResponse, RecordAuditRequest, RecordAuditResponse } from "./audit_service_pb.js";
import { MethodKind } from "@bufbuild/protobuf";
/**
 * Audit API。WORM ストア（Postgres + immutable view）に追記専用で記録する。
 *
 * @generated from service k1s0.tier1.audit.v1.AuditService
 */
export declare const AuditService: {
    readonly typeName: "k1s0.tier1.audit.v1.AuditService";
    readonly methods: {
        /**
         * 監査イベント記録（成功時は audit_id を返す、改竄不可）
         *
         * @generated from rpc k1s0.tier1.audit.v1.AuditService.Record
         */
        readonly record: {
            readonly name: "Record";
            readonly I: typeof RecordAuditRequest;
            readonly O: typeof RecordAuditResponse;
            readonly kind: MethodKind.Unary;
        };
        /**
         * 監査イベント検索（範囲 + フィルタ、出力には PII Mask が自動適用）
         *
         * @generated from rpc k1s0.tier1.audit.v1.AuditService.Query
         */
        readonly query: {
            readonly name: "Query";
            readonly I: typeof QueryAuditRequest;
            readonly O: typeof QueryAuditResponse;
            readonly kind: MethodKind.Unary;
        };
    };
};
//# sourceMappingURL=audit_service_connect.d.ts.map