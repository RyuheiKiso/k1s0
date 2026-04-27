# @k1s0/backstage-plugin-catalog

k1s0 拡張 catalog-info.yaml entity を Backstage Catalog に取り込む plugin（skeleton）。

## annotation 規約

`catalog-info.yaml` の `metadata.annotations` に以下の prefix で属性を付与すると、
本 plugin の Catalog Processor（採用組織が実装）が解釈する。

| annotation key | 値の例 | 用途 |
|---|---|---|
| `k1s0.io/tier` | `tier1` / `tier2` / `tier3` | tier 階層を Backstage で可視化 |
| `k1s0.io/component` | `tier1-state` / `portal-bff` | k1s0 内部コンポーネント識別 |
| `k1s0.io/lang` | `go` / `dotnet` / `rust` / `typescript` / `web` / `go-bff` | 実装言語 |
| `k1s0.io/env` | `dev` / `staging` / `prod` | 環境（同 component の環境別 instance を識別） |

## skeleton の使い方

採用組織は本 package を自分の Backstage `app/packages/app/` から import し、
`@backstage/core-plugin-api` の `createPlugin(...)` で実 plugin を構築する。

```ts
// app/packages/app/src/App.tsx
import { createPlugin } from "@backstage/core-plugin-api";
import { K1S0_CATALOG_PLUGIN_ID, K1S0_ANNOTATIONS } from "@k1s0/backstage-plugin-catalog";

const k1s0Plugin = createPlugin({ id: K1S0_CATALOG_PLUGIN_ID });
// catalog processor / route / hook を採用組織で配線
```

## 関連

- ADR-DEVEX-002（Backstage 採用根拠）
- 各 example の `catalog-info.yaml`
