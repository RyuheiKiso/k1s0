# Protobuf スキーマ管理とロールアウト戦略 OSS の選定

## 目的

tier1 内部通信で必須の Protobuf スキーマ管理、段階的デプロイを実現するプログレッシブデリバリー、Kubernetes オブジェクト状態の可視化を担う OSS を整理する。中核 OSS 群 ([`01_実行基盤中核OSS.md`](../02_中核OSS/01_実行基盤中核OSS.md)) を安全に運用するための補完レイヤである。対象カテゴリは以下のとおり。

- T. Protobuf スキーマ管理
- U. プログレッシブデリバリー
- V. Kubernetes オブジェクト状態メトリクス

---

## T. Protobuf スキーマ管理

| 候補 | 採否 | 評価 |
|---|---|---|
| **Buf** | 採用 | Protobuf のリント・後方互換検証・コード生成を統合する CLI ツール。Apache 2.0 |
| protoc + 個別プラグイン | 却下 | protoc 本体 + protoc-gen-go + protoc-gen-go-grpc + protoc-gen-csharp 等を個別管理する必要があり、バージョン整合性の維持が煩雑 |
| protolock | 却下 | 後方互換検証のみ。リント・コード生成・ドキュメント生成を別途用意する必要がある |

### 採用理由 (Buf)

1. **tier1 の Protobuf 契約管理を単一ツールで完結** — `buf lint` (スタイル検証) + `buf breaking` (後方互換検証) + `buf generate` (多言語コード生成) を 1 つの CLI で提供する。protoc + 個別プラグインの組み合わせ管理が不要になる
2. **後方互換性破壊を CI で機械的に検出** — tier1 公開 API は tier2 / tier3 の唯一の接点であり、Protobuf 契約の破壊的変更は全サービスに影響する。`buf breaking` が PR 時に自動検出し、APIバージョニング戦略の実行手段となる
3. **Protobuf スタイルの自動強制** — `buf lint` が命名規約・フィールド番号・パッケージ構成を検証する。API 設計原則の「規律はツールで強制する」を Protobuf レベルで実現する
4. **多言語コード生成の一元管理** — `buf.gen.yaml` に C# / Go / TypeScript の生成設定を集約する。tier1 内部言語ハイブリッド (Go + Rust) および tier2 / tier3 向けクライアントライブラリの全言語を一括で生成・検証できる
5. **Buf Schema Registry (BSR) はオプション** — BSR は Buf 社のクラウドサービスだが、k1s0 では使用しない。CLI のみで完結し、`.proto` ファイルは `src/tier1/contracts/` に Git 管理する。ベンダーロックインなし

### 既存ドキュメントとの整合

[`04_APIバージョニング戦略.md`](../../02_tier1設計/02_API契約/04_APIバージョニング戦略.md) および [`03_API設計原則.md`](../../02_tier1設計/02_API契約/03_API設計原則.md) で Buf の利用は既に前提とされている。本ドキュメントは正式な選定根拠の記録である。

### CI パイプラインへの統合

| ステージ | 実行内容 | タイミング |
|---|---|---|
| `buf lint` | `.proto` ファイルのスタイル・命名規約違反を検出 | PR 時 (GHA step) |
| `buf breaking` | `main` ブランチとの差分で後方互換性破壊を検出 | PR 時 (GHA step) |
| `buf generate` | C# / Go / TypeScript / Rust のコードを自動生成 | リリース時 (GHA step) |

### MVP スコープ

- Phase 1 (MVP-1a) から導入
- GHA self-hosted runner のカスタムイメージに `buf` CLI を同梱
- `src/tier1/contracts/` 内の `.proto` に対して `buf lint` + `buf breaking` を CI で実行
- `buf generate` は MVP-1a では C# / Go の 2 言語。TypeScript は Phase 2 以降
- 開発者 PC にも `buf` CLI をインストール (Tilt の pre-commit hook で `buf lint` を実行)

### トレードオフ

