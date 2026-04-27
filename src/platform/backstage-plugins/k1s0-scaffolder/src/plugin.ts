// 本ファイルは k1s0-scaffolder plugin のエントリ。
// Backstage Scaffolder の Custom Action として `k1s0-scaffold` CLI を内部実行する。
//
// 本リリース時点 では実装は placeholder。採用組織が `@backstage/plugin-scaffolder-node`
// の `createTemplateAction` で実 Action を構築する。

// plugin メタ情報。
export const K1S0_SCAFFOLDER_PLUGIN_ID = "k1s0-scaffolder";
// plugin バージョン。
export const K1S0_SCAFFOLDER_PLUGIN_VERSION = "0.1.0";

// k1s0-scaffold が受け付ける ServiceType（src/platform/scaffold/ engine が走査する template.yaml metadata.name と整合）。
export type K1s0ScaffoldServiceType =
  | "tier2-go"
  | "tier2-dotnet"
  | "tier3-bff"
  | "tier3-web";

// Backstage SoftwareTemplate（YAML）から渡される input 仕様。
export interface K1s0ScaffolderInput {
  // 生成する ServiceType。
  serviceType: K1s0ScaffoldServiceType;
  // サービス名（kebab-case）。
  name: string;
  // .NET 名前空間（tier2-dotnet で必須）。
  namespace?: string;
  // オーナ識別子（GitHub org / user）。
  owner: string;
}

// 採用組織が `createTemplateAction` で構築するための input schema 定義。
// 本オブジェクトは JSON Schema 互換で、Backstage の Software Template の `parameters`
// セクションから参照される想定。
export const K1S0_SCAFFOLDER_INPUT_SCHEMA = {
  type: "object",
  required: ["serviceType", "name", "owner"],
  properties: {
    serviceType: {
      type: "string",
      enum: ["tier2-go", "tier2-dotnet", "tier3-bff", "tier3-web"],
      description: "k1s0-scaffold が受け付ける ServiceType",
    },
    name: {
      type: "string",
      pattern: "^[a-z][a-z0-9-]*[a-z0-9]$",
      description: "サービス名（kebab-case 推奨）",
    },
    namespace: {
      type: "string",
      pattern: "^[A-Z][A-Za-z0-9]*$",
      description: ".NET 名前空間（tier2-dotnet で必須、PascalCase）",
    },
    owner: {
      type: "string",
      description: "オーナ識別子（GitHub org / user）",
    },
  },
} as const;

// plugin メタ情報を返却するスタブ。
export function getPluginManifest(): {
  id: string;
  version: string;
  inputSchema: typeof K1S0_SCAFFOLDER_INPUT_SCHEMA;
} {
  return {
    id: K1S0_SCAFFOLDER_PLUGIN_ID,
    version: K1S0_SCAFFOLDER_PLUGIN_VERSION,
    inputSchema: K1S0_SCAFFOLDER_INPUT_SCHEMA,
  };
}
