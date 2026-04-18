# OSS 長期戦略

## 目的

k1s0 は OSS を基盤とするプラットフォームである。OSS には有償化・メンテナンス停止・破壊的変更といったリスクが常に伴う。本ドキュメントは、これらのリスクに対する長期的な対処方針を定める。

[`00_選定方針.md`](00_選定方針.md) の「5. OSS リスク管理方針」がリスクの封じ込め構造を定めるのに対し、本ドキュメントはリスクが顕在化した際の出口戦略を定める。

---

## 1. 基本方針

OSS のリスクが顕在化した場合の出口は「自作への置き換え」だけではない。OSS の性質に応じて、以下の 3 つの戦略を使い分ける。

| 戦略 | 内容 | 適用基準 |
|---|---|---|
| A. 永続利用 | OSS をそのまま使い続ける | コミュニティが巨大で商用化リスクが極めて低く、自作が非現実的な場合 |
| B. 代替 OSS への差し替え | 同等機能を持つ別の OSS に乗り換える | tier1 封じ込めにより差し替え可能で、代替候補が存在する場合 |
| C. 自作への置き換え | Rust で自作実装に段階的に移行する | k1s0 固有のドメインロジックであり、チーム規模で保守可能な場合 |

「安定稼働後に OSS を自作に置き換える」という方針を一律に適用してはならない。各 OSS がどの戦略に該当するかを個別に判定し、本ドキュメントで管理する。

---

## 2. 戦略を分ける理由

### 2.1 自作のコストは OSS の成熟度に比例する

Kubernetes や PostgreSQL のような OSS は、数万人年規模のコミュニティ投資の産物である。これを数人のチームで自作することは不可能であり、試みること自体がプロジェクトの存続を危うくする。

### 2.2 自作はバスファクターを悪化させる

k1s0 の最大リスクはバスファクターの低さである（[`../../../01_企画/03_ロードマップと体制/`](../../../01_企画/03_ロードマップと体制/) 参照）。OSS はコミュニティがメンテナンスを担うためバスファクターが実質的に無限大だが、自作に置き換えるとそのコンポーネントのバスファクターは 1〜2 に低下する。OSS を自作に置き換えるほど、プロジェクトの最大の弱点が拡大する。

### 2.3 商用化リスクはガバナンス構造で大きく異なる

すべての OSS が同じリスクを持つわけではない。

| ガバナンス | 商用化リスク | 根拠 |
|---|---|---|
| CNCF Graduated / Incubating | 極めて低い | 財団 TOC の承認なしにライセンス変更不可。過去に変更事例なし |
| Apache Foundation | 極めて低い | Apache License 2.0 固定。財団規約により変更不可 |
| Linux Foundation 傘下 | 低い | 財団ガバナンス下。ただし個別プロジェクトの体制に依存 |
| 企業主導だがフォーク文化あり | 中程度 | Red Hat (Keycloak) / Spotify (Backstage) 等。フォークによる代替が成立する |
| 単一企業支配 | 高い | HashiCorp (Terraform → BSL)、Redis Labs (→ SSPL) の前例あり |

CNCF Graduated プロジェクトが有償化した事例は過去に一度もない。この事実を根拠に、戦略 A（永続利用）を適用できる OSS を明確に区別する。

---

## 3. 各 OSS の戦略分類

### 3.1 戦略 A: 永続利用（インフラ基盤層）

以下の OSS は自作・差し替えのいずれも非現実的であり、永続的に利用する。

| OSS | ガバナンス | 戦略 A の根拠 |
|---|---|---|
| Kubernetes | CNCF Graduated | コンテナオーケストレーションの事実上の標準。代替なし |
| PostgreSQL | PostgreSQL GDG | 30 年以上の開発歴。ライセンス (PostgreSQL License) 変更リスクなし |
| Apache Kafka (Strimzi) | Apache Foundation | Apache License 2.0 固定。イベントストリーミングの業界標準 |
| Istio | CNCF Graduated | サービスメッシュの主流。Envoy データプレーンと共にエコシステムを構成 |
| Envoy | CNCF Graduated | Istio / Envoy Gateway の基盤。単独での差し替え対象にならない |
| Prometheus | CNCF Graduated | メトリクス収集の事実上の標準。Grafana エコシステムの基盤 |
| Grafana | AGPL-3.0 (Grafana Labs) | 可視化基盤。AGPL のためフォーク制約はあるが、SaaS 提供しない k1s0 では影響なし |
| cert-manager | CNCF Incubating | 証明書管理の標準。k8s エコシステムに深く統合 |
| Argo CD | CNCF Graduated | GitOps の標準。CI/CD パイプライン全体に組み込まれており差し替えコストが極めて高い |

これらに対しては、バージョンアップへの追従とセキュリティパッチの適用のみを行う。

### 3.2 戦略 B: 代替 OSS への差し替え準備（ミドルウェア層）

以下の OSS は tier1 封じ込めにより差し替え可能であり、リスク顕在化時に代替 OSS へ乗り換える。

