import { type Transport } from "@connectrpc/connect";
import { TenantContext } from "./proto/k1s0/tier1/common/v1/common_pb.js";
import { StateFacade } from "./state.js";
import { PubSubFacade } from "./pubsub.js";
import { SecretsFacade } from "./secrets.js";
import { LogFacade } from "./log.js";
import { WorkflowFacade } from "./workflow.js";
import { DecisionFacade } from "./decision.js";
import { AuditFacade } from "./audit.js";
import { PiiFacade } from "./pii.js";
import { FeatureFacade } from "./feature.js";
import { BindingFacade } from "./binding.js";
import { InvokeFacade } from "./invoke.js";
import { TelemetryFacade } from "./telemetry.js";
export interface K1s0Config {
    baseUrl: string;
    tenantId: string;
    subject: string;
    transport?: Transport;
}
export declare class K1s0Client {
    readonly transport: Transport;
    readonly config: K1s0Config;
    readonly state: StateFacade;
    readonly pubsub: PubSubFacade;
    readonly secrets: SecretsFacade;
    readonly log: LogFacade;
    readonly workflow: WorkflowFacade;
    readonly decision: DecisionFacade;
    readonly audit: AuditFacade;
    readonly pii: PiiFacade;
    readonly feature: FeatureFacade;
    readonly binding: BindingFacade;
    readonly invoke: InvokeFacade;
    readonly telemetry: TelemetryFacade;
    constructor(config: K1s0Config);
    tenantContext(): TenantContext;
    rawState(): import("@connectrpc/connect").Client<{
        readonly typeName: "k1s0.tier1.state.v1.StateService";
        readonly methods: {
            readonly get: {
                readonly name: "Get";
                readonly I: typeof import("./proto/k1s0/tier1/state/v1/state_service_pb.js").GetRequest;
                readonly O: typeof import("./proto/k1s0/tier1/state/v1/state_service_pb.js").GetResponse;
                readonly kind: import("@bufbuild/protobuf").MethodKind.Unary;
            };
            readonly set: {
                readonly name: "Set";
                readonly I: typeof import("./proto/k1s0/tier1/state/v1/state_service_pb.js").SetRequest;
                readonly O: typeof import("./proto/k1s0/tier1/state/v1/state_service_pb.js").SetResponse;
                readonly kind: import("@bufbuild/protobuf").MethodKind.Unary;
            };
            readonly delete: {
                readonly name: "Delete";
                readonly I: typeof import("./proto/k1s0/tier1/state/v1/state_service_pb.js").DeleteRequest;
                readonly O: typeof import("./proto/k1s0/tier1/state/v1/state_service_pb.js").DeleteResponse;
                readonly kind: import("@bufbuild/protobuf").MethodKind.Unary;
            };
            readonly bulkGet: {
                readonly name: "BulkGet";
                readonly I: typeof import("./proto/k1s0/tier1/state/v1/state_service_pb.js").BulkGetRequest;
                readonly O: typeof import("./proto/k1s0/tier1/state/v1/state_service_pb.js").BulkGetResponse;
                readonly kind: import("@bufbuild/protobuf").MethodKind.Unary;
            };
            readonly transact: {
                readonly name: "Transact";
                readonly I: typeof import("./proto/k1s0/tier1/state/v1/state_service_pb.js").TransactRequest;
                readonly O: typeof import("./proto/k1s0/tier1/state/v1/state_service_pb.js").TransactResponse;
                readonly kind: import("@bufbuild/protobuf").MethodKind.Unary;
            };
        };
    }>;
    rawPubSub(): import("@connectrpc/connect").Client<{
        readonly typeName: "k1s0.tier1.pubsub.v1.PubSubService";
        readonly methods: {
            readonly publish: {
                readonly name: "Publish";
                readonly I: typeof import("./proto/k1s0/tier1/pubsub/v1/pubsub_service_pb.js").PublishRequest;
                readonly O: typeof import("./proto/k1s0/tier1/pubsub/v1/pubsub_service_pb.js").PublishResponse;
                readonly kind: import("@bufbuild/protobuf").MethodKind.Unary;
            };
            readonly bulkPublish: {
                readonly name: "BulkPublish";
                readonly I: typeof import("./proto/k1s0/tier1/pubsub/v1/pubsub_service_pb.js").BulkPublishRequest;
                readonly O: typeof import("./proto/k1s0/tier1/pubsub/v1/pubsub_service_pb.js").BulkPublishResponse;
                readonly kind: import("@bufbuild/protobuf").MethodKind.Unary;
            };
            readonly subscribe: {
                readonly name: "Subscribe";
                readonly I: typeof import("./proto/k1s0/tier1/pubsub/v1/pubsub_service_pb.js").SubscribeRequest;
                readonly O: typeof import("./proto/k1s0/tier1/pubsub/v1/pubsub_service_pb.js").Event;
                readonly kind: import("@bufbuild/protobuf").MethodKind.ServerStreaming;
            };
        };
    }>;
    rawSecrets(): import("@connectrpc/connect").Client<{
        readonly typeName: "k1s0.tier1.secrets.v1.SecretsService";
        readonly methods: {
            readonly get: {
                readonly name: "Get";
                readonly I: typeof import("./proto/k1s0/tier1/secrets/v1/secrets_service_pb.js").GetSecretRequest;
                readonly O: typeof import("./proto/k1s0/tier1/secrets/v1/secrets_service_pb.js").GetSecretResponse;
                readonly kind: import("@bufbuild/protobuf").MethodKind.Unary;
            };
            readonly bulkGet: {
                readonly name: "BulkGet";
                readonly I: typeof import("./proto/k1s0/tier1/secrets/v1/secrets_service_pb.js").BulkGetSecretRequest;
                readonly O: typeof import("./proto/k1s0/tier1/secrets/v1/secrets_service_pb.js").BulkGetSecretResponse;
                readonly kind: import("@bufbuild/protobuf").MethodKind.Unary;
            };
            readonly rotate: {
                readonly name: "Rotate";
                readonly I: typeof import("./proto/k1s0/tier1/secrets/v1/secrets_service_pb.js").RotateSecretRequest;
                readonly O: typeof import("./proto/k1s0/tier1/secrets/v1/secrets_service_pb.js").RotateSecretResponse;
                readonly kind: import("@bufbuild/protobuf").MethodKind.Unary;
            };
        };
    }>;
}
//# sourceMappingURL=client.d.ts.map