- Buf CLI はオープンソース (Apache 2.0) だが、Buf Schema Registry (BSR) は商用サービスのため利用しない。`.proto` ファイルの Git 管理 + CI での検証で同等の品質保証を実現する
- `buf generate` が生成するコードと、手書きの Rust コード生成 (`tonic-build`) の整合性を保つ必要がある → Rust は `buf generate` ではなく `tonic-build` を継続利用し、`.proto` の lint / breaking チェックのみ `buf` で実施する

---

## U. プログレッシブデリバリー

| 候補 | 採否 | 評価 |
|---|---|---|
| **Argo Rollouts** | 採用 | Argoproj エコシステム。Argo CD と統合コスト低。Istio トラフィック分割と連携してカナリア分析。Apache 2.0 |
| Flagger | 次点 | Flux エコシステムが前提。Argo CD を採用している k1s0 では統合コストが高い |
| Istio 手動トラフィック分割 | 却下 | VirtualService の weight 手動変更は属人的で再現性がない。ロールバック判定の自動化ができない |
| kubectl rollout (標準) | 却下 | カナリアリリース・ブルーグリーンデプロイに対応しない。全 Pod 一斉入替のみ |

### 採用理由 (Argo Rollouts)

1. **Argo CD と同一エコシステムで統合コスト低** — Argo Rollouts は Argoproj の一部であり、Argo CD の Application から Rollout リソースをそのまま管理できる。追加の連携設定が最小限で済む
2. **Istio のトラフィック分割と連携したカナリアリリース** — Istio (Phase 2 で導入) の VirtualService を自動操作し、新バージョンへのトラフィック割合を段階的に増加させる。5% → 25% → 50% → 100% のような段階制御が宣言的に定義できる
3. **Prometheus メトリクスに基づく自動ロールバック判定** — AnalysisTemplate でカナリアバージョンのエラー率・レイテンシを Prometheus から取得し、閾値を超えた場合に自動でロールバックする。人間の判断を待たずに障害拡大を防止する
4. **JTC 環境での安全なデプロイを実現** — JTC では「新バージョンで障害が出たら即座に旧バージョンに戻せること」が信頼獲得の前提条件。Argo Rollouts の自動ロールバックはこの要件に直接対応する
5. **ブルーグリーンデプロイにも対応** — カナリアリリースだけでなく、ブルーグリーンデプロイ (新旧環境の切替) にも対応する。tier1 のようなクリティカルなサービスではブルーグリーンが安全な選択肢になる

### デプロイ戦略の使い分け

| 戦略 | 対象 | 理由 |
|---|---|---|
| カナリアリリース | tier2 / tier3 サービス | トラフィックベースの段階的検証が可能。エンドユーザー影響を最小化 |
| ブルーグリーンデプロイ | tier1 サービス | 基盤サービスはカナリア中の不整合リスクを避け、全切替で安全に更新 |
| 標準ローリングアップデート | infra コンポーネント (Operator 等) | Operator は Rollout リソースに変換不要。Helm / kustomize の標準更新で十分 |

### AnalysisTemplate の設計

Argo Rollouts のカナリア分析は AnalysisTemplate で定義する。以下は tier2 サービス向けの標準テンプレート。

| メトリクス | データソース | 成功条件 | 測定間隔 |
|---|---|---|---|
| リクエスト成功率 | Prometheus (Istio メトリクス) | 成功率 >= 99% | 60 秒 |
| p99 レイテンシ | Prometheus (Istio メトリクス) | p99 < 500ms | 60 秒 |
| Pod 再起動回数 | Prometheus (kube-state-metrics) | 再起動 = 0 | 60 秒 |

雛形生成 CLI が Rollout リソースと AnalysisTemplate を同時に生成する。開発者はメトリクス閾値のみカスタマイズ可能とし、分析ロジック自体の改変は禁止する。

### Argo CD との統合

