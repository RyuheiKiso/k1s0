---
id: env.tier1.tier1_index
axis: tier1
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

# tier1 軸エンジニア 環境構築 index

## 一文方針

- tier1 軸エンジニアは Rust / Go / C# / TypeScript / proto / Buf の 4 言語 + スキーマツールチェーンを手元で完全動作させ、`cargo --version` / `go version` / `dotnet --version` / `pnpm --version` / `buf --version` 全応答 + docs_lint green を検収条件とする。

## 至高路線における立ち位置

- tier1 の環境は「コンパイルできる状態」ではなく「全言語の lint / test / proto stub 生成が CI と同等に再現できる状態」である。
- 14 ページは責務の 14 側面に 1:1 対応し、各ページが独立した検収コマンドを持つ。ページ単位で green を確認してから次ページへ進む。
- 段階的セットアップ禁止: 03_必須ランタイムが完了した直後に 04_リポジトリ取得を実行し、07_テスト検証環境が green になるまで 08 以降に進まない。

## 配下ドキュメント

| 番号 | ドキュメント | 主題 | kind |
|---|---|---|---|
| 01 | [01_責務とスコープ](01_責務とスコープ.md) | tier1 ロールの責務と非責務 | responsibility |
| 02 | [02_前提OS環境](02_前提OS環境.md) | WSL2 / Ubuntu / bash | policy |
| 03 | [03_必須ランタイム](03_必須ランタイム.md) | Rust / Go / .NET 8 / Node 20 + pnpm | policy |
| 04 | [04_リポジトリ取得手順](04_リポジトリ取得手順.md) | clone / baseline lint green 確認 | enforcement |
| 05 | [05_主要OSS導入](05_主要OSS導入.md) | Buf / protoc / Apicurio / ts-proto | enforcement |
| 06 | [06_開発エディタIDE設定](06_開発エディタIDE設定.md) | VS Code + 各言語 extension | enforcement |
| 07 | [07_テスト検証環境](07_テスト検証環境.md) | 全言語ローカル test 実行 | enforcement |
| 08 | [08_lintとformat適用](08_lintとformat適用.md) | clippy / fmt / vet / eslint | enforcement |
| 09 | [09_docs_lint実行手順](09_docs_lint実行手順.md) | run_lint.sh 7 check + run_lint.py 8 check | enforcement |
| 10 | [10_frontmatter規約適用](10_frontmatter規約適用.md) | id 導出 / 7 required / 8 forbidden | convention |
| 11 | [11_軸固有環境設定](11_軸固有環境設定.md) | buf generate 演習 / 9 適合仕様確認 | enforcement |
| 12 | [12_CI完全再現](12_CI完全再現.md) | docs_lint.yml 2 job の手元再現 | enforcement |
| 13 | [13_検収基準](13_検収基準.md) | 全バイナリ応答 + lint green | enforcement |
| 14 | [14_Claude_Code連携](14_Claude_Code連携.md) | CLAUDE.md / skills / memory | policy |

## 検収条件

以下を全て満たすまで「tier1 環境構築完了」とはならない。

1. `cargo --version` → stable バージョン応答
2. `go version` → 1.22 以上
3. `dotnet --version` → 8.x 応答
4. `pnpm --version` → 9.x 応答
5. `buf --version` → バージョン応答
6. `bash tools/docs_lint/run_lint.sh` → exit 0
7. `python3 tools/docs_lint/run_lint.py` → `8 check 全 green`

## 上位フェーズとの bind

### 上位フェーズへの依存
- [05_環境構築 phase index](../README.md): 共通前提（OS / git / bash）を宣言
- [08_開発体制](../../01_企画/08_開発体制/README.md): tier1 ロール定義の source of truth

## 読み筋

- 初読: 本ファイル → 01_責務とスコープ → 02 → 03 → 04 と順に実行（各ページの検収コマンドを green にしてから次へ）
- 障害対応時参照: 12_CI完全再現 → 09_docs_lint実行手順 → 07_テスト検証環境

## 関連参照

- [05_環境構築 index](../README.md)
- [docs/00_format/README.md](../../00_format/README.md)
- [docs/01_企画/08_開発体制/README.md](../../01_企画/08_開発体制/README.md)
