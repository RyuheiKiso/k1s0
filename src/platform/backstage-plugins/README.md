# src/platform/backstage-plugins — Backstage 開発者ポータル plugin

k1s0 を Spotify 由来の Backstage 開発者ポータルに統合するための plugin 群。
Backstage の plugin SDK に依存するため、本リリース時点 は **skeleton のみ** で、
実装は採用組織の Backstage インスタンスのバージョン（4.x 系）に応じて確定する。

## 同梱 plugin

| ディレクトリ | 役割 |
|---|---|
| [`k1s0-catalog/`](k1s0-catalog/) | k1s0 の Component / API / System 定義を Backstage Catalog に取り込む（catalog-info.yaml の k1s0 拡張属性パーサ） |
| [`k1s0-scaffolder/`](k1s0-scaffolder/) | `k1s0-scaffold` CLI を Backstage Scaffolder の Custom Action として公開（Web UI から tier2/tier3 サービスを生成） |

## scope

- 本リリース時点: 各 plugin に `package.json` / `src/plugin.ts` / `src/index.ts` / `README.md` を配置。
  実 Backstage SDK 連携は採用組織が `@backstage/core-plugin-api` 等を導入して接続する想定。
- 採用初期: Backstage `app/packages/app/` 配下の `App.tsx` に plugin を import + 配線。
- 採用後の運用拡大時: SoftwareTemplate（YAML）の k1s0 カタログ整備、Catalog Processor の自動 onboard。

## なぜ skeleton なのか

Backstage は `@backstage/core-plugin-api` / `@backstage/plugin-catalog-react` 等の依存が
非常に重く（数十 npm パッケージ、Backstage バージョンへの強い依存）、k1s0 OSS リポジトリ側で
これらを bundle すると採用組織側の Backstage バージョンとのズレが発生する。

そのため:
- 本ディレクトリは **構造と拡張ポイントの設計** のみを提供する
- 実 plugin code は採用組織が Backstage バージョンに合わせて実装する
- 本リポジトリは Backstage 統合の **インタフェース契約**（Component name / annotation 命名）を担保する

## 関連設計

- ADR-DEVEX-002（開発者ポータル採用、Backstage 想定）
- ADR-DEVEX-004（Golden Path 採用）
- 各 example の `catalog-info.yaml`（Backstage Component 定義の正典）