- Argo CD の Application で Rollout リソースを管理する (Deployment の代わりに Rollout を使用)
- Argo CD の Health Assessment が Rollout の進行状態 (Progressing / Healthy / Degraded) を認識する
- Argo Rollouts Dashboard を Backstage に統合し、ロールアウトの進行状態を可視化する

### MVP スコープ

- **Phase 2 で導入** (Istio 導入と同時)
- `operation` namespace にデプロイ (Argo Rollouts Controller)
- Istio TrafficRouting integration を有効化
- 雛形生成 CLI が Deployment ではなく Rollout リソースを生成するオプションを追加
- tier2 サンプルサービスでカナリアリリースの動作検証
- AnalysisTemplate は Prometheus + Istio メトリクスベースの標準テンプレートを 1 つ提供

### トレードオフ

- Argo Rollouts は Deployment リソースの代わりに Rollout CRD を使用するため、既存の Deployment を移行する必要がある。ただし雛形生成 CLI が Rollout を直接生成するため、新規サービスでは追加作業なし
- MVP (Phase 1) では Istio が未導入のため、Argo Rollouts も導入しない。Phase 1 では標準の kubectl rollout で運用する
- カナリア分析の閾値設定は運用実績に基づいて調整が必要。Phase 2 初期は手動承認ステップを残し、メトリクス閾値の妥当性を確認してから全自動に移行する

---

## V. Kubernetes オブジェクト状態メトリクス

| 候補 | 採否 | 評価 |
|---|---|---|
| **kube-state-metrics** | 採用 | k8s SIG 公式。Kubernetes オブジェクトの状態をメトリクスとして公開する唯一のデファクト。Apache 2.0 |
| カスタム Prometheus Exporter | 却下 | 自前実装の保守コストが高く、k8s API の追従が困難 |
| Prometheus node_exporter のみ | 不十分 | node_exporter はノードレベルのリソースメトリクス (CPU / メモリ / ディスク) を提供するが、k8s オブジェクトの状態 (Deployment replicas / Pod phase / PVC status 等) は取得できない |

### 採用理由 (kube-state-metrics)

1. **Prometheus 単体では取得できない k8s オブジェクト状態を補完** — Prometheus の cAdvisor メトリクスはコンテナのリソース消費を計測するが、Deployment の desired/available replicas、Pod の phase (Pending/Running/Failed)、PVC の bound 状態、Job の成功/失敗数といった k8s オブジェクトの「状態」は kube-state-metrics がなければ取得できない
2. **プラットフォーム自己監視の前提条件** — [`../../01_アーキテクチャ/02_可用性と信頼性/02_プラットフォーム自己監視.md`](../../01_アーキテクチャ/02_可用性と信頼性/02_プラットフォーム自己監視.md) で定義された infra 層・tier1 層の監視項目の多くが kube-state-metrics のメトリクスに依存する。これがなければアラートルールが機能しない
3. **Argo Rollouts のカナリア分析に必須** — Argo Rollouts の AnalysisTemplate で「Pod 再起動回数が 0 であること」を検証するには `kube_pod_container_status_restarts_total` メトリクスが必要。これは kube-state-metrics が提供する
4. **k8s SIG 公式プロジェクト** — Kubernetes Special Interest Group (SIG Instrumentation) が管理しており、k8s バージョンとの互換性が公式に保証される。コミュニティも活発で長期的な保守が安定している
5. **軽量で運用負荷が極めて低い** — Deployment 1 つ (1〜2 レプリカ) をデプロイするだけで動作する。設定はほぼデフォルトで十分であり、2 名体制でも追加の運用負荷はほぼゼロ

### 提供される主要メトリクス

