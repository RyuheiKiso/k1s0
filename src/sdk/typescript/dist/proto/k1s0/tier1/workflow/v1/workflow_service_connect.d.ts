import { CancelRequest, CancelResponse, GetStatusRequest, GetStatusResponse, QueryRequest, QueryResponse, SignalRequest, SignalResponse, StartRequest, StartResponse, TerminateRequest, TerminateResponse } from "./workflow_service_pb.js";
import { MethodKind } from "@bufbuild/protobuf";
/**
 * Workflow API。tier2 がワークフロー種別をコード登録し、tier1 経由で実行操作する。
 *
 * @generated from service k1s0.tier1.workflow.v1.WorkflowService
 */
export declare const WorkflowService: {
    readonly typeName: "k1s0.tier1.workflow.v1.WorkflowService";
    readonly methods: {
        /**
         * ワークフロー開始
         *
         * @generated from rpc k1s0.tier1.workflow.v1.WorkflowService.Start
         */
        readonly start: {
            readonly name: "Start";
            readonly I: typeof StartRequest;
            readonly O: typeof StartResponse;
            readonly kind: MethodKind.Unary;
        };
        /**
         * シグナル送信（ワークフローへの入力イベント）
         *
         * @generated from rpc k1s0.tier1.workflow.v1.WorkflowService.Signal
         */
        readonly signal: {
            readonly name: "Signal";
            readonly I: typeof SignalRequest;
            readonly O: typeof SignalResponse;
            readonly kind: MethodKind.Unary;
        };
        /**
         * クエリ（ワークフロー状態の読取り、副作用なし）
         *
         * @generated from rpc k1s0.tier1.workflow.v1.WorkflowService.Query
         */
        readonly query: {
            readonly name: "Query";
            readonly I: typeof QueryRequest;
            readonly O: typeof QueryResponse;
            readonly kind: MethodKind.Unary;
        };
        /**
         * 正常終了の依頼（キャンセル）
         *
         * @generated from rpc k1s0.tier1.workflow.v1.WorkflowService.Cancel
         */
        readonly cancel: {
            readonly name: "Cancel";
            readonly I: typeof CancelRequest;
            readonly O: typeof CancelResponse;
            readonly kind: MethodKind.Unary;
        };
        /**
         * 強制終了
         *
         * @generated from rpc k1s0.tier1.workflow.v1.WorkflowService.Terminate
         */
        readonly terminate: {
            readonly name: "Terminate";
            readonly I: typeof TerminateRequest;
            readonly O: typeof TerminateResponse;
            readonly kind: MethodKind.Unary;
        };
        /**
         * 状態取得
         *
         * @generated from rpc k1s0.tier1.workflow.v1.WorkflowService.GetStatus
         */
        readonly getStatus: {
            readonly name: "GetStatus";
            readonly I: typeof GetStatusRequest;
            readonly O: typeof GetStatusResponse;
            readonly kind: MethodKind.Unary;
        };
    };
};
//# sourceMappingURL=workflow_service_connect.d.ts.map