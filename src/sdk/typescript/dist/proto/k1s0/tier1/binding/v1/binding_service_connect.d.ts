import { InvokeBindingRequest, InvokeBindingResponse } from "./binding_service_pb.js";
import { MethodKind } from "@bufbuild/protobuf";
/**
 * Binding API。バインディング名は運用側で事前設定（s3-archive / smtp-notify 等）。
 *
 * @generated from service k1s0.tier1.binding.v1.BindingService
 */
export declare const BindingService: {
    readonly typeName: "k1s0.tier1.binding.v1.BindingService";
    readonly methods: {
        /**
         * 出力バインディング呼出（tier1 → 外部システムへ送信）
         *
         * @generated from rpc k1s0.tier1.binding.v1.BindingService.Invoke
         */
        readonly invoke: {
            readonly name: "Invoke";
            readonly I: typeof InvokeBindingRequest;
            readonly O: typeof InvokeBindingResponse;
            readonly kind: MethodKind.Unary;
        };
    };
};
//# sourceMappingURL=binding_service_connect.d.ts.map