// 本ファイルは tier1 ヘルスチェック用の補助 proto。
// 12 API 公開リスト（state / pubsub / serviceinvoke / secrets / binding /
// workflow / log / telemetry / decision / audit / feature / pii）には **含まれない**
// プロセス生存確認 / k8s probe 用 RPC で、Liveness / Readiness の 2 RPC のみ。
//
// 12 API 本体は plan 03-02 で `src/contracts/tier1/k1s0/tier1/<api>/v1/` 配下に展開する。
// 本ファイルは buf generate パイプラインの動作確認も兼ねる（plan 03-01 完了条件）。
//
// 設計:
//   docs/03_要件定義/20_機能要件/02_機能一覧.md（12 API の正典）
//   docs/02_構想設計/adr/ADR-TIER1-002-protobuf-grpc.md
//   plan/03_Contracts実装/02_tier1_proto定義.md
import { Message, proto3, protoInt64 } from "@bufbuild/protobuf";
/**
 * Liveness probe のリクエスト（現状フィールドなし、将来拡張用）
 *
 * @generated from message k1s0.tier1.health.v1.LivenessRequest
 */
export class LivenessRequest extends Message {
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.health.v1.LivenessRequest";
    static fields = proto3.util.newFieldList(() => []);
    static fromBinary(bytes, options) {
        return new LivenessRequest().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new LivenessRequest().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new LivenessRequest().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(LivenessRequest, a, b);
    }
}
/**
 * Readiness probe のリクエスト（現状フィールドなし、将来拡張用）
 *
 * @generated from message k1s0.tier1.health.v1.ReadinessRequest
 */
export class ReadinessRequest extends Message {
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.health.v1.ReadinessRequest";
    static fields = proto3.util.newFieldList(() => []);
    static fromBinary(bytes, options) {
        return new ReadinessRequest().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new ReadinessRequest().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new ReadinessRequest().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(ReadinessRequest, a, b);
    }
}
/**
 * Liveness response: process の生存と version 情報を返す。
 *
 * @generated from message k1s0.tier1.health.v1.LivenessResponse
 */
export class LivenessResponse extends Message {
    /**
     * tier1 facade のビルドバージョン（SemVer）
     *
     * @generated from field: string version = 1;
     */
    version = "";
    /**
     * 起動時刻からの経過時間（秒）
     *
     * @generated from field: int64 uptime_seconds = 2;
     */
    uptimeSeconds = protoInt64.zero;
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.health.v1.LivenessResponse";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "version", kind: "scalar", T: 9 /* ScalarType.STRING */ },
        { no: 2, name: "uptime_seconds", kind: "scalar", T: 3 /* ScalarType.INT64 */ },
    ]);
    static fromBinary(bytes, options) {
        return new LivenessResponse().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new LivenessResponse().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new LivenessResponse().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(LivenessResponse, a, b);
    }
}
/**
 * Readiness response: 各依存の状態を個別に返す。
 *
 * @generated from message k1s0.tier1.health.v1.ReadinessResponse
 */
export class ReadinessResponse extends Message {
    /**
     * 全体としての ready 判定（依存すべて OK なら true）
     *
     * @generated from field: bool ready = 1;
     */
    ready = false;
    /**
     * 各依存（postgres / kafka / openbao / keycloak / 等）の個別状態
     *
     * @generated from field: map<string, k1s0.tier1.health.v1.DependencyStatus> dependencies = 2;
     */
    dependencies = {};
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.health.v1.ReadinessResponse";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "ready", kind: "scalar", T: 8 /* ScalarType.BOOL */ },
        { no: 2, name: "dependencies", kind: "map", K: 9 /* ScalarType.STRING */, V: { kind: "message", T: DependencyStatus } },
    ]);
    static fromBinary(bytes, options) {
        return new ReadinessResponse().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new ReadinessResponse().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new ReadinessResponse().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(ReadinessResponse, a, b);
    }
}
/**
 * 個別依存の状態
 *
 * @generated from message k1s0.tier1.health.v1.DependencyStatus
 */
export class DependencyStatus extends Message {
    /**
     * 接続可能か
     *
     * @generated from field: bool reachable = 1;
     */
    reachable = false;
    /**
     * 直近のエラー（reachable=false の時のみ意味を持つ）
     *
     * @generated from field: string error_message = 2;
     */
    errorMessage = "";
    constructor(data) {
        super();
        proto3.util.initPartial(data, this);
    }
    static runtime = proto3;
    static typeName = "k1s0.tier1.health.v1.DependencyStatus";
    static fields = proto3.util.newFieldList(() => [
        { no: 1, name: "reachable", kind: "scalar", T: 8 /* ScalarType.BOOL */ },
        { no: 2, name: "error_message", kind: "scalar", T: 9 /* ScalarType.STRING */ },
    ]);
    static fromBinary(bytes, options) {
        return new DependencyStatus().fromBinary(bytes, options);
    }
    static fromJson(jsonValue, options) {
        return new DependencyStatus().fromJson(jsonValue, options);
    }
    static fromJsonString(jsonString, options) {
        return new DependencyStatus().fromJsonString(jsonString, options);
    }
    static equals(a, b) {
        return proto3.util.equals(DependencyStatus, a, b);
    }
}
//# sourceMappingURL=health_service_pb.js.map