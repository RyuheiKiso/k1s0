<div align="center">

# k1s0

**Dapr を tier1 に閉じ込め、Istio Ambient と共存させる polyglot プラットフォーム基盤。**

<sub>オンプレ k8s / Go + Rust ハイブリッド tier1 / contracts-first な 4 言語 SDK / ZEN Engine による decision-as-data / スパースモノレポ運用。</sub>

<br>

![Rust 2024](https://img.shields.io/badge/Rust-Edition_2024-CE422B?style=flat-square&logo=rust&logoColor=white)
![Go 1.22+](https://img.shields.io/badge/Go-1.22%2B-00ADD8?style=flat-square&logo=go&logoColor=white)
![Kubernetes](https://img.shields.io/badge/Kubernetes-kubeadm-326CE5?style=flat-square&logo=kubernetes&logoColor=white)
![Istio Ambient](https://img.shields.io/badge/Istio-Ambient_Mesh-466BB0?style=flat-square&logo=istio&logoColor=white)
![Dapr](https://img.shields.io/badge/Dapr-Facade_only-0D2192?style=flat-square&logo=dapr&logoColor=white)
![gRPC](https://img.shields.io/badge/gRPC-Protobuf-244C5A?style=flat-square&logo=grpc&logoColor=white)
![License](https://img.shields.io/badge/license-Apache_2.0-blue?style=flat-square)

<br>

[Docs](docs/) ·
[ADR](docs/02_構想設計/adr/) ·
[Contracts](docs/02_構想設計/02_tier1設計/) ·
[License](#license)
 
</div>

---

## TL;DR

- **Dapr をアプリに露出させない。** tier2 / tier3 が叩くのは `k1s0.Log.Info(...)` であって `DaprClient` ではない。ファサードは stable Dapr Go SDK を直接使う Go サービス群、暗号と決定表は Rust コア。Dapr の差し替えはアプリのコードに波及しない。
- **Ambient Mesh で二重サイドカーを回避。** Dapr はサイドカー、Istio はサイドカーレス（`ztunnel` が L4 HBONE mTLS、`waypoint` が L7）。mTLS スタックも設定面も二重化しない。
- **レガシーを殺さない。** .NET Framework モノリスはサイドカー方式 / API Gateway 方式の 2 パターンで相乗り。OpenFeature + flagd で段階的に切り替える。
- **SDK は生成物。** `src/contracts/*.proto` → `buf generate` → .NET / Go / Rust / TypeScript の 4 SDK。手書きゼロ。Pact + OpenAPI で契約テストをマージゲート化。
- **Decision-as-data.** 承認フロー・権限・ルーティングは ZEN Engine（Rust / MIT）が評価する JSON の決定表。`if/else` の塔ではなく `decisions/` への PR で変える。
- **Sparse monorepo.** 10 役別 cone 定義で、ほとんどの開発者はリポジトリの 10〜30% しか materialise しない。

## アーキテクチャ

k1s0 は `infra` → `tier1` → `tier2` → `tier3` の 4 レイヤ構成をとる。依存方向は一方向のみで、逆向きは CI 上の analyzer（`src/platform/analyzer/`）と PR レビューの二重で遮断する。infra に直接依存できるのは tier1 に限る。

<div align="center">

<img src="docs/02_構想設計/01_アーキテクチャ/img/レイヤ構成.svg" alt="k1s0 レイヤ構成" width="780">

</div>

| Layer | 責務 | 実装 | app から可視? |
|---|---|---|---|
| `infra/` | k8s / mesh / messaging / observability / secrets | kubeadm, Istio Ambient, Envoy Gateway, Strimzi Kafka, CloudNativePG, MinIO, OpenBao, LGTMP | — |
| `tier1/` | 11 公開 API（Log / State / PubSub / Secret / Workflow / Audit / Decision / Settings / ...） | Go（Dapr ファサード）+ Rust（ZEN Engine / 暗号 / 雛形 CLI）+ `.proto` 契約 | SDK / gRPC のみ可視、内部言語は不可視 |
| `sdk/` | contracts から生成される多言語クライアント | `K1s0.Sdk`（NuGet）/ `github.com/k1s0/sdk` / `k1s0-sdk` crate / `@k1s0/sdk` | yes |
| `tier2/` | ドメイン共通業務ロジック | C# / Go（自由選択） | — |
| `tier3/` | Web（React + TS + pnpm）/ Native（.NET MAUI）/ BFF（Go）/ Legacy wrap（.NET Framework） | — | — |

詳細設計は [`docs/02_構想設計/01_アーキテクチャ/01_基礎/`](docs/02_構想設計/01_アーキテクチャ/01_基礎/) と [`docs/02_構想設計/02_tier1設計/`](docs/02_構想設計/02_tier1設計/)。

## tier1 の手触り

tier1 の存在意義は「アプリから Dapr を知らないで済ませる」こと。SDK の動詞は 4 言語で揃えてある。以下のコードはどの言語で書いても、裏は `SDK → tier1 gRPC → Dapr Go ファサード → Dapr runtime` という同じ経路を通る。Dapr を差し替えても下の 4 ファイルは変わらない。

```csharp
// tier2 / tier3（C#）— Dapr は見えない
await k1s0.Log.Info("order received", new { orderId });
await k1s0.State.SaveAsync("orders", orderId, order);
await k1s0.PubSub.PublishAsync("order-events", "created", order);
await k1s0.Audit.RecordAsync("ORDER_CREATED", userId, orderId);

var decision = await k1s0.Decision.EvaluateAsync("approval/purchase", new { amount, grade });
if (decision.Outcome == "auto_approve") { /* ... */ }
```

```go
// Go — 同じ動詞、同じ形
k1s0.Log.Info(ctx, "order received", log.F("orderId", orderId))
if err := k1s0.State.Save(ctx, "orders", orderId, order); err != nil { /* ... */ }
_ = k1s0.PubSub.Publish(ctx, "order-events", "created", order)
```

```rust
// Rust — async、型は prost-gen から
k1s0::log::info!("order received", order_id = %order_id);
k1s0::state::save("orders", &order_id, &order).await?;
k1s0::audit::record(AuditKind::OrderCreated, &user_id, &order_id).await?;
```

```typescript
// TypeScript — ESM / CJS 両対応、Promise で返る
await k1s0.log.info("order received", { orderId });
await k1s0.state.save("orders", orderId, order);
await k1s0.pubsub.publish("order-events", "created", order);
```

## 技術的見どころ

### Dapr ファサード（not passthrough）

Dapr は強力だが粘着質で、tier2 / tier3 に `DaprClient` を撒いた瞬間、メジャーバージョン更新や方針変更がサービス全体に浸食する。k1s0 は Dapr を **tier1 の内部依存** として扱い、公開契約（`src/contracts/tier1/v1/*.proto`）に Dapr 型を一切出さない。内製 analyzer が `src/tier1/go/` の外での `dapr.*` import を reject する。[ADR-TIER1-001](docs/02_構想設計/adr/) 参照。

### Ambient Mesh + Dapr の共存

古典的 Istio サイドカー + Dapr サイドカー = pod に 2 枚、mTLS スタックも設定面も二重化。Ambient はメッシュを分離する — `ztunnel`（ノード常駐、L4 HBONE mTLS）と `waypoint`（namespace レベル、L7 認可 / HTTPRoute）。pod に残るサイドカーは Dapr だけになる。パケット経路は [`docs/02_構想設計/01_アーキテクチャ/img/ネットワーク層_Ambientパケット経路.svg`](docs/02_構想設計/01_アーキテクチャ/img/ネットワーク層_Ambientパケット経路.svg)。

### Go + Rust ハイブリッド tier1

- **Go ファサード** — stable Dapr Go SDK に寄生する層。request/response のボイラープレート、retry / circuit-breaker、OTel 伝搬。動きの速い Dapr に追従する責任を背負う。
- **Rust コア** — ZEN Engine 統合、Audit のハッシュチェーン、Transit 風の envelope encryption、雛形生成 CLI。Edition 2024、crate root に `#![forbid(unsafe_code)]`、panic は FFI 境界で `catch_unwind`。

### Contracts-first、SDK は生成物

`src/contracts/` が真実。`buf generate` で 4 SDK を吐き、`buf breaking` で契約の後方互換を破壊する PR をブロックする。契約テストは Pact（消費者駆動）と OpenAPI spec comparator の 2 段で、両方がマージゲート。手書き SDK は存在しない。

### Decision-as-data（ZEN Engine）

承認フロー・権限判定・ワークフロー振り分けは、すべて ZEN Engine の JSON 決定表。`decisions/` への PR が業務ルール変更の単位で、tier2 のデプロイは不要。決定表は Rust コアが評価し、結果は `k1s0.Decision.EvaluateAsync(...)` で返る。決定の入出力はすべて `k1s0.Audit` に記録されハッシュチェーンで改ざん検知される。

### Workflow 2 基盤を 1 API で

短期（〜数分、10 ステップ以下）は Dapr Workflow、長期（数時間〜数週間、人的承認含む）は Temporal。tier2 / tier3 からは `k1s0.Workflow.StartAsync(type, input)` のみ。振り分けは YAML 設定を Go ファサードが読んで自動選択する。基準は [`docs/02_構想設計/02_tier1設計/04_診断と補償/05_ワークフロー振り分け基準.md`](docs/02_構想設計/02_tier1設計/04_診断と補償/05_ワークフロー振り分け基準.md)。

### 観測性: LGTMP + Pyroscope + eBPF

OpenTelemetry → Tempo（trace）/ Loki（log）/ Prometheus + Mimir（metric）/ Pyroscope（continuous profiling）。tier1 API の p99 予算は **500 ms** で、内訳は「業務 200 + Dapr sidecar 80 + OTel 20 + ハッシュチェーン監査 50 + net/DB 150」の机上積算。リリース時点から Pyroscope + eBPF を同梱し、採用側の運用蓄積によって各コンポーネント実測に置き換えられる構成にしてある。

### 本物の依存で回すテスト

- **Integration**: Testcontainers で本物の Postgres / Kafka / Keycloak を起動して回す（Mock は禁止、境界は実体で確かめる）
- **Contract**: Pact（消費者駆動）+ OpenAPI comparator
- **Fuzz**: `cargo-fuzz`（Rust パーサ）/ `go-fuzz`（Go デコーダ）を nightly で回す
- **Chaos**: LitmusChaos で縮退動作を仕様化（ztunnel 落下 / Kafka broker 殺し / Dapr control-plane 再起動）
- **Golden snapshot**: 雛形 CLI の出力を `tests/golden/` に固定し、scaffold 壊しを CI で即検出

## リポジトリ構成

ADR-DIR-001/002/003 によりリリース時点で確定済み。ルートから tier1〜tier3、infra、ops、tools、tests、docs が並ぶフラット構成で、スパースチェックアウトでの役割別運用を前提に設計されている。

```
k1s0/
├── src/
│   ├── contracts/          # .proto — source of truth（tier1 公開 11 API + 内部 gRPC）
│   ├── tier1/go/           # Dapr ファサード（stable Dapr Go SDK）
│   ├── tier1/rust/         # ZEN Engine / crypto / 雛形 CLI
│   ├── sdk/{dotnet,go,rust,typescript}/   # contracts から生成
│   ├── tier2/              # C# / Go のドメイン共通
│   ├── tier3/{web,native,bff,legacy-wrap}/
│   └── platform/           # scaffold CLI / analyzer / Backstage plugins
├── infra/                  # k8s, mesh, dapr, data, security, obs, scaling, envs
├── deploy/                 # Argo CD Apps / Kustomize / Helm / Argo Rollouts / OpenTofu
├── ops/                    # Runbook / chaos / DR / oncall / load / scripts
├── tools/                  # devcontainer / local-stack(kind,k3d) / codegen / sparse
├── tests/                  # e2e(Go) / contract(Pact,OpenAPI) / integration / fuzz / golden
├── docs/                   # 企画 → 構想 → 要件 → 概要 → 実装（IPA 共通フレーム 2013）
├── examples/               # Golden Path example services
└── third_party/            # 社内 OSS フォーク vendoring（UPSTREAM.md / PATCHES.md 必須）
```

依存方向は `tier3 → tier2 → (sdk ← contracts) → tier1 → infra` の一方向に固定され、逆向きは analyzer で遮断される。

## Getting started

partial clone + sparse checkout を前提にしている。全部を checkout するとサイズが重いので、自分の役割に対応する cone を当てること。

```bash
# 1. partial + sparse で clone
git clone --filter=blob:none --sparse <url> k1s0 && cd k1s0
git config core.sparseCheckoutCone true

# 2. 役割を選ぶ
#    tier1-rust-dev | tier1-go-dev | tier2-dev | tier3-web-dev | tier3-native-dev
#    platform-cli-dev | sdk-dev | infra-ops | docs-writer | full
./tools/sparse/checkout-role.sh tier1-rust-dev

# 3. ローカルに infra を立てる（k3d + Dapr + Postgres + Kafka + Keycloak）
./tools/local-stack/up.sh

# 4. contracts を再生成して tier1 facade を起動
buf generate
cd src/tier1/go && go run ./cmd/k1s0d
```

VS Code で開く場合は、役割ごとの Dev Container プロファイル（`tools/devcontainer/profiles/`）を選ぶと、言語 toolchain / buf / kubectl / istioctl / dapr CLI / drawio CLI が揃う。docs-writer プロファイルがルート既定（軽量）。

## Contributing

- コードはすべて **日本語コメント**。ファイル冒頭に説明コメント、各ステートメントの 1 行上に日本語コメント（[`CLAUDE.md`](CLAUDE.md)）。
- **1 ファイル 500 行以内**（docs 例外）。超えたら分割。
- **契約優先**: 跨 tier の変更は `src/contracts/` → ファサード → 生成 SDK → tier2/3 の順に降りる。SDK への直接パッチは reject される。
- **ADR 必須**: 構造に触る変更は [`docs/02_構想設計/adr/`](docs/02_構想設計/adr/) に ADR を切る。採番は `ADR-<AREA>-NNN`。
- **drawio 規約**: アスキー図は禁止。重要概念は drawio → SVG export → md から参照。4 レイヤ記法（[`docs/00_format/drawio_layer_convention.md`](docs/00_format/drawio_layer_convention.md)）に従い、アプリ層=暖色 / ネットワーク層=寒色 / インフラ層=中性灰 / データ層=薄紫 で責務分離。
- **CI ゲート**: `buf lint` / `buf breaking` / `go vet` / `clippy -D warnings` / 内製 analyzer（禁止 import 検出）/ contract tests / fuzz corpus 回帰。

## リリース範囲

k1s0 は起案者が業務外の時間で開発した個人開発 OSS で、リリース時点で全機能を一気通貫に同梱した状態で GitHub に公開している。リリース後の段階分割追加は採らず、採用側組織が必要な範囲を選び取って導入できる構成にしてある。

| 領域 | 同梱内容 |
|---|---|
| infra | kubeadm / Istio Ambient / Envoy Gateway / Strimzi Kafka / CloudNativePG / MinIO / OpenBao / LGTMP |
| tier1 | 11 公開 API（Log / State / PubSub / Secret / Workflow / Audit / Decision / Settings / ...） |
| ファサード | Dapr Go ファサード（stable Dapr Go SDK）+ Rust コア（ZEN Engine / 暗号 / 雛形 CLI） |
| SDK | .NET / Go / Rust / TypeScript の 4 言語クライアント（contracts から自動生成） |
| 配信 | Backstage 開発者ポータル / Argo CD / Argo Rollouts / OpenTofu |
| 観測性 | OpenTelemetry → Tempo / Loki / Prometheus + Mimir / Pyroscope + eBPF |
| ワークフロー | Dapr Workflow（短期）+ Temporal（長期）の 2 基盤を 1 API に統合 |
| レガシー共存 | .NET Framework モノリスとサイドカー方式 / API Gateway 方式の 2 パターン共存 |
| ガバナンス | ZEN Engine による decision-as-data / `k1s0.Audit` ハッシュチェーン |
| マルチクラスタ | クラスタフェデレーション設計を tier1 / infra に内蔵 |

採用側組織での運用段階（採用初期 / 運用拡大時 / マルチクラスタ移行時 / 全社展開期）でどの構成要素を活用するかは [`docs/04_概要設計/55_運用ライフサイクル方式設計/`](docs/04_概要設計/55_運用ライフサイクル方式設計/) に整理してある。

## License

[Apache License 2.0](LICENSE)。GitHub に公開済みの撤回不可の OSS で、採用側組織は Apache 2.0 の条件下で改変・再配布・商用利用ができる。

依存 OSS のうち AGPL ライセンスのもの（MinIO ほか）は `infra/` 内部でのみ稼働し、ネットワーク越しに改変版をユーザーに提供しない構成のため FSF / Grafana Labs 公式 FAQ に照らして義務発動なしと判定済み。判定の根拠と再配布時の注意事項は [`docs/02_構想設計/05_法務とコンプライアンス/`](docs/02_構想設計/05_法務とコンプライアンス/) を参照。

---

<div align="center">

<sub>k1s0 — <i>keep it simple, zero (vendor lock-in)</i>.</sub>

</div>
