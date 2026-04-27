// 本ファイルは k1s0-catalog plugin のエントリ。
// Backstage 4.x の `createPlugin` で plugin を生成し、Catalog Processor として
// k1s0 拡張 annotation（k1s0.io/tier / k1s0.io/component / 等）を解釈する。
//
// 本リリース時点 では実 Backstage SDK 統合は placeholder。採用組織が
// `@backstage/core-plugin-api` 等を依存追加して `createPlugin(...)` で接続する。

// Backstage 統合時の plugin メタ情報（将来 createPlugin に渡す）。
export const K1S0_CATALOG_PLUGIN_ID = "k1s0-catalog";
// plugin バージョン（k1s0 リポジトリの semver と独立、Backstage の互換性は peerDependencies で示す）。
export const K1S0_CATALOG_PLUGIN_VERSION = "0.1.0";

// k1s0 拡張 annotation の prefix 規約。
// catalog-info.yaml の metadata.annotations にこの prefix で属性を付与すると
// 本 plugin の Catalog Processor が解釈する。
export const K1S0_ANNOTATION_PREFIX = "k1s0.io/";

// k1s0 拡張属性のキー一覧（tier / component / lang / env）。
export const K1S0_ANNOTATIONS = {
  // tier 階層（tier1 / tier2 / tier3）
  Tier: "k1s0.io/tier",
  // コンポーネント識別子（tier1-state / portal-bff 等）
  Component: "k1s0.io/component",
  // 実装言語（go / dotnet / rust / typescript / web 等）
  Lang: "k1s0.io/lang",
  // 環境（dev / staging / prod、運用上の identification）
  Env: "k1s0.io/env",
} as const;

// 採用組織が createPlugin で plugin を構築するためのスタブ。
// 本関数は Backstage SDK 依存が解決された時点で実装される。
//
// 想定 signature（Backstage 4.x の createPlugin 仕様）:
//   import { createPlugin } from '@backstage/core-plugin-api';
//   export const k1s0CatalogPlugin = createPlugin({
//     id: K1S0_CATALOG_PLUGIN_ID,
//     // ... routes / apis / hooks
//   });
export function getPluginManifest(): {
  id: string;
  version: string;
  annotationPrefix: string;
} {
  // 採用組織が plugin metadata として参照可能な情報のみ返却。
  return {
    id: K1S0_CATALOG_PLUGIN_ID,
    version: K1S0_CATALOG_PLUGIN_VERSION,
    annotationPrefix: K1S0_ANNOTATION_PREFIX,
  };
}
