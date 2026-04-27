<div align="center">

# k1s0

**個人開発のクラウドネイティブ・アプリケーションプラットフォーム**

_OSS 積み上げで、レガシーと共存できる、ベンダーに縛られない開発基盤_

[![License](https://img.shields.io/badge/license-Apache_2.0-blue.svg)](LICENSE)
[![Rust Edition](https://img.shields.io/badge/Rust-Edition_2024-dea584.svg)](src/CLAUDE.md)
[![Go SDK](https://img.shields.io/badge/Go-Dapr_facade-00ADD8.svg)](src/tier1/go)
[![Status](https://img.shields.io/badge/status-pre--v0-orange.svg)](docs/SHIP_STATUS.md)
[![Docs](https://img.shields.io/badge/docs-658_files_/_53k_lines-success.svg)](docs/INDEX.md)
[![ADR](https://img.shields.io/badge/ADR-36_records-informational.svg)](docs/02_構想設計/adr/README.md)

[**Why k1s0**](#-why-k1s0) ·
[**Architecture**](#-architecture) ·
[**Quick Start**](#-quick-start) ·
[**Docs**](docs/INDEX.md) ·
[**SHIP STATUS**](docs/SHIP_STATUS.md)

</div>

---

## ✦ What is k1s0

k1s0（ケイワンエスゼロ）は、**Kubernetes / Istio Ambient Mesh / Dapr / Backstage** といった CNCF エコシステムの OSS を、一枚岩のアプリケーションプラットフォームに統合した個人開発の基盤です。Web 系テック企業ではなく、**日々レガシー資産と向き合いながら新しいシステムを作らされている、ごく普通の業務システム開発の現場** を一次ターゲットに据えて設計されています。

業務システムが構造的に抱える 3 つの痛み — **レガシー .NET Framework 資産の肥大化・横断的関心事のコピペ実装・商用基盤のベンダーロックイン** — は、しばしば別個に語られますが、現場では絡み合って 1 本のロープのように開発組織の身動きを奪っています。k1s0 はこれを **1 本の OSS スタックで同時に解きほぐす** ことを目標に設計されました。

> **「段階導入の作法を捨て、設計の純度を最初から貫く」** — これは個人開発だからこそ取れるアプローチです。期日も納期もなく、納得するまで作り込んでから世に出すという立場が、商用 IDP では取れない設計判断を可能にします。

特徴を一行ずつまとめると次の通りです。

- **OSS のみで構成**（本体 Apache 2.0）。商用ライセンスを前提とした構成要素は採用していない
- **オンプレ / VM 第一級**。クラウド前提ではないため、社内データセンター・閉域網・規制業界でも完結する
- **レガシーを排除しない**。.NET Framework アプリは tier3 の拡張ポイントとして共存できる
- **言語の自由を尊重**。tier2 / tier3 では C# / Go / .NET MAUI / React を自由に選べる
- **基盤実装は Rust + Go**。長期保守性を最優先し、tier1 自作領域には Rust（Edition 2024）を採用

詳細: [`docs/01_企画/`](docs/01_企画/)

---

## ✦ The Problem — なぜ k1s0 が必要か

業務システムの開発現場では、しばしば次のような **負のフィードバックループ** が観測されます。

> 古い技術で作りこむ → 新技術への移行が困難 → 新技術を提案しても「よくわからないからダメ」 → 学習する若手が報われない → 古参が古い技術にしがみつく → さらに新技術への移行が困難に……（ループ）

このループの結果、単一技術ごとのモノレポの集合は時間とともに **「崩れかけのジェンガ」** 状態に陥ります。新しい一手を打つたびに別の場所が崩れる、という状態です。k1s0 が真正面から扱おうとする 4 つの痛みは次の通りです。

| 痛み | 現場で起きていること |
|---|---|
| レガシー .NET Framework 資産が動き続けている | 捨てるに捨てられず、新規開発の足を引っ張る |
| 横断的関心事（認証 / ログ / 監視 / state / pub-sub）のコピペ実装 | 業務ロジックに集中できず、各サービスで微妙に挙動がブレる |
| 端末への手動アプリインストール | PC リプレース時に情シスが大規模な対応に追われ、退職者の権限剥奪も遅延する |
| 商用基盤の高額ライセンス / ベンダーロックイン | 撤退コストが膨大で、技術選定が「政治」になる |

これらは個別の技術選定では解けません。**「アプリケーションプラットフォーム」全体として一貫した設計**が必要であり、それが k1s0 を単一の OSS プラットフォームとして公開する理由です。

詳細: [`docs/01_企画/01_背景と目的/`](docs/01_企画/01_背景と目的/)

---

## ✦ Design Principles — 5 つの設計特性

k1s0 の設計は、上記の課題を解くための明確な **設計方針** に貫かれています。

> **「OSS 積み上げで、レガシーと共存できる、ベンダーに縛られない開発プラットフォーム」**

| # | 特性 | 意味するところ |
|---|------|----------------|
| 1 | **無償で始められる** | Apache 2.0 OSS 前提。商用ライセンス費用を 1 円も発生させない構成 |
| 2 | **レガシーと共存** | .NET Framework アプリを tier3 で取り込み、段階移行ではなく **併存** を許す |
| 3 | **オンプレ / VM で完結** | クラウド非依存。社内データセンター・閉域網・規制業界でも導入可能 |
| 4 | **言語を尊重** | tier2 / tier3 は言語自由（C# / Go / MAUI / React 等）。プラットフォームが言語を強制しない |
| 5 | **採用ハードルを下げる** | 雛形 CLI + リファレンス実装 + Backstage Software Templates + 詳細ドキュメントで初手の壁を排除 |

これらは「あれば便利な機能」ではなく、**設計判断の上位制約**として機能します。技術選定や ADR 起票の際は、まずこの 5 特性に照らして決定が行われます。

---

## ✦ Why k1s0 — 競合と比べた立ち位置

商用 IDP（Internal Developer Platform）や k8s 商用ディストリビューションは、Web 系テック企業 / 大規模 SaaS を一次ターゲットに置いており、レガシー資産・オンプレ・言語自由度の優先度は低いままに設計されていることが多くあります。k1s0 はその **逆** を取ります。

| 軸 | 商用 IDP / k8s 商用 | **k1s0** |
|---|---|---|
| 対象ユーザー層 | Web 系テック企業 / 大規模 SaaS | **採用側組織の情シス / レガシー共存が必要な現場** |
| レガシー共存 | 通常考慮されない | **第一級で扱う**（tier3 の拡張ポイントとして併存） |
| 実行環境 | クラウドマネージド前提 | **オンプレ / VM 第一級** |
| 言語の自由度 | プラットフォーム指定 | **既存資産の言語を尊重**（C# / Go / MAUI / React） |
| エンドユーザー配信 | Intune 等の SaaS | **オンプレ完結ポータル**（PWA / MSIX） |
| 基盤言語 | Go / TS / Java 中心 | **Rust** で長期保守性最優先 |
| ライセンス | 商用（年額数百万〜数千万） | **Apache 2.0 OSS** |

「商用 IDP の機能セットを劣化コピーしたもの」ではなく、**意図的に異なる戦線に立つことで、商用が手薄にしている領域を埋める** という位置づけです。

詳細: [`docs/01_企画/02_競合と差別化/`](docs/01_企画/02_競合と差別化/)

---

## ✦ Architecture

k1s0 は **4 層の単方向依存** で構成されます。**逆向きの参照は CI で機械的に遮断**されます。

<div align="center">

<img src="docs/02_構想設計/01_アーキテクチャ/img/レイヤ構成.svg" alt="k1s0 レイヤ構成" width="780">

</div>

**依存ルール**: `tier3 → tier2 → (sdk ← contracts) → tier1 → infra` の一方向。`infra` への直接依存は **tier1 のみ**に許可されます。

### tier1 の核心 — Dapr 隠蔽

tier1 は **Dapr を完全に隠蔽する Go ファサード** として振る舞います。tier2 / tier3 から見えるのは `k1s0.Log` / `k1s0.State` / `k1s0.PubSub` / `k1s0.Audit` といった **12 の公開 API** のみです。Dapr 自体は強力ですが、tier2 / tier3 で直接使うと **Dapr に縛られて** バージョン更新・破壊的変更・将来の差し替え時に影響が広範に及びます。tier1 がファサード層を担うことで、その影響範囲を tier1 内部に閉じ込めます。

```csharp
// tier2/tier3 のコード — Dapr を一切意識しない
await k1s0.Log.Info("注文を受領", new { orderId });
await k1s0.State.SaveAsync("orders", orderId, order);
await k1s0.PubSub.PublishAsync("order-events", "created", order);
await k1s0.Audit.RecordAsync("ORDER_CREATED", userId, orderId);
```

### tier1 のもう一つの核心 — バラつきを防ぐ多層防御

> **「機械で遮断できる逸脱はツールで、残余は人のレビューで」**

12 の公開 API があっても、それを意図通りに使ってもらえなければ意味がありません。k1s0 は **6 段の捕捉構造** で「設計思想を外した使い方」をブロックします。完全自動化ではなく **継続的な捕捉率向上** を狙う構造で、動的言語のリフレクション経由 SDK 呼び出しのような CI 単独で検出しきれない逸脱まで対象に入れます。

| 段 | 施策 | 対象 |
|---|------|------|
| ① | **雛形生成 CLI** | ゼロから書かせない |
| ② | **Opinionated API** | やり方を 1 通りに絞る |
| ③ | **CI ガード** | 静的言語の禁止 import を機械検出 |
| ④ | **リファレンス実装** | 模範サービスを 1 本提供 |
| ⑤ | **PR チェックリスト** | 動的言語・設計思想逸脱を人のレビューで補完 |
| ⑥ | **内製 analyzer** | 頻出逸脱を吸い上げて成長する |

### 2 つのポータル

k1s0 は対象ユーザーが異なる **2 つのポータル** を併設します。両者は競合しません。

| 軸 | アプリ配信ポータル（k1s0 自製） | Backstage（OSS） |
|---|---|---|
| 対象 | **業務担当 / エンドユーザー** | **開発者 / 運用者 / SRE** |
| 主目的 | 業務アプリの利用開始 | サービスの発見・把握・運用 |
| 表示内容 | 業務アプリ一覧 / 説明 / レビュー | サービスカタログ / 依存関係 / API |
| 端末設定コピー | あり（PC リプレース対応） | なし |

詳細: [`docs/02_構想設計/01_アーキテクチャ/`](docs/02_構想設計/01_アーキテクチャ/) · [tier1 設計](docs/02_構想設計/02_tier1設計/) · [CICD と配信](docs/02_構想設計/04_CICDと配信/)

---

## ✦ Tech Stack

> **方針**: ベンダーロックインを回避するため、CNCF / 主要 OSS を意図的に組み合わせる。商用代替は採用しない。

<table>
<tr><td>

**Orchestration & Mesh**
- Kubernetes
- Istio (**Ambient Mesh**)
- Envoy Gateway

**Data & Messaging**
- Apache Kafka (Strimzi)
- CloudNativePG (PostgreSQL)
- Longhorn / MinIO
- Valkey

</td><td>

**Observability (LGTMP)**
- OpenTelemetry
- Grafana Tempo
- Pyroscope / Loki / Prometheus

**Building Blocks**
- Dapr (CNCF Graduated)
- Temporal (long-running workflows)
- ZEN Engine (BRE)

</td><td>

**Security & Supply Chain**
- Keycloak (OIDC)
- OpenBao (secret / PKI)
- SPIRE (workload identity)
- Cosign / SLSA L3 / Kyverno

**Developer Experience**
- Backstage (portal)
- OpenTofu (IaC)
- Renovate · Argo CD · Argo Rollouts

</td></tr>
</table>

技術選定の根拠: [`docs/02_構想設計/03_技術選定/`](docs/02_構想設計/03_技術選定/)

---

## ✦ Quick Start

### 推奨: Dev Container

`tools/devcontainer/profiles/` に役割別の toolchain プロファイルが入っており、Rust / Go / .NET / Node / buf / kubectl / dapr CLI / drawio CLI などが自動で揃います。

```bash
# 役割別 sparse-checkout を有効化（10 役: tier1-rust-dev / tier1-go-dev / ...）
./tools/sparse/checkout-role.sh tier1-go-dev
```

### ローカル開発

```bash
make doctor          # toolchain 診断（role 自動検出）
make codegen         # contracts (*.proto) → 4 言語 SDK 生成
make lint            # proto lint + pre-commit
make verify-quick    # origin/main からの差分のみ検査（高速イテレーション用）
make verify          # CI と同等の全検査（push 前の最終ゲート）
```

利用可能な target は `make help` で一覧できます。

---

## ✦ Repository Layout

```text
k1s0/
├── src/                    # 全ソースコード（一次配置）
│   ├── contracts/          # *.proto — single source of truth
│   ├── tier1/              # Go (Dapr facade) + Rust (core)
│   ├── sdk/                # contracts から自動生成（4 言語）
│   ├── tier2/              # 業務ドメインロジック
│   ├── tier3/              # Web / Native / BFF / Legacy wrap
│   └── platform/           # scaffold CLI / dependency analyzer
├── infra/                  # クラスタ構成（k8s / mesh / observability）
├── deploy/                 # GitOps 配信（Argo CD / Helm / Rollouts）
├── ops/                    # 運用 Runbook / SRE 資材
├── tools/                  # 横断ツール（codegen / sparse / ci / devcontainer）
├── tests/                  # tier 横断テスト
├── docs/                   # 設計ドキュメント（658 ファイル / 53k 行）
├── examples/               # tier1 ファサード等のサンプル
└── third_party/            # 外部 OSS の取り込み
```

依存方向: `tier3 → tier2 → (sdk ← contracts) → tier1 → infra`（[`src/README.md`](src/README.md)）

---

## ✦ Documentation

ドキュメントは規模が大きい（**658 ファイル / 53,000 行超**）ため、階層索引から辿ることを推奨します。

| 入口 | 用途 |
|---|---|
| [`docs/INDEX.md`](docs/INDEX.md) | 全体索引 / ID 体系 / 探索動線 |
| [`docs/01_企画/`](docs/01_企画/) | 採用検討者向けの企画資料（背景・競合・ロードマップ） |
| [`docs/02_構想設計/`](docs/02_構想設計/) | アーキテクチャ・tier1 設計・技術選定 |
| [`docs/02_構想設計/adr/`](docs/02_構想設計/adr/) | ADR 36 件（アーキテクチャ決定記録） |
| [`docs/03_要件定義/`](docs/03_要件定義/) | IPA 共通フレーム 2013 準拠の要件定義書 |
| [`docs/04_概要設計/`](docs/04_概要設計/) | IPA 準拠の概要設計書（`DS-*` 体系） |
| [`docs/05_実装/`](docs/05_実装/) | 実装段階設計（`IMP-*` 接頭辞） |
| [`docs/40_運用ライフサイクル/`](docs/40_運用ライフサイクル/) | Runbook 集（5 段構成: 検出 / 初動 / 復旧 / 原因調査 / 事後処理） |

---

## ✦ Maturity Disclosure

> docs（設計）が記述する全体像と、`git clone` 直後にビルド・起動できる範囲には**意図的なギャップ**があります。

採用検討者は **必ず** [`docs/SHIP_STATUS.md`](docs/SHIP_STATUS.md) を参照してください。各領域の実装率を **同梱済 / 雛形あり / 設計のみ** の 3 ランクで開示しています。

| ランク | 意味 |
|---|---|
| **同梱済** | 実コードが存在し、ビルド・起動・テストが走る |
| **雛形あり** | 主要ファイル・README は存在するが、ロジックは最小骨格 |
| **設計のみ** | docs に詳細設計があるが、実装側ディレクトリは空 |

---

## ✦ Non-Functional Targets

| 分類 | 目標値 |
|---|---|
| tier1 API レイテンシ (p99) | 全 API < **500 ms** |
| 可用性 (業務時間帯) | **99 %** |
| バックアップ RPO | PostgreSQL: 数秒 / etcd: 24 時間 |
| バックアップ RTO | PostgreSQL: 15 分 / クラスタ全壊: 4 時間 |
| CVE 対応 (Critical) | **48 時間以内**（Renovate 自動 PR） |

詳細: [`docs/02_構想設計/01_アーキテクチャ/04_非機能とデータ/01_非機能要件.md`](docs/02_構想設計/01_アーキテクチャ/04_非機能とデータ/01_非機能要件.md)

---

## ✦ Project Posture

k1s0 は OSS ですが、運用の前提を**正直に開示**します。

- **バス係数 = 1**: 起案者が継続的に維持。会社利用者は自身でフォーク・内製化する余地を確保する責任を持つ
- **SLO / SLA は提供しない**: OSS としての品質保証はない。業務利用する側に運用責任がある
- **dogfooding で品質を担保**: 起案者が日常利用し、会社が業務利用することで継続的にフィードバックが回る構造
- **段階導入はしない**: 設計の純度を保つため、最初から本来あるべき形で実装する

---

## ✦ Contributing

PR / Issue / 議論を歓迎します。最初のコミットを送る前に以下を一読してください。

- [`CONTRIBUTING.md`](CONTRIBUTING.md) — 開発フロー / コミット規約 / レビュー観点
- [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md) — 行動規範
- [`SECURITY.md`](SECURITY.md) — 脆弱性報告手順（CVE 48h SLA）
- [`GOVERNANCE.md`](GOVERNANCE.md) — 意思決定プロセス
- [`src/CLAUDE.md`](src/CLAUDE.md) — コーディング規約（日本語コメント必須 / 1 ファイル 500 行制限 等）

大きな変更（新 API / 新 tier / アーキテクチャ変更）は **ADR** を起票してから PR をお願いします（[`/adr` slash command](.claude/commands/adr.md)）。

---

## ✦ License

[Apache License 2.0](LICENSE)

- 商用利用・改変・再配布が自由
- 明示的特許許諾条項あり
- ZEN Engine (MIT) と整合
- `infra/` で利用する AGPL 依存 OSS（Grafana / Loki / MinIO 等）は本体ライセンスに伝播しない構成

法務サマリ: [`docs/01_企画/05_法務サマリ/`](docs/01_企画/05_法務サマリ/)

---

<div align="center">

**k1s0** — _Built by one, designed for many._

[Docs](docs/INDEX.md) · [SHIP STATUS](docs/SHIP_STATUS.md) · [ADR](docs/02_構想設計/adr/README.md) · [Issues](https://github.com/RyuheiKiso/k1s0/issues)

</div>
