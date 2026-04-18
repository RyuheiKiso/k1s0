# 実行基盤中核 OSS の選定

## 目的

k1s0 の実行基盤を構成する中核 OSS (k8s / Istio / Envoy Gateway / Kafka / Dapr / tier1 実装言語) の選定結果と根拠を整理する。周辺 OSS (認証 / CI/CD / レジストリ / キャッシュ) は [`02_周辺OSS.md`](../03_周辺OSS/02_周辺OSS.md) を参照。

---

## 1. コンテナオーケストレーション

| 候補 | 採否 | 評価 |
|---|---|---|
| **Kubernetes (k8s)** | 採用 | デファクトスタンダード。エコシステム最大。学習コストは高いが情報源豊富 |
| Docker Swarm | 却下 | シンプルだが機能不足。新規採用事例も乏しい |
| HashiCorp Nomad | 却下 | 軽量で優秀だがコミュニティ規模が小さく人材確保困難 |

**採用理由**: マイクロサービス基盤としての機能成熟度・エコシステム・人材市場性で他候補を圧倒する。

---

## 2. API ゲートウェイ

| 候補 | 採否 | 評価 |
|---|---|---|
| **Envoy Gateway** | 採用 | Istio と同じ Envoy データプレーンで統一可能。k8s Gateway API ネイティブ準拠 |
| Kong Gateway | 次点 | プラグインエコシステム最大。OSS 版単体で運用可能だが Istio との二重プロキシ問題 |
| Traefik | 次点 | k8s 親和性高い。プラグインエコシステムは Kong より弱く性能も Envoy に劣る |
| NGINX Ingress | 却下 | 実績は十分だが API Gateway 機能 (認証・変換・レート制限) が弱い |

### Kong vs Envoy Gateway 詳細比較

| 観点 | Kong Gateway | Envoy Gateway |
|---|---|---|
| ベース技術 | NGINX / OpenResty (Lua + Go プラグイン) | Envoy Proxy (C++) |
| 設定モデル | 独自 CRD (Kong Ingress Controller) / DB-less。Gateway API 対応中 | **k8s Gateway API ネイティブ準拠**。標準 CRD のみ |
| パフォーマンス | 実用十分。Envoy ほどの高スループットは出ない | Envoy の C++ 実装により**高スループット・低レイテンシ** |
| プラグイン / 拡張 | **OSS だけで数十種類** (OIDC, JWT, rate-limit, transformation 等)。Lua / Go で独自拡張可 | Envoy Filter (Wasm / Lua) で拡張。既製プラグインは Kong より少ないが Wasm でほぼ何でも書ける |
| 認証機能 | OIDC / JWT / OAuth2 / Basic / Key Auth がプラグインで即利用可 | 基本認証は対応。OIDC 等は **ext_authz サーバー (oauth2-proxy 等) との連携**が必要 |
| Istio との共存 | Kong (NGINX) + Istio (Envoy) で**プロキシが 2 種類混在** → 運用二重管理 | **共に Envoy** → 計装・メトリクス・デバッグが統一 |
| 成熟度 | 2015 年〜、GitHub Star 39k+。事例豊富 | CNCF 配下、2023 年 GA。Envoy 自体は CNCF Graduated (2018) |
| 商用版の位置付け | OSS 版と商用版 (Kong Konnect / Enterprise) で**機能差あり** | 純粋な OSS。商用依存なし |
| 日本語情報 | Qiita / Zenn / 書籍の情報が多い | 相対的に少ない。公式ドキュメント (英語) は整備済み |
| Gateway API 対応 | v3 系で実装進行。従来は独自 CRD が中心 | **リファレンス実装に位置付けられ標準性が高い** |

### 採用理由 (Envoy Gateway)

1. **プロキシ技術が Envoy に統一される** — Istio が Envoy を採用しているため、ゲートウェイとメッシュの監視・デバッグ・メトリクスが一本化される
2. **k8s Gateway API ネイティブ準拠** — 将来の乗り換えコストが低い
3. **商用版との機能差を気にしなくて良い** — Kong の OSS / Enterprise の切り分けを監視する必要がない
4. **高レベル機能は tier1 に集約** — 認証・レート制限等を tier1 で共通実装する方針なので、Kong のプラグインに依存する必要がない

### トレードオフと対処

- **初期開発コスト**: 認証・レート制限・変換を tier1 で実装する必要がある → MVP では認証を最優先、他は後続フェーズに送る
- **日本語情報の少なさ**: 公式ドキュメント (英語) を一次情報として扱い、tier1 チームで運用手順書を内製
- **ext_authz サーバー**: OIDC / SSO 連携は `oauth2-proxy` 等を ext_authz として組み合わせる。tier1 の認証サービスが ext_authz インタフェースを満たす設計とする

---

## 3. サービスメッシュ

