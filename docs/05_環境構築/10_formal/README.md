---
id: env.formal.formal_index
axis: formal
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

# formal 軸エンジニア 環境構築 index

## 一文方針

- formal 軸エンジニアは TLA+ / Apalache / Stainless / Dafny / Lean 4 / Kani / CBMC の 7 ツールを手元で動作させ、5 proof_class × 19 軸 = 95 cell の proof_matrix で sample proof 5 件が成功することを環境構築の検収条件とする。

## 至高路線における立ち位置

- formal の環境は「ツールが起動する状態」ではなく「counter-example を閉鎖できる状態」である。5 proof_class それぞれについて sample spec を手元で検証し、dual reviewer sign-off フローを確認してから CI と同等の検証を行う。
- 14 ページは責務の 14 側面に 1:1 対応し、各ページが独立した検収コマンドを持つ。ページ単位で green を確認してから次ページへ進む。
- 段階的セットアップ禁止: 03_必須ランタイムが完了した直後に 04_リポジトリ取得を実行し、11_軸固有環境設定で 5 proof_class が green になるまで 12 以降に進まない。

## 配下ドキュメント

| 番号 | ドキュメント | 主題 | kind |
|---|---|---|---|
| 01 | [01_責務とスコープ](01_責務とスコープ.md) | proof_matrix 管理・dual reviewer sign-off | responsibility |
| 02 | [02_前提OS環境](02_前提OS環境.md) | WSL2 / Ubuntu 22.04 / Java 21 / 16GB RAM | policy |
| 03 | [03_必須ランタイム](03_必須ランタイム.md) | TLA+ / Apalache / Dafny / Lean 4 / Kani / CBMC | policy |
| 04 | [04_リポジトリ取得手順](04_リポジトリ取得手順.md) | clone / baseline lint green 確認 | enforcement |
| 05 | [05_主要OSS導入](05_主要OSS導入.md) | 7 OSS のインストール手順 | enforcement |
| 06 | [06_開発エディタIDE設定](06_開発エディタIDE設定.md) | VS Code + TLA+ / Dafny / Lean 4 extension | enforcement |
| 07 | [07_テスト検証環境](07_テスト検証環境.md) | 5 proof_class sample proof 実行 | enforcement |
| 08 | [08_lintとformat適用](08_lintとformat適用.md) | tlc syntax / dafny /compile:0 / lean --check | enforcement |
| 09 | [09_docs_lint実行手順](09_docs_lint実行手順.md) | run_lint.sh / run_lint.py の手元実行 | enforcement |
| 10 | [10_frontmatter規約適用](10_frontmatter規約適用.md) | id 導出 / 7 required / 8 forbidden | convention |
| 11 | [11_軸固有環境設定](11_軸固有環境設定.md) | proof_matrix YAML 雛形 / 5 proof_class 実行 / dual sign-off 演習 | enforcement |
| 12 | [12_CI完全再現](12_CI完全再現.md) | docs_lint.yml 2 job の手元再現 | enforcement |
| 13 | [13_検収基準](13_検収基準.md) | 7 ツール全応答 + sample proof 5 件成功 | enforcement |
| 14 | [14_Claude_Code連携](14_Claude_Code連携.md) | LLM 補助の範囲 / dual sign-off 必須事項 | policy |

## 検収条件

以下を全て満たすまで「formal 軸エンジニアの環境構築完了」とはならない。

1. `tlc -help` が応答する
2. `apalache-mc help` が応答する
3. `dafny --version` が応答する
4. `lean --version` が応答する
5. `cargo kani --version` が応答する
6. `cbmc --version` が応答する
7. 5 proof_class の sample proof が全て成功する
8. `bash tools/docs_lint/run_lint.sh` → exit 0

## 上位フェーズとの bind

### 上位フェーズへの依存
- [05_環境構築 phase index](../README.md): 共通前提（OS / git / bash）を宣言
- [08_開発体制](../../01_企画/08_開発体制/README.md): formal 軸エンジニアロール定義の source of truth

## 読み筋

- 初読: 本ファイル → 01_責務とスコープ → 02 → 03 → 04 と順に実行（各ページの検収コマンドを green にしてから次へ）
- 障害対応時参照: 12_CI完全再現 → 09_docs_lint実行手順 → 07_テスト検証環境

## 関連参照

- [05_環境構築 index](../README.md)
- [docs/00_format/README.md](../../00_format/README.md)
- [docs/01_企画/08_開発体制/README.md](../../01_企画/08_開発体制/README.md)
