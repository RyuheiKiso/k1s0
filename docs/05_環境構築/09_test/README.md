---
id: env.test.test_index
axis: test
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

# test 軸エンジニア 環境構築 index

## 一文方針

- test 軸エンジニアは 5 verification_class × 19 軸 = 95 cell coverage matrix 管理 / mutation score 維持 / contract test / chaos test / property-based test / scenario replay の 6 責務を持ち、pytest / Vitest / Pact broker / Litmus / cargo-mutants が手元で動くことが環境構築の検収条件である。

## 至高路線における立ち位置

- test 軸の環境は「テストが書ける状態」ではなく「95 cell coverage matrix を局所生成して mutation score を計測できる状態」を到達目標とする。Pact broker + Litmus chaos hub が全て起動するまで「test 軸環境が整った」とは言わない。
- 14 ページは test 責務の 14 側面に 1:1 対応し、それぞれが独立した検収コマンドを持つ。一括ではなくページ単位で green を確認してから次ページへ進む。
- 段階的セットアップ禁止: 全言語ランタイム（Python / Node / Java / Rust）が揃ってから 05_主要OSS導入に進む。

## 配下ドキュメント

| 番号 | ドキュメント | 主題 | kind |
|---|---|---|---|
| 01 | [01_責務とスコープ](01_責務とスコープ.md) | test ロールの責務と非責務 | responsibility |
| 02 | [02_前提OS環境](02_前提OS環境.md) | WSL2 / Ubuntu / Docker Engine | policy |
| 03 | [03_必須ランタイム](03_必須ランタイム.md) | Python / Node / Java / Rust + Docker Compose | policy |
| 04 | [04_リポジトリ取得手順](04_リポジトリ取得手順.md) | clone / baseline lint green 確認 | enforcement |
| 05 | [05_主要OSS導入](05_主要OSS導入.md) | pytest / Vitest / Pact / Litmus / pitest / Stryker | enforcement |
| 06 | [06_開発エディタIDE設定](06_開発エディタIDE設定.md) | VS Code + Python / Java / Playwright extension | enforcement |
| 07 | [07_テスト検証環境](07_テスト検証環境.md) | Pact broker / Litmus chaos hub / mutation / contract | enforcement |
| 08 | [08_lintとformat適用](08_lintとformat適用.md) | Playwright ESLint / ruff / mutation threshold | enforcement |
| 09 | [09_docs_lint実行手順](09_docs_lint実行手順.md) | run_lint.sh + run_lint.py | enforcement |
| 10 | [10_frontmatter規約適用](10_frontmatter規約適用.md) | id 導出 / 7 required / 8 forbidden | convention |
| 11 | [11_軸固有環境設定](11_軸固有環境設定.md) | coverage matrix 生成 / mutation 演習 / Pact broker 起動 | enforcement |
| 12 | [12_CI完全再現](12_CI完全再現.md) | docs_lint.yml 手元再現 | enforcement |
| 13 | [13_検収基準](13_検収基準.md) | pytest / vitest / cargo-mutants / Pact / verification_class | enforcement |
| 14 | [14_Claude_Code連携](14_Claude_Code連携.md) | CLAUDE.md / memory / skills | policy |

## 検収条件

以下を全て満たすまで「test 軸エンジニアの環境構築完了」とはならない。

1. `bash tools/docs_lint/run_lint.sh` → exit 0
2. `python3 tools/docs_lint/run_lint.py` → `8 check 全 green`
3. `pytest --version` が応答する
4. `pnpm vitest --version` が応答する
5. `cargo-mutants --version` が応答する
6. Pact broker ローカル起動成功
7. 5 verification_class の sample 実行完了

## 上位フェーズとの bind

### 上位フェーズへの依存
- [05_環境構築 phase index](../README.md): 共通前提（OS / git / bash）を宣言
- [08_開発体制](../../01_企画/08_開発体制/README.md): test 軸エンジニアロール定義の source of truth

## 読み筋

- 初読: 本ファイル → 01_責務とスコープ → 02 → 03 → 04 と順に実行（各ページの検収コマンドを green にしてから次へ）
- 障害対応時参照: 12_CI完全再現 → 09_docs_lint実行手順 → 07_テスト検証環境

## 関連参照

- [05_環境構築 index](../README.md)
- [docs/00_format/README.md](../../00_format/README.md)
- [docs/01_企画/08_開発体制/README.md](../../01_企画/08_開発体制/README.md)
