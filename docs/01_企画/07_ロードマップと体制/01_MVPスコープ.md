# MVP スコープ

## 目的

Phase 1 (MVP) で **何を含め、何を含めないか** を明示する。MVP のゴールは「k1s0 の 5 つの勝ち筋が小規模でも成立することを示す」ことであり、機能網羅ではない。

---

## 1. MVP の原則

1. **起案者単独で立ち上げ可能な規模に抑える** — 協力体制は Phase 2 以降
2. **1 つの小規模業務で実証** — パイロット業務を 1 件選び、そこで価値を示す
3. **スコープ外の機能はスタブで十分** — tier1 公開 API の形だけ決めておき、実装は後続フェーズ
4. **「運用可能である」を優先** — 派手な機能より、JTC 情シスが運用できる状態を示す
5. **競合製品との差別化ポイントを MVP で先に見せる** — レガシー共存 / オンプレ完結 / 言語自由のうち、最低 1 つは MVP で実証

---

## 2. MVP に含めるもの

### infra 層

| コンポーネント | スコープ | 備考 |
|---|---|---|
| OpenTofu | k8s ノード用 VM 作成 + k8s bootstrap の最小 HCL | `tofu apply` で環境再現可能にする |
| Kubernetes | 最小 3 ノード構成 | オンプレまたは VM 上 (OpenTofu で構築) |
| Istio | サービスメッシュ (mTLS + トラフィック制御) | サイドカー自動注入 |
| Envoy Gateway | API Gateway | k8s Gateway API 準拠 |
| Apache Kafka (Strimzi) | KRaft モード | Phase 1 は最小クラスタ |
| OpenTelemetry + Jaeger | 分散トレース | Collector 経由 |
| Prometheus | メトリクス | — |
| Loki + Grafana | ログ / 可視化 | — |
| Valkey | キャッシュ / KV | tier1 State / Configuration のバックエンド |
| CloudNativePG + PostgreSQL | RDBMS | プライマリ 1 + レプリカ 1。Keycloak / Backstage / Argo CD / Harbor が共有 |

### tier1 層

