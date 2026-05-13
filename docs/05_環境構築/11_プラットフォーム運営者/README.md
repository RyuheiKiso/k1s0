---
id: env.ops.platform_operator_index
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

# プラットフォーム運営者 環境構築 index

## 一文方針

- プラットフォーム運営者は OpenBao CLI / Cosign / kubectl / ykman の 4 ツールを手元で動作させ、KEK shamir M-of-N ceremony（ローカル rehearsal）/ break-glass dry-run / audit hash chain replay の 3 演習を完走することを環境構築の検収条件とする。

## 至高路線における立ち位置

- プラットフォーム運営者の環境は「ツールが動く状態」ではなく「production ceremony を安全に執行できる状態」である。rehearsal を繰り返してオペレーション手順を体得し、break-glass 時にパニックなく動作できることが到達目標。
- 14 ページは責務の 14 側面に 1:1 対応し、各ページが独立した検収コマンドを持つ。ページ単位で green を確認してから次ページへ進む。
- 段階的セットアップ禁止: 03_必須ランタイムが完了した直後に 04_リポジトリ取得を実行し、11_軸固有環境設定で 3 演習が完走するまで 12 以降に進まない。
- production KEK 鍵は env_setup フェーズでは rehearsal 手順のみ記載する。実鍵は本番 ceremony で生成するため本ドキュメントのスコープ外とする。

## 配下ドキュメント

| 番号 | ドキュメント | 主題 | kind |
|---|---|---|---|
| 01 | [01_責務とスコープ](01_責務とスコープ.md) | production cluster 運用・KEK shamir・break-glass・audit | responsibility |
| 02 | [02_前提OS環境](02_前提OS環境.md) | WSL2 / Ubuntu 22.04 / Yubikey PIV | policy |
| 03 | [03_必須ランタイム](03_必須ランタイム.md) | OpenBao CLI / Cosign / kubectl / ykman | policy |
| 04 | [04_リポジトリ取得手順](04_リポジトリ取得手順.md) | clone / baseline lint green 確認 | enforcement |
| 05 | [05_主要OSS導入](05_主要OSS導入.md) | OpenBao / ykman / audit-trail-tool 導入手順 | enforcement |
| 06 | [06_開発エディタIDE設定](06_開発エディタIDE設定.md) | CLI 中心 / Yubikey manager GUI | enforcement |
| 07 | [07_テスト検証環境](07_テスト検証環境.md) | shamir assembly 演習 / break-glass dry-run / audit replay | enforcement |
| 08 | [08_lintとformat適用](08_lintとformat適用.md) | OPA conftest / cosign verify 自動化 | enforcement |
| 09 | [09_docs_lint実行手順](09_docs_lint実行手順.md) | run_lint.sh / run_lint.py の手元実行 | enforcement |
| 10 | [10_frontmatter規約適用](10_frontmatter規約適用.md) | id 導出 / 7 required / 8 forbidden | convention |
| 11 | [11_軸固有環境設定](11_軸固有環境設定.md) | KEK rehearsal / break-glass dry-run / audit hash chain replay | enforcement |
| 12 | [12_CI完全再現](12_CI完全再現.md) | docs_lint.yml 2 job の手元再現 | enforcement |
| 13 | [13_検収基準](13_検収基準.md) | 4 ツール全応答 + 3 演習完走 | enforcement |
| 14 | [14_Claude_Code連携](14_Claude_Code連携.md) | LLM 補助の範囲 / ceremony は LLM 禁止 | policy |

## 検収条件

以下を全て満たすまで「プラットフォーム運営者の環境構築完了」とはならない。

1. `bao --version` が応答する
2. `ykman --version` が応答する
3. `kubectl version --client` が応答する
4. shamir assembly 演習（ローカル rehearsal）完了
5. break-glass dry-run 完了
6. audit hash chain replay 成功
7. `bash tools/docs_lint/run_lint.sh` → exit 0

## 上位フェーズとの bind

### 上位フェーズへの依存
- [05_環境構築 phase index](../README.md): 共通前提（OS / git / bash）を宣言
- [08_開発体制](../../01_企画/08_開発体制/README.md): プラットフォーム運営者ロール定義の source of truth

## 読み筋

- 初読: 本ファイル → 01_責務とスコープ → 02 → 03 → 04 と順に実行
- 障害対応時参照: 12_CI完全再現 → 09_docs_lint実行手順 → 07_テスト検証環境

## 関連参照

- [05_環境構築 index](../README.md)
- [docs/00_format/README.md](../../00_format/README.md)
- [docs/01_企画/08_開発体制/README.md](../../01_企画/08_開発体制/README.md)