| メトリクス | 提供情報 | 利用先 |
|---|---|---|
| `kube_deployment_spec_replicas` / `kube_deployment_status_replicas_available` | Deployment の desired vs available replicas | Grafana ダッシュボード / アラート |
| `kube_pod_status_phase` | Pod の状態 (Pending / Running / Succeeded / Failed / Unknown) | Pod 異常検知アラート |
| `kube_pod_container_status_restarts_total` | コンテナの累計再起動回数 | CrashLoopBackOff 検知 / Argo Rollouts AnalysisTemplate |
| `kube_node_status_condition` | ノードの Ready / MemoryPressure / DiskPressure 等 | ノード異常検知アラート |
| `kube_persistentvolumeclaim_status_phase` | PVC の Bound / Pending 状態 | ストレージ異常検知 |
| `kube_job_status_succeeded` / `kube_job_status_failed` | Job の成功/失敗数 | マイグレーション Job / バックアップ Job の監視 |
| `kube_daemonset_status_number_ready` | DaemonSet の Ready Pod 数 | OTel Collector / MetalLB speaker の稼働監視 |
| `kube_statefulset_status_replicas_ready` | StatefulSet の Ready Pod 数 | PostgreSQL / Kafka / OpenBao の稼働監視 |

### Prometheus との統合

kube-state-metrics は Prometheus の ServiceMonitor (Prometheus Operator) でスクレイプする。

```
Prometheus
    ↓ scrape (ServiceMonitor)
kube-state-metrics (Deployment)
    ↓ k8s API watch
k8s API Server
```

kube-state-metrics は k8s API Server を watch し、オブジェクトの状態変更をリアルタイムでメトリクスに反映する。Prometheus がこのメトリクスを定期スクレイプする。

### MVP スコープ

- Phase 1 (MVP-1a) から導入
- `infra` namespace にデプロイ (Deployment、1 レプリカ)
- Prometheus の ServiceMonitor でスクレイプ対象に追加
- Grafana に k8s クラスタ状態ダッシュボードを追加 (Deployment / Pod / Node の一覧と状態)
- アラートルール: Pod CrashLoopBackOff / Deployment replicas mismatch / Node NotReady / PVC Pending

### メモリ概算

| 構成 | メモリ |
|---|---|
| kube-state-metrics (1 レプリカ) | 0.1 GB |

3 ノード / Pod 数 50 未満の MVP 規模ではメモリ消費は極めて少ない。Phase 2 以降でオブジェクト数が増加した場合も 0.2〜0.3 GB で収まる。

### トレードオフ

- kube-state-metrics は k8s API Server を watch するため、API Server への負荷が微増する。ただし read-only の watch であり、MVP 規模では影響は無視できる
- メトリクスのカーディナリティが高い (namespace × resource × name の組合せ) ため、Prometheus のストレージ消費がやや増加する。Phase 2 以降で `metricLabelsAllowlist` / `metricAnnotationsAllowList` の設定で不要なメトリクスを除外することを検討する

---

## 関連ドキュメント

- [`07_ストレージと運用補助.md`](07_ストレージと運用補助.md) — 追加 OSS の選定 (L〜S)
- [`09_ネットワークとテレメトリ基盤.md`](09_ネットワークとテレメトリ基盤.md) — ネットワークとテレメトリ基盤の選定 (Q〜S)
- [`04_選定一覧.md`](../01_俯瞰/04_選定一覧.md) — 採用 OSS 選定一覧
- [`../../02_tier1設計/02_API契約/04_APIバージョニング戦略.md`](../../02_tier1設計/02_API契約/04_APIバージョニング戦略.md) — Buf の利用を前提とした Protobuf 契約管理
- [`../../02_tier1設計/02_API契約/03_API設計原則.md`](../../02_tier1設計/02_API契約/03_API設計原則.md) — CI ガードの一部として Buf を利用
- [`../../04_CICDと配信/00_CICDパイプライン.md`](../../04_CICDと配信/00_CICDパイプライン.md) — CI パイプラインへの Buf / Argo Rollouts の統合
- [`../../01_アーキテクチャ/02_可用性と信頼性/02_プラットフォーム自己監視.md`](../../01_アーキテクチャ/02_可用性と信頼性/02_プラットフォーム自己監視.md) — kube-state-metrics が提供するメトリクスの利用先