| コンポーネント | スコープ | 備考 |
|---|---|---|
| Dapr Control Plane | operator / sidecar-injector / sentry | operation namespace に配置 |
| tier1 公開 API | `k1s0.Log` / `k1s0.Telemetry` のみ実装 | 他の API はスタブ定義のみ |
| tier1 公開 API スタブ | `k1s0.State` / `k1s0.PubSub` / `k1s0.Workflow` / `k1s0.Secrets` / `k1s0.Auth` / `k1s0.Audit` / `k1s0.Decision` / `k1s0.Settings` | インタフェース契約のみ |
| tier1 内部 Go サービス | Daprファサード (最小 1 サービス) | stable Dapr Go SDK を使用 |
| tier1 内部 Rust サービス | (Phase 2 以降) | MVP では不要 |
| 雛形生成 CLI | 最小テンプレート (C# or Go 向け 1 パターン) | GHA workflow / deploy yaml / catalog-info.yaml を生成 |
| リファレンス実装 | 1 本のサンプル tier1 クライアント | 模範コードとして機能 |

### tier2 層

| 項目 | スコープ |
|---|---|
| サンプルサービス | (Phase 2 以降) MVP では 1 本だけリファレンス用に作る |

### tier3 層

| 項目 | スコープ |
|---|---|
| アプリ配信ポータル | 最小機能 (アプリカタログ表示 / 認証連携 / Web アプリ起動リンク / 監査ログ記録) |
| サンプル業務アプリ | (Phase 2 以降) |

### operation 層

| コンポーネント | スコープ |
|---|---|
| Keycloak | Realm `k1s0`、ローカル DB、AD 連携なし |
| Argo CD | tier1 向け ApplicationSet 1 つ |
| Harbor | プロジェクト `tier1` / `tier2` / `tier3` / `infra` の 4 つ |
| Trivy | Harbor 内蔵。push 時の自動スキャン (Critical で拒否) |
| GitHub Actions self-hosted runner | `actions-runner-controller` 管理、最小 2 Pod |
| Backstage | Software Catalog + TechDocs + Keycloak SSO 連携 |

### パイプライン

| ステージ | スコープ |
|---|---|
| Lint / 型 / UT / Build / FS Scan / Push / GitOps 更新 | GHA 1 本のワークフローで完結 |
| イメージ署名 (Cosign) | (Phase 2 以降) |
| Tekton 代替フロー | (Phase 2 以降) |

---

## 3. MVP に含めないもの (明示的な除外)

「やらないこと」を明示することは「やること」を明示することと同じくらい重要。

| 項目 | 除外理由 | いつ着手 |
|---|---|---|
| tier1 公開 API の full 実装 (State / PubSub / Workflow 等) | MVP では API 契約のみで十分。tier2 / tier3 サンプルが増える Phase 2 で本格実装 | Phase 2 |
| tier2 サンプルサービス (複数) | 1 本のリファレンス実装で価値を示せる | Phase 2 |
| ネイティブアプリ配信 (MSIX / ClickOnce) | 第一選択は PWA。ネイティブは Phase 3 で十分 | Phase 3 |
| 端末設定コピー | スキーマ設計は tier3 アプリが増えてから現実的 | Phase 3 |
| 申請ワークフロー / 稟議承認 | Dapr Workflow が必要、tier1 実装の後半 | Phase 4 |
| ZEN Engine 本格実装 | API スタブのみで Phase 1 は足りる。実装は Phase 2 で開始 | Phase 2 |
| レガシー .NET Framework ラップ配信 | 最も複雑な形態。共存実証は Phase 4〜5 | Phase 4〜5 |
| Cosign 署名 / Kyverno 未署名ブロック | サプライチェーンセキュリティの強化。MVP のスコープに含めると構築コストが跳ねる | Phase 2 |
| マルチクラスタ (staging / prod 分離) | 1 クラスタで十分価値を示せる | Phase 3 |
| AD 連携 | Keycloak ローカル DB で MVP ユーザー数十名を運用 | Phase 2 以降 |
| Tekton 代替フロー | GHA が使えない環境が出てから着手 | Phase 2 以降 |
| 内製 analyzer (禁止パターン検知) | CI ガードで最低限の防御はできる | Phase 3 以降 |

---

## 4. ハードウェア要件 (MVP 最小構成)

MVP の全コンポーネントを 3 ノードの k8s クラスタ上で稼働させるために必要なハードウェアリソースの見積もり。

### 4.1 コンポーネント別メモリ概算

| コンポーネント群 | 概算メモリ | 備考 |
|---|---|---|
| k8s システム (kubelet / etcd / apiserver / controller-manager / scheduler / CoreDNS) | 6 GB (2 GB × 3 ノード) | etcd は奇数ノードで分散 |
| Istio (istiod + sidecar) | 2 GB + sidecar 各 128 MB | istiod 1 レプリカ + 各 Pod にサイドカー |
| Envoy Gateway | 0.5 GB | コントローラ + Envoy Pod |
| Apache Kafka (Strimzi, 3 broker KRaft) | 6 GB | broker あたり 2 GB。JVM ヒープ含む |
| Prometheus | 2 GB | メトリクス保持期間に依存。MVP は 15 日 |
| Loki | 1 GB | 最小構成 (monolithic mode) |
| Jaeger | 1 GB | all-in-one 構成 |
| Grafana | 0.5 GB | — |
| OTel Collector | 0.5 GB | — |
| Valkey | 0.5 GB | — |
| CloudNativePG + PostgreSQL (1 primary + 1 replica) | 2 GB | shared_buffers 含む |
| Dapr Control Plane (operator / sidecar-injector / sentry / placement) | 1 GB | — |
| Keycloak (2 レプリカ) | 1.5 GB | JVM ヒープ含む |
| Backstage | 1 GB | Node.js プロセス |
| Harbor (core / registry / trivy / jobservice / portal) | 4 GB | Trivy DB キャッシュ含む |
| Argo CD (server / repo-server / controller) | 1.5 GB | — |
| GHA self-hosted runner (2 Pod) | 2 GB | ビルド時は一時的に増加 |
| tier1 Go サービス + Dapr sidecar | 0.5 GB | — |
| アプリ配信ポータル + Dapr sidecar | 0.5 GB | — |
| **合計** | **約 35 GB** | 余裕を見て 40 GB 以上を推奨 |

### 4.2 ノード別要件

| 項目 | 最小要件 | 推奨要件 |
|---|---|---|
| ノード数 | 3 | 3 |
| vCPU / ノード | 8 | 16 |
| メモリ / ノード | 16 GB | 32 GB |
| ディスク / ノード | 200 GB SSD | 500 GB SSD |
| ネットワーク | 1 Gbps | 10 Gbps |

### 4.3 補足

- **最小要件 (8 vCPU / 16 GB / 200 GB SSD × 3)** は動作するが、ビルド時や Kafka 負荷時に不安定になる可能性がある。開発 / 検証用途に限定
- **推奨要件 (16 vCPU / 32 GB / 500 GB SSD × 3)** であれば Phase 2 の tier2 サンプルサービス追加まで余裕を持って収容可能
- ディスクは SSD を必須とする。HDD では PostgreSQL / Kafka / etcd の I/O レイテンシが許容範囲を超える
- vSphere 環境であれば仮想マシン 3 台を 1 つの物理ホスト上に配置可能。ただし HA を確保するなら 2 ホスト以上に分散を推奨
- OpenTofu の HCL でこれらの VM スペックを宣言し、`tofu apply` で再現可能にする

### 4.4 Phase 別のスケール想定

| フェーズ | ノード数 | ノードスペック | 備考 |
|---|---|---|---|
| Phase 1 (MVP) | 3 | 8〜16 vCPU / 16〜32 GB / 200〜500 GB SSD | 最小構成 |
| Phase 2 | 3〜5 | 16 vCPU / 32 GB / 500 GB SSD | tier2 サービス追加に伴いノード増設 |
| Phase 3 | 5〜8 | 同上 | staging / prod 分離開始 |
| Phase 5 | 10+ | 同上 + ストレージノード追加 | 全社ロールアウト |

---

## 5. MVP のゴールと成功指標

| 軸 | ゴール |
|---|---|
| 機能 | 1 業務のパイロットが Web アプリとして配信ポータルから起動できる |
| 開発体験 | 雛形生成 CLI から 1 コマンドで tier1 リファレンス実装と同等のサービスが立ち上がる |
| 運用 | Backstage から Software Catalog / TechDocs / Argo CD 同期状態を確認できる |
| 認証 | Argo CD / Harbor / Backstage / 配信ポータルが Keycloak SSO で統一される |
| 監査 | 配信ポータルの全起動イベントが tier1 監査ログ (スタブでも可) に記録される |
| 再現性 | `tofu apply` で k8s クラスタが再構築できる (バス係数の緩和) |
| 差別化 | オンプレ k8s で完結し、クラウド依存が一切ない状態で稼働する |

---

## 6. MVP 完了の判定条件

以下がすべて満たされた時点で Phase 2 へ移行する。

1. tier1 リファレンス実装で GHA フロー (PR → ビルド → スキャン → Harbor push → GitOps 更新 → Argo CD 同期) が疎通する
2. パイロット業務の Web アプリが配信ポータルからエンドユーザーが起動できる
3. Keycloak SSO が Argo CD / Harbor / Backstage / 配信ポータルで機能する
4. Backstage Software Catalog に tier1 リファレンス実装が登録されている
5. 運用手順書 (再起動 / バックアップ / リストア / ログ閲覧) が TechDocs として公開されている

---

## 関連ドキュメント

- [`00_フェーズ計画.md`](./00_フェーズ計画.md) — 全フェーズの俯瞰
- [`02_体制と役割.md`](./02_体制と役割.md) — MVP 時点の体制
- [`../05_CICDと配信/00_CICDパイプライン.md`](../05_CICDと配信/00_CICDパイプライン.md) — GHA ワークフローの MVP スコープ
- [`../05_CICDと配信/02_アプリ配信ポータル.md`](../05_CICDと配信/02_アプリ配信ポータル.md) — 配信ポータルの MVP スコープ
- [`../04_技術選定/04_選定一覧.md`](../04_技術選定/04_選定一覧.md) — 採用 OSS 一覧
- [`../04_技術選定/05_IaC.md`](../04_技術選定/05_IaC.md) — OpenTofu の採用根拠と MVP スコープ
- [`../06_競合と差別化/03_TCOとBuildVsBuy.md`](../06_競合と差別化/03_TCOとBuildVsBuy.md) — MVP 最小化と Build リスク低減の関係
