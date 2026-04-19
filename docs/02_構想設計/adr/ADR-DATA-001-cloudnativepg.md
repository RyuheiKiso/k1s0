# ADR-DATA-001: リレーショナル DB に CloudNativePG を採用

- ステータス: Accepted
- 起票日: 2026-04-19
- 決定日: 2026-04-19
- 起票者: kiso ryuhei
- 関係者: システム基盤チーム / データ基盤チーム / 運用チーム

## コンテキスト

tier1 の State API（Relational Store）、Temporal のワークフロー永続化、Keycloak/Grafana/Backstage 等の裏側 RDBMS、監査ログの WORM 保管など、k1s0 全体で PostgreSQL 系 RDBMS への依存が多数発生する。これらをどう運用するかで、可用性（NFR-A-CONT-001: RPO 秒オーダー）、運用工数（NFR-C-NOP-001）、法令対応（NFR-G-RES-001: 国内保管）に直接影響する。

制約条件は以下の通り。

- オンプレミス完結（NFR-F-SYS-001）でクラウドマネージド（RDS、Cloud SQL）は選択肢外
- 2 名運用チームで回せる運用密度（NFR-C-NOP-001）
- RPO 秒オーダー、RTO 4 時間（NFR-A-CONT-001）
- バックアップとレプリケーションは自動化必須（NFR-C-NOP-002）
- Kubernetes 上で他コンポーネントと統合的に運用したい（GitOps、Argo CD ベース）

手動運用の PostgreSQL（VM 上に pacemaker 等で HA）は運用工数が過大。Postgres Operator 系の OSS を比較した結果、CloudNativePG が最も JTC 要件に適合する。

## 決定

**リレーショナル DB は CloudNativePG（PostgreSQL Operator、CNCF Sandbox）で運用する。**

- PostgreSQL 16 LTS を基本、セキュリティパッチのみ自動適用
- Streaming Replication による Primary 1 + Standby 2 構成を標準
- Backup は MinIO（ADR-DATA-003）へ barman cloud で圧縮 + 暗号化保管
- テナントごとに独立 DB（論理分離、tenant_id で物理テーブル内 partition）、大規模テナントは独立 Cluster（物理分離）へ昇格可
- PITR（Point-in-Time Recovery）対応、RPO 秒オーダー達成
- Prometheus Exporter で SLI 計測、Grafana Mimir に集約

## 検討した選択肢

### 選択肢 A: CloudNativePG（採用）

- 概要: EDB 発、CNCF Sandbox の PostgreSQL Operator。Kubernetes ネイティブで Primary/Standby 管理
- メリット:
  - Apache 2.0 ライセンス、商用サポートも EDB から取得可能
  - Barman 統合で PITR / 継続的バックアップが標準
  - Prometheus メトリクス、Grafana ダッシュボード公式提供
  - Major Version upgrade の手順が Operator で体系化
  - K8s Resource（Cluster / ScheduledBackup / Pooler）で宣言的管理、GitOps 親和性高
- デメリット:
  - Operator 自体のバグや変更破壊リスクに備える必要（Operator バージョン固定方針）
  - Postgres の扱いを Operator 経由に集約するため、Operator 障害時の影響範囲が広い

### 選択肢 B: Zalando Postgres Operator

- 概要: Zalando 社の Postgres Operator、2017 年から運用実績多い
- メリット: 歴史が長く、大規模事例あり
- デメリット:
  - HA は Patroni 前提、Cluster API の抽象度が CNPG より低い
  - メンテナンスが散発的、2023〜2024 年のコミット頻度が低い時期があった

### 選択肢 C: Crunchy Postgres Operator

- 概要: Crunchy Data 社の商用寄り Operator
- メリット: 商用サポート充実、MLH-grade 金融・医療案件の実績
- デメリット:
  - 商用ライセンス（PGO for Kubernetes）が一部必要
  - JTC のコスト削減目標（BC-COST-003）と逆行

### 選択肢 D: VM 上の素の PostgreSQL + pacemaker/corosync

- 概要: Kubernetes を使わず VM 上で従来型 HA
- メリット: 運用ノウハウが豊富、障害事例が公開されている
- デメリット:
  - Kubernetes 上の他コンポーネントとの統合（GitOps、サービスメッシュ、観測性）ができない
  - 運用工数が 2 名で破綻

## 帰結

### ポジティブな帰結

- Operator ベースで宣言的運用、Argo CD との統合で変更の証跡保全
- PITR 対応で RPO 秒オーダー達成、BC-ONB-003 の 99.9% SLA 契約に貢献
- テナント追加時の DB プロビジョニングが自動化可能（BC-ONB-003）
- 商用サポート（EDB）を必要時に取得できる保険

### ネガティブな帰結

- CloudNativePG は CNCF Sandbox で Incubation/Graduated より成熟度は低い。四半期単位の安定版監視が必要
- Operator 障害時の影響範囲が広い（全 Cluster に影響しうる）ため、Operator バージョン固定と staging 先行適用を徹底
- barman バックアップの復元手順を定期的に訓練する必要（OR-INC-006）

## 実装タスク

- CloudNativePG Operator の Helm Chart バージョンを Argo CD の ApplicationSet で統一管理
- Cluster マニフェストのテンプレートを Backstage Software Template 化（BC-ONB-003）
- barman クラウド設定（MinIO 接続、暗号化、保管期間）を共通 ConfigMap 化
- PITR 復元訓練を四半期で実施（OR-INC-006）
- Major Version upgrade 手順書を Runbook 化（NFR-C-NOP-003）

## 参考文献

- CloudNativePG 公式: cloudnative-pg.io
- CNCF Sandbox Projects
- PostgreSQL 16 Release Notes
- Barman: pgbarman.org
