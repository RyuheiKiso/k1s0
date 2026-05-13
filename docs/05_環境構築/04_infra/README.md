---
id: env.infra.infra_index
axis: infra
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

# infra 軸エンジニア 環境構築 index

## 一文方針

- infra 軸エンジニアは kubectl / helm / kustomize / kind / Argo CD CLI / istioctl / calicoctl / OpenBao CLI / Cosign の全ツールを手元で動作させ、kind cluster 上で Kyverno 25+ policy と 5 topology_class が green になることを環境構築の検収条件とする。

## 至高路線における立ち位置

- infra の環境は「kubectl が動く状態」ではなく「k8s cluster を GitOps で完全管理できる状態」である。Argo CD bootstrap / Kyverno policy 適用 / Istio サービスメッシュ設定を手元 kind cluster で再現してから CI と同等の検証を行う。
- 14 ページは責務の 14 側面に 1:1 対応し、各ページが独立した検収コマンドを持つ。ページ単位で green を確認してから次ページへ進む。
- 段階的セットアップ禁止: 03_必須ランタイムが完了した直後に 04_リポジトリ取得を実行し、11_軸固有環境設定で kind cluster が green になるまで 12 以降に進まない。

## 配下ドキュメント

| 番号 | ドキュメント | 主題 | kind |
|---|---|---|---|
| 01 | [01_責務とスコープ](01_責務とスコープ.md) | infra ロールの責務と非責務 | responsibility |
| 02 | [02_前提OS環境](02_前提OS環境.md) | WSL2 / Ubuntu / Docker Engine | policy |
| 03 | [03_必須ランタイム](03_必須ランタイム.md) | kubectl / helm / kustomize / kind | policy |
| 04 | [04_リポジトリ取得手順](04_リポジトリ取得手順.md) | clone / baseline lint green 確認 | enforcement |
| 05 | [05_主要OSS導入](05_主要OSS導入.md) | Argo CD CLI / istioctl / calicoctl / OpenBao / Cosign / Litmus | enforcement |
| 06 | [06_開発エディタIDE設定](06_開発エディタIDE設定.md) | VS Code Kubernetes extension / Lens Desktop | enforcement |
| 07 | [07_テスト検証環境](07_テスト検証環境.md) | kind cluster / Kyverno admission test / Litmus chaos | enforcement |
| 08 | [08_lintとformat適用](08_lintとformat適用.md) | kubeconform / conftest / helm lint / kustomize build | enforcement |
| 09 | [09_docs_lint実行手順](09_docs_lint実行手順.md) | run_lint.sh 7 check + run_lint.py 8 check | enforcement |
| 10 | [10_frontmatter規約適用](10_frontmatter規約適用.md) | id 導出 / 7 required / 8 forbidden | convention |
| 11 | [11_軸固有環境設定](11_軸固有環境設定.md) | kind cluster 起動 / Argo CD bootstrap / Kyverno 25+ policy | enforcement |
| 12 | [12_CI完全再現](12_CI完全再現.md) | docs_lint.yml 2 job の手元再現 | enforcement |
| 13 | [13_検収基準](13_検収基準.md) | 全バイナリ応答 + kind cluster green | enforcement |
| 14 | [14_Claude_Code連携](14_Claude_Code連携.md) | CLAUDE.md / skills / memory | policy |

## 検収条件

以下を全て満たすまで「infra 環境構築完了」とはならない。

1. `kubectl version` → 1.30 以上応答
2. `helm version` → 3.15 以上応答
3. `kind version` → 0.23 以上応答
4. `istioctl version` → バージョン応答
5. `argocd version` → バージョン応答
6. kind cluster が Running 状態
7. `bash tools/docs_lint/run_lint.sh` → exit 0
8. `python3 tools/docs_lint/run_lint.py` → `8 check 全 green`

## 上位フェーズとの bind

### 上位フェーズへの依存
- [05_環境構築 phase index](../README.md): 共通前提（OS / git / bash）を宣言
- [08_開発体制](../../01_企画/08_開発体制/README.md): infra ロール定義の source of truth

## 読み筋

- 初読: 本ファイル → 01_責務とスコープ → 02 → 03 → 04 と順に実行（各ページの検収コマンドを green にしてから次へ）
- 障害対応時参照: 12_CI完全再現 → 09_docs_lint実行手順 → 07_テスト検証環境

## 関連参照

- [05_環境構築 index](../README.md)
- [docs/00_format/README.md](../../00_format/README.md)
- [docs/01_企画/08_開発体制/README.md](../../01_企画/08_開発体制/README.md)
