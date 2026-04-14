# ADR-0005: Dapr の採用と tier2/tier3 への隠蔽

- ステータス: Accepted
- 起票日: 2026-04-14
- 決定日: 2026-04-14
- 起票者: kiso ryuhei
- 関係者: インフラチーム / 起案者 / 決裁者

## コンテキスト

tier1 は state / pub-sub / secrets / workflow / bindings 等の共通基盤機能を tier2 / tier3 に提供する責務を負う。
これらの機能群を **どう実現するか** について、大きく 3 つの選択肢があった。

1. すべて自前実装 (Rust 一本)
2. すべて Dapr に依存
3. **Dapr を基本としつつ、足りない箇所を tier1 で自前補完**

加えて、Dapr を採用する場合の **tier2 / tier3 への露出** をどうするかが論点となる。
Dapr はサービスメッシュの考え方が強く、Sidecar や HTTP/gRPC API が利用者に直接見える設計だが、k1s0 ではこれを隠蔽する方針を企画段階で示している。

## 決定

**Dapr を基本採用し、不足箇所は tier1 で補完する** ハイブリッド戦略を採る。
さらに **tier2 / tier3 からは Dapr の存在を完全に隠蔽** する。利用者が触るのは tier1 が公開する k1s0 API (`k1s0.Log` / `k1s0.State` / `k1s0.PubSub` / `k1s0.Workflow` / `k1s0.Secrets` / `k1s0.Decision` / `k1s0.Feature` 等) のみとする。

- **Dapr 直接利用**: tier1 ファサード (Go) のみ。Dapr Go SDK 経由で building blocks を呼び出す。
- **自作領域**: Dapr に依存しない。Rust で実装する ([ADR-0002](./ADR-0002-tier1-language-hybrid.md))。
- **tier2 / tier3**: 雛形 CLI が出力する k1s0 クライアントライブラリのみを利用。Dapr Sidecar の存在も含めて意識しない。

## 検討した選択肢

### 選択肢 A: 全自前実装

- 概要: state / pub-sub / secrets / workflow をすべて tier1 で実装する。
- メリット: 外部依存ゼロ。完全に内製で制御可能。
- デメリット: 開発・保守コストが極めて大きい。CNCF エコシステムの建付けから外れ、長期的な技術負債になる。

### 選択肢 B: 全 Dapr 依存

- 概要: 自作領域も含めて Dapr の枠組みに乗せる。
- メリット: 統一感がある。
- デメリット: JTC 固有要件 (例: ZEN Engine による業務ルール宣言化、レガシー .NET 共存) で Dapr の標準機能では対応できない領域が残る。

### 選択肢 C: Dapr 未採用 (標準化を放棄)

- 概要: 各サービスが state / pub-sub を独自実装する。
- メリット: 各サービスの自由度が最大。
- デメリット: 標準化を放棄することになり、tier1 を設ける意義そのものが失われる。

### 選択肢 D: Dapr 採用 + tier1 補完 + tier2/3 隠蔽 (採用)

- 概要: Dapr building blocks を基本として活用し、不足箇所は tier1 で補完。tier2/3 には Dapr を見せない。
- メリット: CNCF 標準を取り込みつつ、JTC 固有要件にも対応可能。将来 Dapr 自体を別実装に差し替えても tier2/3 への影響が最小。
- デメリット: tier1 の責務が大きくなる (ファサード / 自前補完の両方を持つ)。

## 決定理由

- **building blocks の機能優位性**: Dapr の Service Invocation / State / PubSub / Secrets / Workflow / Bindings は、自前実装した場合に必要な工数を大幅に削減する。
- **言語非依存性**: tier2 / tier3 の実装言語選択を Dapr が阻害しない。これは k1s0 の中長期的な拡張性に直結する。
- **Workflow の置き換え可能性**: Phase 2 で Workflow バックエンドを Dapr Workflow から **Temporal** に差し替える計画があるが、tier1 の `k1s0.Workflow` API を介していれば tier2/3 の利用コードは変更不要となる。隠蔽方針の重要性を裏付ける具体例。
- **Dapr の方向転換リスクの吸収**: 雛形 CLI が tier2/3 のクライアントコードを生成し、k1s0 API を利用する形に閉じ込めることで、Dapr の互換性破壊を tier1 内で吸収できる。
- **Dapr Rust SDK Alpha 問題の回避**: ファサードを Go で実装する ([ADR-0002](./ADR-0002-tier1-language-hybrid.md)) ことで、Rust SDK の不安定性に起因するリスクをゼロにする。

## 影響

### ポジティブな影響

- CNCF エコシステムの building blocks を取り込みながら、JTC 固有要件にも対応できる。
- tier2 / tier3 の利用者は Dapr の概念を学習する必要がない。`k1s0.State.Get(...)` のような直感的な API のみを扱える。
- Dapr の方向転換 / Workflow バックエンド差し替え (Temporal 等) を tier2/3 に影響させずに実施できる。

### ネガティブな影響 / リスク

- tier1 の責務が大きくなり、tier1 自体の安定性・可観測性が極めて重要となる。
  - 緩和策: tier1 API のヘルスチェック / メトリクスを Phase 1b から Prometheus + Grafana で監視する。
- Dapr Control Plane / Sidecar の運用コストが追加で発生する。
  - 緩和策: MVP-1a の段階で Dapr Control Plane の運用手順を Runbook 化する。
- tier1 ファサードがボトルネックになりうる。
  - 緩和策: Phase 2 で `k1s0.Feature` (OpenFeature/flagd) によるカナリア制御、`k1s0` レート制限を導入し、ボトルネックの可視化と切り離しを行う。

### 移行・対応事項

- tier1 ファサード (Go) と Dapr Sidecar を組み合わせた最小構成を Phase 1a で構築する (`k1s0.Log` のみ)。
- Phase 1b で `k1s0.Telemetry` を追加。他の building block API はスタブとして雛形のみ用意する。
- Phase 2 で `k1s0.State` / `k1s0.PubSub` / `k1s0.Workflow` / `k1s0.Secrets` / `k1s0.Decision` / `k1s0.Feature` を順次有効化する。
- `k1s0.Workflow` の Temporal 切替えポリシー (短期=Dapr Workflow / 長期=Temporal) は別途 ADR で起票する。
- 雛形 CLI が出力する tier2 / tier3 クライアントコードは k1s0 API のみを参照し、Dapr SDK には直接依存しないことをガイドラインで明記する。

## 参考資料

- [`../../01_企画/03_tier1設計/01_Dapr隠蔽方針.md`](../../01_企画/03_tier1設計/01_Dapr隠蔽方針.md) — 隠蔽方針の詳細
- [`../../01_企画/03_tier1設計/02_内部言語ハイブリッド.md`](../../01_企画/03_tier1設計/02_内部言語ハイブリッド.md) — Go / Rust ハイブリッドの根拠
- [`../../01_企画/03_tier1設計/05_ワークフロー振り分け基準.md`](../../01_企画/03_tier1設計/05_ワークフロー振り分け基準.md) — Dapr Workflow / Temporal 振り分け
- [ADR-0002](./ADR-0002-tier1-language-hybrid.md) — tier1 内部言語ハイブリッド
