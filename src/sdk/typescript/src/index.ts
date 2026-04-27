// 本ファイルは k1s0 TypeScript SDK のエントリポイント。
// 動詞統一 facade（k1s0.state.save 等）を提供する。
//
// docs 正典:
//   docs/05_実装/10_ビルド設計/30_TypeScript_pnpm_workspace/01_TypeScript_pnpm_workspace.md
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/
//
// scope（リリース時点 最小、3 代表 service）:
//   - K1s0Client.state.{get|save|delete}
//   - K1s0Client.pubsub.publish
//   - K1s0Client.secrets.{get|rotate}
//   - その他 9 service は raw 経由で利用可能（Connect transport を返す）

// 共通型 / 12 service facade を再 export
export * as Common from "./proto/k1s0/tier1/common/v1/common_pb.js";
export { K1s0Client, type K1s0Config } from "./client.js";
export { StateFacade } from "./state.js";
export { PubSubFacade } from "./pubsub.js";
export { SecretsFacade } from "./secrets.js";
export { LogFacade } from "./log.js";
export { WorkflowFacade } from "./workflow.js";
export { DecisionFacade } from "./decision.js";
export { AuditFacade } from "./audit.js";
export { PiiFacade } from "./pii.js";
export { FeatureFacade } from "./feature.js";
export { BindingFacade } from "./binding.js";
export { InvokeFacade } from "./invoke.js";
export { TelemetryFacade } from "./telemetry.js";