| OSS | ガバナンス | k1s0 ラッパー | 代替候補 |
|---|---|---|---|
| Dapr | CNCF Graduated | `k1s0.*` Go ファサード | Knative / 自前 gRPC Gateway |
| Valkey | Linux Foundation | `k1s0.State.*` (一部) | DragonflyDB (ライセンス注視)、KeyDB、Garnet |
| OpenBao | Linux Foundation | `k1s0.Secrets.*` | CyberArk Conjur OSS、k8s Secrets + ESO 直結 |
| Temporal | 企業主導 (Temporal Inc.) | `k1s0.Workflow.*` | Dapr Workflow (短命)、Apache Airflow (バッチ型) |
| Backstage | CNCF Incubating | 直接利用 (operation 層) | Port (OSS 版)、自前カタログ UI |
| Keycloak | CNCF Incubating | `k1s0.Auth.*` | Zitadel、Authentik |
| flagd | OpenFeature (CNCF) | `k1s0.Feature.*` | Unleash、Flipt |
| Loki | AGPL-3.0 (Grafana Labs) | infra 層直接利用 | VictoriaLogs |
| Tempo | AGPL-3.0 (Grafana Labs) | infra 層直接利用 | Jaeger (CNCF Graduated) |
| Pyroscope | AGPL-3.0 (Grafana Labs) | infra 層直接利用 | Parca (CNCF Sandbox) |
| Harbor | CNCF Graduated | operation 層直接利用 | Zot (CNCF Sandbox) |
| Kyverno | CNCF Incubating | infra 層直接利用 | OPA Gatekeeper (CNCF Graduated) |
| KEDA | CNCF Graduated | infra 層直接利用 | Knative Serving |
| Litmus | CNCF Incubating | operation 層直接利用 | Chaos Mesh (CNCF Incubating) |
| Longhorn | CNCF Incubating | infra 層直接利用 | OpenEBS (CNCF Sandbox)、Rook-Ceph |

代替候補は定期的に見直し、新たな選択肢が登場した場合は本表を更新する。

### 3.3 戦略 C: 自作への置き換え（ドメイン固有ロジック）

以下の OSS は k1s0 固有のドメインロジックに密結合しており、安定稼働後に Rust 自作実装へ段階的に移行する合理性がある。

| OSS | 現在の役割 | 自作移行の根拠 |
|---|---|---|
| ZEN Engine | `k1s0.Decision.*` のルール評価 | JTC 固有のルール体系（承認フロー・権限判定等）に最適化した評価エンジンが必要になる可能性がある。MIT ライセンスかつ Rust 実装のため、フォークからの段階的な自作化が現実的 |
| テンプレートジェネレータ | サービス雛形の自動生成 | k1s0 のレイヤ構造・命名規約・Dapr アノテーション等に完全に特化しており、汎用 OSS では代替できない。現時点で既に自作（Rust CLI） |

これらはいずれも規模が小さく（数千〜数万行）、チームで保守可能な範囲に収まる。

---

## 4. 戦略 B の発動条件と手順

tier1 封じ込めにより差し替え自体は技術的に可能だが、実際に差し替えを判断する基準を定める。

### 4.1 発動条件

以下のいずれかに該当した場合、代替 OSS への差し替えを検討する。

| 条件 | 具体例 |
|---|---|
| ライセンスが OSI 非承認に変更された | Redis → RSALv2 / SSPL のケース |
| メンテナンスが 12 か月以上停止した | コア開発者の離脱・資金枯渇 |
| 致命的な脆弱性が 90 日以上未修正 | CVE Critical が放置 |
| 破壊的変更により移行コストが代替 OSS への移行コストを超える | メジャーバージョンアップで API 全面改訂 |

### 4.2 差し替え手順

1. 本ドキュメントの代替候補表から差し替え先を選定する
2. 差し替え先が選定方針（[`00_選定方針.md`](00_選定方針.md)）の前提条件を満たすことを確認する
3. tier1 内部のラッパー実装を差し替え先の API に接続し直す
4. tier2 / tier3 の統合テストを実行し、公開 API の挙動が変わらないことを検証する
5. 段階的にリリースする（Argo Rollouts による Canary リリースを推奨）

---

## 5. 戦略 C の移行基準

自作への移行は以下の条件をすべて満たした場合にのみ着手する。

| 条件 | 理由 |
|---|---|
| Phase 2 以降で安定稼働が確認されている | 不安定な状態での自作移行は二重のリスク |
| チームが 3 名以上で構成されている | バスファクターを 2 以上に維持するため |
| 既存 OSS の制約が業務要件を阻害している | 「置き換えたい」ではなく「置き換えないと困る」が起点 |
| 自作実装の保守コストが既存 OSS の運用コストを下回ると見積もれる | コスト逆転が見込めない場合は着手しない |

---

## 6. 定期見直し

本ドキュメントの内容は以下のタイミングで見直す。

| タイミング | 見直し内容 |
|---|---|
| 半期ごと | 各 OSS のライセンス・ガバナンス状況の確認、代替候補表の更新 |
| 新規 OSS 採用時 | 戦略分類（A / B / C）の判定と本ドキュメントへの追記 |
| リスク顕在化時 | 該当 OSS の差し替え手順の即時発動 |

---

## 関連ドキュメント

- [`00_選定方針.md`](00_選定方針.md) — 選定の前提条件と OSS リスク管理方針
- [`04_選定一覧.md`](04_選定一覧.md) — 採用 OSS の一覧
- [`../../02_tier1設計/01_設計の核/01_Dapr隠蔽方針.md`](../../02_tier1設計/01_設計の核/01_Dapr隠蔽方針.md) — tier1 封じ込めの具体的実装
- [`../../../01_企画/03_ロードマップと体制/`](../../../01_企画/03_ロードマップと体制/) — フェーズ計画とチーム体制
