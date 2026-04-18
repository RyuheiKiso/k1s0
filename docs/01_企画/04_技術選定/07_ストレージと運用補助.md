# 追加採用 OSS の選定

## 目的

[`02_周辺OSS.md`](./02_周辺OSS.md) (A〜K) に続き、追加で採用が決定した OSS の選定根拠を整理する。対象カテゴリは以下のとおり。

- L. 分散ストレージ
- M. 接続プーリング
- N. 依存パッケージ自動更新
- O. イベント駆動自動化
- P. 統合テスト基盤
- Q. DB スキーママイグレーション (Go)
- R. DB スキーママイグレーション (Rust)
- S. ベクトル検索拡張 (PostgreSQL)

---

## L. 分散ストレージ

| 候補 | 採否 | 評価 |
|---|---|---|
| **Longhorn** | 採用 | CNCF Incubating。SUSE 開発で実績豊富。UI 管理でレプリケーション / スナップショット / バックアップを提供。kubeadm / k3s いずれの k8s ディストリビューションでも動作 |
| Rook-Ceph | 却下 | 運用負荷が JTC 2 名体制に対して過大 |
| ローカル PV | 却下 | ノード障害時にデータ消失 |

### 採用理由 (Longhorn)

1. **SUSE 開発で実績豊富** — CNCF Incubating プロジェクトとしてコミュニティも活発。kubeadm / k3s いずれの環境でも動作
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

## P. 統合テスト基盤

| 候補 | 採否 | 評価 |
|---|---|---|
| **Testcontainers** | 採用 | AtomicJar (Docker 社傘下)。Go / Rust / C# / TS の全言語に公式 or 準公式ライブラリ。テストコード内からコンテナを宣言的に起動・破棄 |
| Docker Compose + テストスクリプト | 却下 | テストコードとインフラ定義が分離し、テスト実行手順が属人化する |
| k8s test namespace に手動デプロイ | 却下 | フィードバックが遅い (デプロイ待ち)。ローカル実行不可 |
| in-memory 実装のみ | 却下 | Dapr in-memory backend は軽量統合テストに有用だが、PostgreSQL / Valkey / Kafka の実挙動との乖離を検出できない |

### 採用理由 (Testcontainers)

1. **テストコード内でコンテナのライフサイクルを完結** — テスト開始時に PostgreSQL / Valkey / Kafka コンテナを自動起動し、テスト終了後に自動破棄。外部のテスト環境管理が不要
2. **ローカルと CI で同一テストが実行可能** — 開発者 PC (Docker Desktop / Rancher Desktop) でも GHA runner Pod (DinD sidecar) でも同じテストコードが動く
3. **k1s0 の全言語をカバー** — Go (`testcontainers-go`)、Rust (`testcontainers-rs`)、C# (`Testcontainers for .NET`)、TypeScript (`testcontainers-node`) の公式 or コミュニティライブラリが存在
4. **in-memory バックエンドでは検出できない不具合を捕捉** — PostgreSQL の制約違反、Valkey の TTL 挙動、Kafka のパーティション分散など、実バックエンドでのみ再現する問題を CI で早期検出
5. **k8s test namespace への依存を削減** — 統合テストの大半をコンテナベースで実行できるため、k8s へのデプロイが不要になりフィードバックループが短縮される

### Tilt との棲み分け

| ツール | 目的 | 対象 | k8s 依存 |
|---|---|---|---|
| Tilt | 開発中の即時確認・探索的開発 | 開発者がコード変更を手元で確認 | あり (ローカル k8s) |
| Testcontainers | 自動化された統合テスト | CI + ローカルで実行される再現可能なテスト | なし (Docker のみ) |

Tilt は「人間が画面を見て確認する」探索的開発を高速化する。Testcontainers は「機械が自動で検証する」統合テストを高速化する。両者は補完関係にあり、競合しない。

### CI 環境での実行

GHA self-hosted runner Pod で Testcontainers を実行するには、Docker ソケットへのアクセスが必要。以下のいずれかの方式で対応する。

| 方式 | 評価 |
|---|---|
| DinD (Docker-in-Docker) sidecar | 推奨。runner Pod に DinD sidecar を追加し、Testcontainers がコンテナを起動する。セキュリティ境界が明確 |
| ホスト Docker ソケットマウント | 次点。パフォーマンスは良いが、ホストへのアクセス権限が広がるためセキュリティリスクがある |

MVP-1a の runner イメージ (Kaniko / Trivy / crane 同梱) に DinD sidecar を追加する。

### MVP スコープ

- Phase 1 (MVP-1a) から導入
- tier1 Go サービスの PostgreSQL 統合テストで初適用
- GHA runner Pod に DinD sidecar を追加
- 雛形生成 CLI が Testcontainers のテストテンプレートを生成

### トレードオフ

- テスト実行時間がユニットテストより長い (コンテナ起動に 3〜10 秒)。ユニットテストの代替ではなく補完として位置付ける
- DinD sidecar のメモリ消費 (約 0.5 GB) が GHA runner Pod に追加される
- Testcontainers for Rust (`testcontainers-rs`) はコミュニティ管理であり、他言語と比較して成熟度がやや低い。ただし Rust テストの実行には十分な機能を持つ

