import { ExportAuditChunk, ExportAuditRequest, QueryAuditRequest, QueryAuditResponse, RecordAuditRequest, RecordAuditResponse, VerifyChainRequest, VerifyChainResponse } from "./audit_service_pb.js";
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
        /**
         * ハッシュチェーン整合性検証（FR-T1-AUDIT-002）。
         * テナント配下の全イベントの prev_hash / event_hash の連鎖を検証する。
         * 改ざん検知時は valid=false で先頭の不整合 sequence_number と reason を返す。
         *
         * @generated from rpc k1s0.tier1.audit.v1.AuditService.VerifyChain
         */
        readonly verifyChain: {
            readonly name: "VerifyChain";
            readonly I: typeof VerifyChainRequest;
            readonly O: typeof VerifyChainResponse;
            readonly kind: MethodKind.Unary;
        };
        /**
         * 監査ログのテナント単位エクスポート（FR-T1-AUDIT-002 疑似 IF "Audit.Export"）。
         * server-streaming で範囲内の events を batch（chunk）に分けて配信し、
         * 大量レコードでもメモリを圧迫しない。format で CSV / JSON / NDJSON を選択する。
         *
         * @generated from rpc k1s0.tier1.audit.v1.AuditService.Export
         */
        readonly export: {
            readonly name: "Export";
            readonly I: typeof ExportAuditRequest;
            readonly O: typeof ExportAuditChunk;
            readonly kind: MethodKind.ServerStreaming;
        };
    };
};
//# sourceMappingURL=audit_service_connect.d.ts.map