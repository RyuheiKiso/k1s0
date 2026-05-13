---
id: env.ops.ops_index
axis: ops
phase: env_setup
kind: index
status: draft
depends_on:
  - env.overview.environment_setup_index
  - plan.development_team_structure
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# ops 軸エンジニア 環境構築 index

## 一文方針

- ops 軸エンジニアは SRE 50% rule 徹底 / SLO dashboard 管理 / on-call rotation 設計 / postmortem 主導 / Backstage TechDocs 管理の 5 責務を持ち、Argo Rollouts / Grafana / Prometheus / k6 を手元で完全起動できることが環境構築の検収条件である。

## 至高路線における立ち位置

- ops 軸の環境は「監視ツールが見れる状態」ではなく「SLO error budget を計算し runbook を即座に参照できる状態」である。ローカル docker compose で Grafana + Prometheus + Backstage が全て green になってから実装支援に入る。
- 14 ページは ops 責務の 14 側面に 1:1 対応し、それぞれが独立した検収コマンドを持つ。一括ではなくページ単位で green を確認してから次ページへ進む。
- 段階的セットアップ禁止: 03_必須ランタイムが終わった直後に 04_リポジトリ取得を実行し、05_主要OSS導入が green になるまで次に進まない。

## 配下ドキュメント

| 番号 | ドキュメント | 主題 | kind |
|---|---|---|---|
| 01 | [01_責務とスコープ](01_責務とスコープ.md) | ops ロールの責務と非責務 | responsibility |
| 02 | [02_前提OS環境](02_前提OS環境.md) | WSL2 / Ubuntu / Docker Engine | policy |
| 03 | [03_必須ランタイム](03_必須ランタイム.md) | kubectl / helm / grafana-cli / k6 | policy |
| 04 | [04_リポジトリ取得手順](04_リポジトリ取得手順.md) | clone / baseline lint green 確認 | enforcement |
| 05 | [05_主要OSS導入](05_主要OSS導入.md) | Argo Rollouts / Tekton / Backstage / Grafana / Prometheus | enforcement |
| 06 | [06_開発エディタIDE設定](06_開発エディタIDE設定.md) | VS Code + Grafana extension + YAML extension | enforcement |
| 07 | [07_テスト検証環境](07_テスト検証環境.md) | docker compose 起動 / k6 ローカル実行 | enforcement |
| 08 | [08_lintとformat適用](08_lintとformat適用.md) | techdocs-cli / promtool / k6 ESLint | enforcement |
| 09 | [09_docs_lint実行手順](09_docs_lint実行手順.md) | run_lint.sh + run_lint.py | enforcement |
| 10 | [10_frontmatter規約適用](10_frontmatter規約適用.md) | id 導出 / 7 required / 8 forbidden | convention |
| 11 | [11_軸固有環境設定](11_軸固有環境設定.md) | software template / on-call / SLO error budget | enforcement |
| 12 | [12_CI完全再現](12_CI完全再現.md) | docs_lint.yml 手元再現 | enforcement |
| 13 | [13_検収基準](13_検収基準.md) | Backstage + Grafana + Prometheus 起動 / k6 応答 | enforcement |
| 14 | [14_Claude_Code連携](14_Claude_Code連携.md) | CLAUDE.md / memory / skills | policy |

## 検収条件

以下を全て満たすまで「ops 軸エンジニアの環境構築完了」とはならない。

1. `bash tools/docs_lint/run_lint.sh` → exit 0
2. `python3 tools/docs_lint/run_lint.py` → `8 check 全 green`
3. `docker compose up -d` （Backstage + Grafana + Prometheus）が全コンテナ Up
4. `k6 run --version` が応答する
5. `kubectl version --client` が応答する

## 上位フェーズとの bind

### 上位フェーズへの依存
- [05_環境構築 phase index](../README.md): 共通前提（OS / git / bash）を宣言
- [08_開発体制](../../01_企画/08_開発体制/README.md): ops 軸エンジニアロール定義の source of truth

## 読み筋

- 初読: 本ファイル → 01_責務とスコープ → 02 → 03 → 04 と順に実行（各ページの検収コマンドを green にしてから次へ）
- 障害対応時参照: 12_CI完全再現 → 09_docs_lint実行手順 → 07_テスト検証環境

## 関連参照

- [05_環境構築 index](../README.md)
- [docs/00_format/README.md](../../00_format/README.md)
- [docs/01_企画/08_開発体制/README.md](../../01_企画/08_開発体制/README.md)
