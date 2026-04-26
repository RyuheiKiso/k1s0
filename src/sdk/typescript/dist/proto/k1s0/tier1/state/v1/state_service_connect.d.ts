import { BulkGetRequest, BulkGetResponse, DeleteRequest, DeleteResponse, GetRequest, GetResponse, SetRequest, SetResponse, TransactRequest, TransactResponse } from "./state_service_pb.js";
import { MethodKind } from "@bufbuild/protobuf";
/**
 * 状態管理 API。Store 名で valkey / postgres / minio 等のバックエンドを切り替える。
 *
 * @generated from service k1s0.tier1.state.v1.StateService
 */
export declare const StateService: {
    readonly typeName: "k1s0.tier1.state.v1.StateService";
    readonly methods: {
        /**
         * キー単位の取得（未存在時は not_found=true、エラーには非該当）
         *
         * @generated from rpc k1s0.tier1.state.v1.StateService.Get
         */
        readonly get: {
            readonly name: "Get";
            readonly I: typeof GetRequest;
            readonly O: typeof GetResponse;
            readonly kind: MethodKind.Unary;
        };
        /**
         * キー単位の保存（ETag 不一致時は FAILED_PRECONDITION でエラー）
         *
         * @generated from rpc k1s0.tier1.state.v1.StateService.Set
         */
        readonly set: {
            readonly name: "Set";
            readonly I: typeof SetRequest;
            readonly O: typeof SetResponse;
            readonly kind: MethodKind.Unary;
        };
        /**
         * キー単位の削除（ETag を expected_etag に渡せば楽観的排他で削除）
         *
         * @generated from rpc k1s0.tier1.state.v1.StateService.Delete
         */
        readonly delete: {
            readonly name: "Delete";
            readonly I: typeof DeleteRequest;
            readonly O: typeof DeleteResponse;
            readonly kind: MethodKind.Unary;
        };
        /**
         * 複数キーの一括取得（部分的な未存在は not_found=true で表現、エラーにしない）
         *
         * @generated from rpc k1s0.tier1.state.v1.StateService.BulkGet
         */
        readonly bulkGet: {
            readonly name: "BulkGet";
            readonly I: typeof BulkGetRequest;
            readonly O: typeof BulkGetResponse;
            readonly kind: MethodKind.Unary;
        };
        /**
         * トランザクション境界付きの複数操作（全 Store で対応するわけではない）
         *
         * @generated from rpc k1s0.tier1.state.v1.StateService.Transact
         */
        readonly transact: {
            readonly name: "Transact";
            readonly I: typeof TransactRequest;
            readonly O: typeof TransactResponse;
            readonly kind: MethodKind.Unary;
        };
    };
};
//# sourceMappingURL=state_service_connect.d.ts.map