| 候補 | 採否 | 評価 |
|---|---|---|
| **Istio** | 採用 | 機能最多。運用ノウハウ豊富。オーバーヘッドは大きい |
| Linkerd | 次点 | 軽量でシンプル。Rust 製データプレーン。機能は Istio より絞られる |
| Consul Connect | 却下 | HashiCorp 依存。k8s 前提なら Istio が合う |

**採用理由**: トラフィック制御・mTLS・観測性を一括で提供。CNCF Graduated で信頼性が高い。JTC の長期要件に対応しやすい機能面で Istio を選ぶ。

---

## 4. 分散トレース / メトリクス / ログ

| カテゴリ | 候補 | 採否 | 評価 |
|---|---|---|---|
| 分散トレース | **OpenTelemetry + Grafana Tempo** | 採用 | OTel は計装の業界標準、Tempo は OTLP ネイティブで MinIO (S3) をバックエンドに使用可能。LGTM スタックを完成させる |
| | Jaeger | 不採用 | CNCF Graduated で実績豊富だが、バックエンドに Elasticsearch / Cassandra が必要であり 2 名体制での運用が困難。Red Hat が 2025 年 11 月に Jaeger サポートを終了し Tempo 移行を公式推奨 |
| | Zipkin | 却下 | バックエンドストレージの運用負荷が高い。コミュニティ規模も Jaeger に劣る |
| メトリクス | **Prometheus** | 採用 | デファクト。k8s との統合完成 |
| | VictoriaMetrics | 次点 | Prometheus 互換で高性能。スケール時に検討 |
| | Thanos | 次点 | 長期保存要件が出たら追加導入 |
| ログ | **Loki** | 採用 | 軽量・低コスト・Grafana 統合 |
| | Elasticsearch (ELK) | 却下 | リソース消費大、オンプレ小規模にはオーバースペック |

**採用方針**: OTel を計装層として統一し、バックエンドは差し替え可能な構成にする。tier1 公開 API (`k1s0.Telemetry.*`) が各バックエンドを抽象化する。**OTel Collector** (Agent モード / DaemonSet) をテレメトリパイプラインの中継地点として配置し、サービスからバックエンドへの直接送信を排除する。これにより、バックエンド追加・変更時にサービスの再デプロイが不要になる。トレーシングバックエンドは Grafana Tempo を採用し、Loki / Prometheus / Grafana と合わせて LGTM スタックを構成する。OTel Collector の選定根拠は [`09_ネットワークとテレメトリ基盤.md#S`](../03_周辺OSS/09_ネットワークとテレメトリ基盤.md)、Grafana Tempo の選定根拠は [`11_トレーシングとDB補助.md#W`](../03_周辺OSS/11_トレーシングとDB補助.md) を参照。

---

## 5. メッセージング / イベントバス

| 候補 | 採否 | 評価 |
|---|---|---|
| **Apache Kafka (Strimzi Operator)** | 採用 | 実績豊富。エコシステム最大。イベントソーシング / ストリーム処理・永続化・リプレイが堅牢 |
| NATS (JetStream) | 次点 | 軽量・高速。ただしエコシステムと実績で Kafka に劣る |
| RabbitMQ | 却下 | 実績はあるがスループットとイベントソーシング適性で Kafka に劣る |

### 採用理由 (Kafka)

- 長期運用での実績・エコシステム・イベントソーシング対応を重視
- Saga パターンにおけるイベント履歴のリプレイ性が堅牢
- KRaft モード (Zookeeper 非依存) と Strimzi Operator により運用負荷は以前ほど重くない
- 依然としてリソース消費は大きいため、tier1 が共通クライアントを提供し、利用者が直接 Kafka API に触れない設計とする

---

## 6. 分散トランザクション

| 候補 | 採否 | 評価 |
|---|---|---|
| **Saga パターン** | 採用 | マイクロサービスの標準解。実装負荷はあるが疎結合を保てる |
| 2PC (Two-Phase Commit) | 却下 | 可用性を下げる。マイクロサービスに不向き |
| TCC (Try-Confirm-Cancel) | 却下 | Saga より複雑。学習コスト高 |

**採用方針**: tier1 公開 API `k1s0.Workflow.*` が Saga を隠蔽する。内部実装は Dapr Workflow を利用する。

---

## 7. マイクロサービス building blocks ランタイム (tier1 実装戦略)

tier1 が提供する共通機能 (service invocation / state / pub-sub / secrets / workflow / bindings 等) をどう実装するかの戦略選定。

