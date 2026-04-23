# 10. ビルド設計

本章は k1s0 モノレポの各言語ネイティブビルド（Cargo / go build / pnpm / dotnet）をどう組み合わせ、どこまで選択的に実行するかを実装フェーズ確定版として固定する。`00_ディレクトリ設計/00_設計方針/02_世界トップ企業事例比較.md` で確定した「Bazel / Buck2 を採用しない」判断（ADR-TIER1-001 のハイブリッド言語構成下では学習コストを正当化できない）を前提に、各言語のネイティブビルド機構で選択ビルド・キャッシュ・ワークスペース境界をどう切るかを規定する。

## 本章の位置付け

Google や Meta は Bazel / Buck2 で選択ビルドを実現しているが、k1s0 は 2 名運用から始まるため OSS スタックを増やさない方針を取る。代わりに Cargo workspace の `[workspace.dependencies]`、Go の複数 `go.mod` 分離、pnpm workspace の `--filter`、dotnet の `sln` 分割をそれぞれの言語で最適運用する。ADR-TIER1-001（Go + Rust ハイブリッド）と ADR-TIER1-003（内部言語不可視）により tier1 内部は 2 言語並行ビルドとなり、契約は ADR-DIR-001 の `src/contracts/` 集約を経由して 4 言語 SDK に展開される。本章はこれら 4 機構の境界設定・切り替え基準・将来の Bazel 移行経路を明示する。

Phase 1c 時点で Rust / Go / TypeScript のいずれかで単言語ビルド時間が 30 分を超えた場合、当該言語のみ Bazel 導入を再評価する（新 ADR 起票）。本章はその判定を可能にする計測点の設置も含む。

![ビルド設計概観: Cargo 2 分割 + Go module 5 分割 + buf 連動 + path-filter](img/10_ビルド設計概観.svg)

## Phase 確定範囲

- Phase 0: Cargo workspace・go.mod 分離・pnpm workspace・dotnet sln 境界の確定、選択ビルド判定の path-filter 運用、ローカルキャッシュ戦略
- Phase 1a: CI リモートキャッシュ（GitHub Actions cache / sccache）、並列化設定
- Phase 1c: ビルド時間計測結果にもとづく Bazel 導入可否判定（新 ADR 起票）

## RACI

| 役割 | 責務 |
|---|---|
| Platform/Build（主担当 / A） | Cargo / go.mod / pnpm / dotnet の境界決定、キャッシュ設定、path-filter ルール |
| SRE（共担当 / B） | ビルド時間 SLI の計測と公開、リモートキャッシュ稼働率 |
| DX（共担当 / C） | 開発者ローカルのビルド時間と IDE 応答性の計測 |
| Security（共担当 / D） | ビルド成果物の署名連携点（`80_サプライチェーン設計/` と結合） |

## 節構成予定

```
10_ビルド設計/
├── README.md
├── 00_方針/                # Bazel 不採用の根拠と 4 言語ネイティブ運用
├── 10_Rust_Cargo_workspace/
├── 20_Go_module分離戦略/
├── 30_TypeScript_pnpm_workspace/
├── 40_dotnet_sln境界/
├── 50_選択ビルド判定/      # path-filter / changed files
├── 60_キャッシュ戦略/      # ローカル / CI / リモート
└── 90_対応IMP-BUILD索引/
```

## IMP ID 予約

本章で採番する実装 ID は `IMP-BUILD-*`（予約範囲: IMP-BUILD-001 〜 IMP-BUILD-099）。

## 対応 ADR / 概要設計 ID / NFR

- ADR: [ADR-TIER1-001](../../02_構想設計/adr/ADR-TIER1-001-go-rust-hybrid.md)（Go + Rust ハイブリッド）/ [ADR-TIER1-002](../../02_構想設計/adr/ADR-TIER1-002-protobuf-grpc.md)（Protobuf gRPC）/ [ADR-TIER1-003](../../02_構想設計/adr/ADR-TIER1-003-language-opacity.md)（内部言語不可視）/ [ADR-DIR-001](../../02_構想設計/adr/ADR-DIR-001-contracts-elevation.md)（contracts 昇格）
- DS-SW-COMP: DS-SW-COMP-003（Dapr/Rust 二分）/ 120 / 121 / 122 / 129 / 130（tier1 Rust / Go / SDK / 配置）
- NFR: NFR-B-PERF-001（p99 < 500ms に至る性能基盤）/ NFR-C-NOP-004（ビルド所要時間運用）/ NFR-C-MGMT-001（設定 Git 管理）

## 関連章

- `20_コード生成設計/` — 生成物がビルド入力となるため前段
- `30_CI_CD設計/` — ビルドの CI 実行面
- `95_DXメトリクス/` — ビルド時間 SLI の公開先
