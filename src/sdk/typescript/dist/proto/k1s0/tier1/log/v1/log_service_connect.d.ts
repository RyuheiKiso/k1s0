import { BulkSendLogRequest, BulkSendLogResponse, SendLogRequest, SendLogResponse } from "./log_service_pb.js";
import { MethodKind } from "@bufbuild/protobuf";
/**
 * Log API。本 API は OTel Logs パイプラインに直接乗せる（Loki / Grafana で参照）。
 *
 * @generated from service k1s0.tier1.log.v1.LogService
 */
export declare const LogService: {
    readonly typeName: "k1s0.tier1.log.v1.LogService";
    readonly methods: {
        /**
         * 単一エントリ送信
         *
         * @generated from rpc k1s0.tier1.log.v1.LogService.Send
         */
        readonly send: {
            readonly name: "Send";
            readonly I: typeof SendLogRequest;
            readonly O: typeof SendLogResponse;
            readonly kind: MethodKind.Unary;
        };
        /**
         * 一括送信（accepted / rejected で集計を返す）
         *
         * @generated from rpc k1s0.tier1.log.v1.LogService.BulkSend
         */
        readonly bulkSend: {
            readonly name: "BulkSend";
            readonly I: typeof BulkSendLogRequest;
            readonly O: typeof BulkSendLogResponse;
            readonly kind: MethodKind.Unary;
        };
    };
};
//# sourceMappingURL=log_service_connect.d.ts.map