| 候補 | 採否 | 評価 |
|---|---|---|
| **Dapr** | 採用 | tier1 の基本実装手段として採用。CNCF Graduated、building blocks 豊富、言語非依存 |
| 自前実装 (Rust のみ) | 併用 (補完) | Dapr で対応できない領域のみ tier1 で自前実装 (ログ標準化 / OTel アプリ内部計装 / JTC 固有機能 / 決定エンジン / 雛形 CLI) |
| Cloudstate.io / Akka Platform | 却下 | JVM 依存。学習コスト高 |
| Orleans (.NET) | 却下 | .NET エコシステム固定。言語自由度と整合しない |
| Mia-Platform | 却下 | 商用製品 |

### 採用理由 (Dapr)

1. tier1 が自前実装するはずだった機能の大半を CNCF Graduated レベルで提供済み
2. 言語非依存で tier2 / tier3 の言語自由度を阻害しない
3. Dapr Workflow により Saga オーケストレータの自前実装が不要
4. Component による宣言的なバックエンド切替で infra 変更 (Kafka → NATS 等) の影響を tier2 / tier3 に波及させない
5. 業界標準に乗れるためアプリチームの学習資産が市場価値を持つ

### 並立構成の原則

- Dapr を基本、足りない箇所を tier1 自前実装で補完 (Go + Rust ハイブリッド)
- Daprファサードは tier1 内部の Go サービスが stable Dapr Go SDK を直接利用
- Rust は Dapr 非依存の自作領域 (ZEN Engine 統合 / JTC 固有機能 / 雛形 CLI / Component バリデータ) に限定
- **tier2 / tier3 は Dapr を一切意識しない**。tier1 公開 API のみを利用する

詳細は [`../../02_tier1設計/01_設計の核/01_Dapr隠蔽方針.md`](../../02_tier1設計/01_設計の核/01_Dapr隠蔽方針.md) を参照。

---

## 8. tier1 実装言語 (内部サービス)

tier1 の内部サービスの実装言語選定。公開クライアントライブラリは別系統で各対象言語 (C# / Go / TS) に提供する。

| 候補 | 採否 | 評価 |
|---|---|---|
| **Go (Daprファサード)** | 採用 | Dapr Go SDK が stable。Daprファサード系の薄いハンドラで生産性が高い。k8s / OTel / Loki / oauth2-proxy 等のエコシステムが Go 中心で人材も豊富 |
| **Rust (自作領域)** | 採用 | メモリ安全・高性能・長期保守性。ZEN Engine の in-process 統合 / 雛形生成 CLI / JTC 固有機能で Rust の優位性が出る |
| C# (.NET) | 却下 | Linux 対応は実用だが、Dapr Go SDK と同等の枯れ具合がなく k8s 周辺エコシステムとの整合性で Go に劣る |

**採用方針**: tier1 内部は Daprファサード = Go、自作領域 = Rust のハイブリッド構成。詳細は [`../../02_tier1設計/01_設計の核/02_内部言語ハイブリッド.md`](../../02_tier1設計/01_設計の核/02_内部言語ハイブリッド.md) を参照。

---

## 9. コンテナランタイム

| 候補 | 採否 | 評価 |
|---|---|---|
| **containerd** | 採用 | k8s 標準。Docker 依存を排除できる |
| Docker Engine | 併用 | 開発環境は Docker Desktop を許容 |
| Podman | 却下 | デーモンレスで良いが k8s 本番運用での採用例が少ない |

---

## 関連ドキュメント

- [`00_選定方針.md`](../01_俯瞰/00_選定方針.md) — 前提条件と判断軸
- [`02_周辺OSS.md`](../03_周辺OSS/02_周辺OSS.md) — 認証 / CI/CD / レジストリ / キャッシュ
- [`03_ルールエンジン.md`](03_ルールエンジン.md) — ZEN Engine
- [`04_選定一覧.md`](../01_俯瞰/04_選定一覧.md) — 全 OSS の一覧
- [`09_ネットワークとテレメトリ基盤.md`](../03_周辺OSS/09_ネットワークとテレメトリ基盤.md) — MetalLB / kube-vip / OTel Collector
- [`18_言語選定の定量分析.md`](../01_俯瞰/18_言語選定の定量分析.md) — 本資料の Go / Rust 採用に対する工数・性能の定量裏付け
- [`../../02_tier1設計/01_設計の核/01_Dapr隠蔽方針.md`](../../02_tier1設計/01_設計の核/01_Dapr隠蔽方針.md) — Dapr を採用したうえで tier1 が何を隠蔽するか（本資料の選定を受けた設計判断）
- [`../../02_tier1設計/01_設計の核/02_内部言語ハイブリッド.md`](../../02_tier1設計/01_設計の核/02_内部言語ハイブリッド.md) — Go + Rust の設計意図（本資料の言語選定を受けた設計判断）
- [`../../02_tier1設計/`](../../02_tier1設計/) — tier1 内部設計の詳細
- [`../../../01_企画/02_競合と差別化/01_主要製品別評価.md`](../../../01_企画/02_競合と差別化/01_主要製品別評価.md) — Dapr との関係
