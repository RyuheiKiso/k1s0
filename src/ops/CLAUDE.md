# ops コーディングポリシー

全軸共通ルールは `src/CLAUDE.md` を参照。本ファイルは ops 固有の制約のみ記述する。

## 配置・構成

- **ops ループ**: `src/ops/ops_loop/`（signal_classes.yaml + ops_loop.lock.yaml）
- **escalation engine**: Argo Workflows + Mattermost（`src/ops/escalation_engine/`）
- **lock.yaml 配置先**: `src/ops/lock/`（手書き禁止）
- **runbook**: `src/ops/runbook_catalog/`（.md ファイルは ops 運用文書として許可）

設計パターン・モジュール構成の詳細は `docs/03_概要設計/08_ops設計方針/README.md` を単一の真とする。

## コーディング制約

### cluster 上の操作制限

- production cluster での chaos / scenario / load test 禁止（shadow cluster か Testcontainers のみ）
- ops_edge_cluster は独立 K8s クラスタ（`src/ops/ops_edge_cluster/manifests/` に escalation trigger config のみ配置）
- K8s 基盤 manifests は `src/_crosscutting/10_ops_edge_cluster/manifests/` が管理

### ops ループ

- ops ループは build artifact（`ops_loop.lock.yaml`）で管理
- `signal_class` / `phase` の変更は dual sign-off 必須
- Argo Workflow の step で runbook 手順をコード化（人間の手順書への依存を減らす）

### Perses dashboard

- dashboard は `src/ops/perses/` に JSON 定義
- Perses API から export した JSON を git commit（手動 GUI 編集での状態管理禁止）

### alert catalog

- alert rule は `src/ops/alert_catalog/` に配置
- SLO breach alert は `src/tier1/slo/` の SLO class と連動させる

## 関連参照

- `docs/03_概要設計/08_ops設計方針/README.md` — 設計パターン
- `docs/04_詳細設計/02_強制機構/07_ops強制機構.md` — CI fail 条件の詳細
- `docs/04_詳細設計/01_適合仕様/17_運用ループ適合仕様.md` — 運用ループ
