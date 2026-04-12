# 追加採用 OSS の選定

## 目的

[`02_周辺OSS.md`](./02_周辺OSS.md) (A〜K) に続き、追加で採用が決定した OSS の選定根拠を整理する。対象カテゴリは以下のとおり。

- L. 分散ストレージ
- M. 接続プーリング
- N. 依存パッケージ自動更新
- O. イベント駆動自動化

---

## L. 分散ストレージ

| 候補 | 採否 | 評価 |
|---|---|---|
| **Longhorn** | 採用 | CNCF Incubating。k3s と同じ SUSE 開発で実績豊富。UI 管理でレプリケーション / スナップショット / バックアップを提供 |
| Rook-Ceph | 却下 | 運用負荷が JTC 2 名体制に対して過大 |
| ローカル PV | 却下 | ノード障害時にデータ消失 |

### 採用理由 (Longhorn)

1. **k3s と同じ SUSE 開発で実績豊富** — CNCF Incubating プロジェクトとしてコミュニティも活発
2. **3 ノードレプリケーション / スナップショット / バックアップを UI 管理可能** — JTC 2 名体制でも運用可能な低い学習曲線
3. **CloudNativePG の HA フェイルオーバーが PV のノード依存なしで確実に機能** — ローカル PV ではフェイルオーバー先ノードにデータがなく HA が破綻する
4. **MinIO の PV も保護** — オブジェクトストレージ自体のデータ永続性を Longhorn のレプリケーションで担保

### MVP スコープ

- Phase 1 (MVP-1a) から導入
- `infra` namespace にデプロイ
- レプリカ数 2 (3 ノード中 2 つにデータ複製)
- CloudNativePG / MinIO / Harbor の PV を Longhorn で管理

### トレードオフ

- ストレージのオーバーヘッドが発生するが、データ保護の観点で必須
- Phase 2 でレプリカ数 3 に引き上げ

---

## M. 接続プーリング

| 候補 | 採否 | 評価 |
|---|---|---|
| **PgBouncer (CloudNativePG Pooler CRD)** | 採用 | CloudNativePG の `Pooler` CRD で宣言的に配置可能。追加導入コスト最小 |
| Pgpool-II | 却下 | 機能過多で設定複雑 |
| アプリ側接続プール | 却下 | サービスごとの管理が煩雑 |

### 採用理由 (PgBouncer / CloudNativePG Pooler)

1. **CloudNativePG の `Pooler` CRD で宣言的に配置可能** — 追加導入コスト最小で Operator エコシステムに統合
2. **PostgreSQL 共有クラスタに Keycloak / Backstage / ArgoCD / Harbor / Audit 等 5+ DB を収容するため接続プーリング必須** — DB 統合方針の前提条件
3. **PostgreSQL デフォルト max_connections=100 に対し、MVP-1a 時点で約 85 接続が見込まれ余裕がない** — 接続プーリングなしでは接続枯渇リスクが高い
4. **接続枯渇は Keycloak 停止 → 認証基盤全停止のカスケード障害を引き起こす** — 認証基盤の可用性を守るための必須措置

### MVP スコープ

- Phase 1 (MVP-1a) から導入
- CloudNativePG Pooler CRD 1 つで PgBouncer を配置
- PostgreSQL への実接続数を 20-30 に固定
- サービスは PgBouncer 経由で接続

### トレードオフ

- プリペアドステートメントの扱いに注意 (transaction pooling mode では非対応)
- `session` mode をデフォルトとし、パフォーマンス要件が出た時点で `transaction` mode を検討

---

## N. 依存パッケージ自動更新

| 候補 | 採否 | 評価 |
|---|---|---|
| **Renovate** | 採用 | Mend 社管理。Go modules / Cargo.toml / NuGet / npm / Dockerfile / Helm chart / k8s manifest のイメージタグを一元管理 |
| Dependabot | 却下 | GitHub.com 専用で self-hosted runner での柔軟性が低い |
| 手動更新 | 却下 | 30+ OSS の CVE 追跡を 2 名体制で実施不能 |

### 採用理由 (Renovate)

1. **Go modules / Cargo.toml / NuGet / npm / Dockerfile / Helm chart / k8s manifest のイメージタグを一元管理** — 全技術スタックの依存を単一ツールでカバー
2. **CVE 検知 → 修正 PR 自動作成 → CI 自動実行でマージボタンを押すだけ** — 2 名体制での運用負荷を最小化
3. **非機能要件「CVE Critical: 48 時間以内」を 2 名体制で達成する唯一の手段** — 手動では 30+ OSS の追跡が物理的に不可能

### MVP スコープ

- Phase 1 (MVP-1a) から導入
- GHA self-hosted runner 上で Renovate を週次実行
- 対象: tier1 Go/Rust 依存 + Dockerfile + Helm chart
- Backstage 未導入時は GitHub PR Dashboard で更新状況を管理

### トレードオフ

- 自動 PR が大量に生成される可能性 → automerge ルール (patch バージョンは CI pass で自動マージ) とグルーピング (同一エコシステムの更新をまとめる) で対処

---

## O. イベント駆動自動化

| 候補 | 採否 | 評価 |
|---|---|---|
| **Argo Events** | 採用 | Argoproj エコシステム。Argo CD と統合コスト低。Apache 2.0 |
| Knative Eventing | 却下 | Knative 全体の導入が必要で重量級 |
| 自前 Webhook | 却下 | 運用の属人化 |

### 採用理由 (Argo Events)

1. **Argo CD と同一エコシステムで統合コスト低** — Argoproj の統一運用体験
2. **GitHub Webhook → Argo CD 即時同期** — ポーリング 3 分待ちの解消
3. **Harbor スキャン結果の自動通知** — 脆弱性検知から通知までを自動化
4. **個人情報削除フローの自動化** — Keycloak ユーザー無効化 → tier2 各サービスへの削除イベント配信
5. **Litmus Chaos 結果の自動連携** — カオス実験結果のフィードバックループ構築

### MVP スコープ

- Phase 2 で導入 (Kafka 導入と同時)
- `operation` namespace にデプロイ
- EventSource: GitHub Webhook / Harbor Webhook / Kafka
- Sensor: Argo CD 即時同期 / Alertmanager 通知

### トレードオフ

- Phase 1 では不要 (イベントソースとなる Kafka がまだない)
- Phase 2 の Kafka / Istio 導入と同時に構築

---

## 関連ドキュメント

- [`02_周辺OSS.md`](./02_周辺OSS.md) — 周辺 OSS の選定 (A〜K)
- [`04_選定一覧.md`](./04_選定一覧.md) — 採用 OSS 選定一覧
- [`../02_アーキテクチャ/10_レート制限.md`](../02_アーキテクチャ/10_レート制限.md) — Rate Limiting
- [`../02_アーキテクチャ/05_障害復旧とバックアップ.md`](../02_アーキテクチャ/05_障害復旧とバックアップ.md) — Longhorn によるデータ保護
- [`../02_アーキテクチャ/09_データアーキテクチャ.md`](../02_アーキテクチャ/09_データアーキテクチャ.md) — PgBouncer による接続管理
- [`../05_CICDと配信/00_CICDパイプライン.md`](../05_CICDと配信/00_CICDパイプライン.md) — Renovate / Argo Events のパイプライン統合