---

## Q. DB スキーママイグレーション (Go)

| 候補 | 採否 | 評価 |
|---|---|---|
| **golang-migrate** | 採用 | Go エコシステム標準。SQL ベースのマイグレーションで宣言的に管理。CLI + ライブラリの 2 形態で利用可能 |
| goose | 却下 | 機能的には十分だがコミュニティ規模と GitHub Star 数で golang-migrate に劣る |
| Atlas | 却下 | 宣言的スキーマ管理は魅力的だが商用モデル (Ariga 社) への依存リスク |

### 採用理由 (golang-migrate)

1. **SQL ベースのマイグレーション** — Up / Down の SQL ファイルを `migrations/` ディレクトリに格納し、バージョン番号で順序管理する。ORM に依存しないため、tier1 Go サービスと tier2 Go サービスの双方で統一的に利用可能
2. **CLI + ライブラリの 2 形態** — CI パイプラインでは CLI (`migrate` コマンド) を使用し、アプリケーション起動時にはライブラリとして組み込むことも可能。Argo CD PreSync Hook での実行にも対応
3. **PostgreSQL ドライバ対応** — `pgx` / `lib/pq` の双方をサポート。CloudNativePG + PgBouncer 構成でも動作確認済みのコミュニティ事例が豊富
4. **Testcontainers との統合** — Testcontainers で起動した PostgreSQL コンテナに対してマイグレーションを適用し、統合テストでスキーマの正しさを自動検証できる
5. **CI でのマイグレーション検証** — 空の PostgreSQL コンテナに対して全マイグレーションを適用し、構文エラーや互換性違反を PR 時に検出する

### 実行方式

| タイミング | 実行方式 | 環境 |
|---|---|---|
| CI (PR 時) | GHA step で `migrate` CLI を実行。Testcontainers の PostgreSQL に全マイグレーション適用 | GHA runner Pod |
| デプロイ時 | Argo CD PreSync Hook (Job) で `migrate up` を実行 | k8s |
| ローカル開発 | `migrate` CLI を直接実行、または Tilt の `local_resource` で自動適用 | 開発者 PC |

### MVP スコープ

- Phase 1 (MVP-1a) から導入
- tier1 Go サービス (`k1s0_audit` DB) のマイグレーションで初適用
- 雛形生成 CLI がマイグレーションディレクトリ構造と初期マイグレーションファイルを自動生成
- `down` マイグレーション (ロールバック SQL) の作成を必須とする

### トレードオフ

- マイグレーションファイルの手書き管理が必要 (ORM 自動生成ではない) だが、SQL を直接管理する方が長期的な可読性と制御性が高い
- マイグレーション順序の衝突 (複数開発者が同時にマイグレーション追加) は CI で検出する

---

## R. DB スキーママイグレーション (Rust)

| 候補 | 採否 | 評価 |
|---|---|---|
| **sqlx-cli** | 採用 | Rust エコシステム標準。コンパイル時 SQL 検証と統合されたマイグレーション管理 |
| diesel_cli | 却下 | ORM (Diesel) と密結合。sqlx の方が軽量で SQL 直書きとの親和性が高い |
| refinery | 却下 | 機能は十分だがコミュニティ規模が sqlx に劣る |

### 採用理由 (sqlx-cli)

1. **コンパイル時 SQL 検証** — `sqlx::query!` マクロが実行する SQL をコンパイル時に PostgreSQL に対して検証する。型の不一致やテーブル/カラムの不存在をビルド時に検出でき、ランタイムエラーを未然に防止する
2. **SQL ベースのマイグレーション** — golang-migrate と同様に Up / Down の SQL ファイルで管理する。ORM に依存しないため、監査ログの複雑なスキーマも SQL で直接定義できる
3. **tier1 Rust サービスとの統合** — `sqlx` は tier1 Rust サービス (監査ログ・PII マスキング) の PostgreSQL クライアントとしても採用するため、マイグレーションツールとアプリケーションコードが同一エコシステムで統一される
4. **Testcontainers との統合** — `testcontainers-rs` で起動した PostgreSQL に対してマイグレーションを適用し、監査ログ永続化のテストを自動化できる

### 実行方式

| タイミング | 実行方式 | 環境 |
|---|---|---|
| CI (PR 時) | GHA step で `sqlx migrate run` を実行。Testcontainers の PostgreSQL でスキーマ検証 | GHA runner Pod |
| デプロイ時 | Argo CD PreSync Hook (Job) で `sqlx migrate run` を実行 | k8s |
| ローカル開発 | `sqlx migrate run` を直接実行、または Tilt の `local_resource` で自動適用 | 開発者 PC |
| コンパイル時 | `sqlx::query!` マクロが `.sqlx/` キャッシュを参照してオフラインでも検証可能 | GHA runner Pod / 開発者 PC |

### sqlx オフラインモード

