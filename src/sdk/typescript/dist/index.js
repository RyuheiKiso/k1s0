// 本ファイルは k1s0 TypeScript SDK のエントリポイント。
// proto 生成物（src/proto/ 配下）を再 export する薄いラッパ。
//
// docs 正典:
//   docs/05_実装/10_ビルド設計/30_TypeScript_pnpm_workspace/01_TypeScript_pnpm_workspace.md
//   docs/05_実装/20_コード生成設計/10_buf_Protobuf/01_buf_Protobuf生成パイプライン.md
//
// 利用例:
//   import { StateServiceClient } from "@k1s0/sdk-rpc/proto/k1s0/tier1/state/v1/state_service_connect.js";
//   import * as common from "@k1s0/sdk-rpc/proto/k1s0/tier1/common/v1/common_pb.js";
//
// 高水準ファサード（k1s0.State.save 等の動詞統一）はロードマップ #8 で追加予定。
// 共通型（TenantContext / ErrorDetail / K1s0ErrorCategory）の再 export
export * as Common from "./proto/k1s0/tier1/common/v1/common_pb.js";
//# sourceMappingURL=index.js.map