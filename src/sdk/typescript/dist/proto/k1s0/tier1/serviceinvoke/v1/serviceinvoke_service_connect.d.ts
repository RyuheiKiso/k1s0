import { InvokeChunk, InvokeRequest, InvokeResponse } from "./serviceinvoke_service_pb.js";
import { MethodKind } from "@bufbuild/protobuf";
/**
 * サービス間呼出を仲介する。tier1 が allowlist / RBAC / 監査を一括強制する。
 *
 * @generated from service k1s0.tier1.serviceinvoke.v1.InvokeService
 */
export declare const InvokeService: {
    readonly typeName: "k1s0.tier1.serviceinvoke.v1.InvokeService";
    readonly methods: {
        /**
         * 任意サービスの任意メソッドを呼び出す（app_id は Dapr の app_id 概念と互換）
         *
         * @generated from rpc k1s0.tier1.serviceinvoke.v1.InvokeService.Invoke
         */
        readonly invoke: {
            readonly name: "Invoke";
            readonly I: typeof InvokeRequest;
            readonly O: typeof InvokeResponse;
            readonly kind: MethodKind.Unary;
        };
        /**
         * ストリーミング呼出（大容量応答や段階出力のため、サーバ → クライアントの単方向ストリーム）
         * server-streaming のため HTTP/JSON gateway 経由非対応（gRPC で直接呼出す運用）。
         *
         * @generated from rpc k1s0.tier1.serviceinvoke.v1.InvokeService.InvokeStream
         */
        readonly invokeStream: {
            readonly name: "InvokeStream";
            readonly I: typeof InvokeRequest;
            readonly O: typeof InvokeChunk;
            readonly kind: MethodKind.ServerStreaming;
        };
    };
};
//# sourceMappingURL=serviceinvoke_service_connect.d.ts.map