CI 環境や DB 未接続の開発環境では、`cargo sqlx prepare` で生成した `.sqlx/` ディレクトリ (クエリメタデータのキャッシュ) をリポジトリにコミットすることで、PostgreSQL に接続せずにコンパイル時検証を実行できる。

### MVP スコープ

- Phase 2 から導入 (tier1 Rust サービスの本格実装と同時)
- tier1 Rust サービス (`k1s0_audit` DB の監査テーブル) のマイグレーションで初適用
- `down` マイグレーション (ロールバック SQL) の作成を必須とする
- `.sqlx/` ディレクトリをリポジトリにコミットし、オフラインコンパイル時検証を有効化

### トレードオフ

- `cargo sqlx prepare` の実行を忘れるとオフラインビルドが失敗する → CI で `.sqlx/` の最新性を検証する step を追加
- sqlx-cli のバージョンと sqlx クレートのバージョンを一致させる必要がある → Renovate で統一管理

---

## S. ベクトル検索拡張 (PostgreSQL)

| 候補 | 採否 | 評価 |
|---|---|---|
| **pgvector** | 採用 | PostgreSQL 拡張。既存の CloudNativePG クラスタに追加するだけでベクトル検索が利用可能。追加インフラ不要 |
| Weaviate | 却下 | 専用のベクトル DB サーバーが必要。インフラ追加のコストと運用負荷が大きい |
| Qdrant | 却下 | 同上。Rust 製で高性能だが 2 名体制で別 DB を運用する余裕がない |
| Milvus | 却下 | 大規模向け。JTC の利用規模にはオーバースペックでリソース消費が大きい |
| ベクトル検索なし | 却下 | Phase 4 以降の AI/ML 連携 (RAG / セマンティック検索) で必要になる |

### 採用理由 (pgvector)

1. **既存インフラへの追加のみ** — CloudNativePG の PostgreSQL インスタンスに `CREATE EXTENSION vector;` を実行するだけで利用可能。追加の DB サーバー・Operator・ストレージが不要
2. **PostgreSQL エコシステムとの統合** — `WHERE` 句のフィルタリング、`JOIN`、トランザクションなど PostgreSQL の全機能とベクトル検索を組み合わせられる。ACID 保証の中でベクトルデータを管理できる
3. **バックアップ戦略の統合** — CloudNativePG の barman-cloud バックアップにベクトルデータも含まれる。専用ベクトル DB の場合は別途バックアップ戦略が必要
4. **将来の AI/ML ユースケース** — 社内文書のセマンティック検索、ナレッジベース構築、RAG (Retrieval-Augmented Generation) のバックエンドとして利用可能。tier1 公開 API として `k1s0.Search.*` を将来追加する際の基盤技術となる
5. **接続プーリングとの共存** — PgBouncer 経由でのベクトル検索クエリ実行に問題がない

### インデックス方式

| インデックス | 特徴 | 推奨用途 |
|---|---|---|
| IVFFlat | 構築が高速。精度はやや劣る | データ量が少ない初期段階 (Phase 2-3) |
| HNSW | 検索精度が高い。構築にメモリを消費する | データ量が増えた Phase 4 以降 |

### MVP スコープ

- Phase 2 で PostgreSQL 拡張として有効化 (CloudNativePG の `postgresql.conf` に追加)
- Phase 2 時点では拡張の有効化のみ。実際のベクトルデータ格納は Phase 4 以降
- tier1 Rust サービスでの利用を想定 (sqlx は pgvector 型をサポート)

### トレードオフ

- 大量のベクトルデータ (100 万件以上) を扱う場合は HNSW インデックスのメモリ消費が大きくなる → Phase 4 以降の実データ量に応じてインデックス戦略を再評価
- PostgreSQL のメジャーバージョンアップ時に pgvector の互換性を確認する必要がある → Renovate の対象に含める (CloudNativePG イメージに pgvector が同梱されるコミュニティイメージを利用)

---

## 関連ドキュメント

- [`02_周辺OSS.md`](./02_周辺OSS.md) — 周辺 OSS の選定 (A〜K)
- [`04_選定一覧.md`](./04_選定一覧.md) — 採用 OSS 選定一覧
- [`../02_アーキテクチャ/10_レート制限.md`](../02_アーキテクチャ/10_レート制限.md) — Rate Limiting
- [`../02_アーキテクチャ/05_障害復旧とバックアップ.md`](../02_アーキテクチャ/05_障害復旧とバックアップ.md) — Longhorn によるデータ保護
- [`../02_アーキテクチャ/09_データアーキテクチャ.md`](../02_アーキテクチャ/09_データアーキテクチャ.md) — PgBouncer による接続管理 / スキーマ進化戦略
- [`../05_CICDと配信/00_CICDパイプライン.md`](../05_CICDと配信/00_CICDパイプライン.md) — Renovate / Argo Events のパイプライン統合
- [`../05_CICDと配信/03_テスト戦略.md`](../05_CICDと配信/03_テスト戦略.md) — Testcontainers のテスト戦略への統合
