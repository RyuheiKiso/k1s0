// 本ファイルは k1s0 TypeScript SDK の Client 型と接続管理。
//
// 利用例:
//   const client = new K1s0Client({
//     baseUrl: "https://tier1.k1s0.example.com",
//     tenantId: "tenant-A",
//     subject: "svc-foo",
//   });
//   const result = await client.state.get("valkey-default", "user/123");

import { createPromiseClient, type Transport } from "@connectrpc/connect";
import { createGrpcWebTransport } from "@connectrpc/connect-web";
import { StateService } from "./proto/k1s0/tier1/state/v1/state_service_connect.js";
import { PubSubService } from "./proto/k1s0/tier1/pubsub/v1/pubsub_service_connect.js";
import { SecretsService } from "./proto/k1s0/tier1/secrets/v1/secrets_service_connect.js";
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

// K1s0Config は Client 初期化時に渡す設定。
export interface K1s0Config {
  // gRPC-Web / Connect transport の base URL（Istio Ambient Gateway 経由）。
  baseUrl: string;
  // テナント ID（全 RPC の TenantContext.tenant_id に自動付与）。
  tenantId: string;
  // 主体識別子（subject）。
  subject: string;
  // 任意で外部 transport を注入可能（テスト用 mock 等）。
  transport?: Transport;
}

// K1s0Client は 12 service へのアクセス起点。
export class K1s0Client {
  // Connect transport（HTTP/1.1 ベースの gRPC-Web 互換）。
  readonly transport: Transport;
  // 自動付与する TenantContext 情報。
  readonly config: K1s0Config;
  // 動詞統一 facade（12 service すべて）。
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

  // Config から Client を生成する。transport が省略されたら gRPC-Web を使う。
  constructor(config: K1s0Config) {
    // baseUrl から transport を構築する（外部注入があればそれを優先）。
    this.transport =
      config.transport ?? createGrpcWebTransport({ baseUrl: config.baseUrl });
    // Config を保持する。
    this.config = config;
    // 各 facade を初期化する。
    this.state = new StateFacade(this);
    this.pubsub = new PubSubFacade(this);
    this.secrets = new SecretsFacade(this);
    this.log = new LogFacade(this);
    this.workflow = new WorkflowFacade(this);
    this.decision = new DecisionFacade(this);
    this.audit = new AuditFacade(this);
    this.pii = new PiiFacade(this);
    this.feature = new FeatureFacade(this);
    this.binding = new BindingFacade(this);
    this.invoke = new InvokeFacade(this);
    this.telemetry = new TelemetryFacade(this);
  }

  // 内部用: TenantContext proto を生成する。
  tenantContext(): TenantContext {
    // 構造体リテラル（new を使う connect-es v1 慣用）。
    return new TenantContext({
      tenantId: this.config.tenantId,
      subject: this.config.subject,
      // correlation_id は OTel interceptor 後段付与。
      correlationId: "",
    });
  }

  // 内部用: 各 facade が自前で生成 client を作るための helper。
  // 残り 9 service の raw アクセスは利用者がここから createPromiseClient で構築する。
  rawState() {
    return createPromiseClient(StateService, this.transport);
  }
  rawPubSub() {
    return createPromiseClient(PubSubService, this.transport);
  }
  rawSecrets() {
    return createPromiseClient(SecretsService, this.transport);
  }
}
