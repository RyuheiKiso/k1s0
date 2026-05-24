---
id: env.tier2.tier2_index
axis: tier2
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

# tier2 軸エンジニア 環境構築 index

## 一文方針

- tier2 軸エンジニアは 4 言語ランタイム + Docker で atomic 三表書込の 4 サービス（postgres / kafka / clickhouse / apicurio）を起動し、ドメイン bound CT 通過を検収条件とする。

## 至高路線における立ち位置

- tier2 の環境は「業務資産（Domain Event / Workflow / 決定表 / 業務マスタ / DB schema）を所有し、atomic 三表書込を手元で検証できる状態」である。
- 14 ページは責務の 14 側面に 1:1 対応し、各ページが独立した検収コマンドを持つ。ページ単位で green を確認してから次ページへ進む。
- 段階的セットアップ禁止: 07_テスト検証環境が完了するまで 08 以降に進まない。

## 配下ドキュメント

| 番号 | ドキュメント | 主題 | kind |
|---|---|---|---|
| 01 | [01_責務とスコープ](01_責務とスコープ.md) | tier2 ロールの責務と非責務 | responsibility |
| 02 | [02_前提OS環境](02_前提OS環境.md) | WSL2 / Ubuntu / bash / Docker | policy |
| 03 | [03_必須ランタイム](03_必須ランタイム.md) | 4 言語 + 業界 pack ビルドツール | policy |
| 04 | [04_リポジトリ取得手順](04_リポジトリ取得手順.md) | clone / baseline lint green 確認 | enforcement |
| 05 | [05_主要OSS導入](05_主要OSS導入.md) | ドメインイベント生成器 / 決定表 lint / workflow lint | enforcement |
| 06 | [06_開発エディタIDE設定](06_開発エディタIDE設定.md) | VS Code + 各言語 extension | enforcement |
| 07 | [07_テスト検証環境](07_テスト検証環境.md) | atomic 三表書込 local 検証 / ドメイン CT | enforcement |
| 08 | [08_lintとformat適用](08_lintとformat適用.md) | 全言語 lint / format | enforcement |
| 09 | [09_docs_lint実行手順](09_docs_lint実行手順.md) | run_lint.sh 7 check + run_lint.py 8 check | enforcement |
| 10 | [10_frontmatter規約適用](10_frontmatter規約適用.md) | id 導出 / 7 required / 8 forbidden | convention |
| 11 | [11_軸固有環境設定](11_軸固有環境設定.md) | atomic 三表書込演習 / Domain Event schema 整合 | enforcement |
| 12 | [12_CI完全再現](12_CI完全再現.md) | docs_lint.yml 2 job の手元再現 | enforcement |
| 13 | [13_検収基準](13_検収基準.md) | 4 言語 + 4 サービス起動 + CT 通過 | enforcement |
| 14 | [14_Claude_Code連携](14_Claude_Code連携.md) | CLAUDE.md / skills / memory | policy |

## 検収条件

以下を全て満たすまで「tier2 環境構築完了」とはならない。

> **pre-P0 注記**: `tools/docs_lint/` は P2 deliverable（pre-P0 時点で実行不可）。src/ 依存手順は P10 完了後に有効。

1. `cargo --version` / `go version` / `dotnet --version` / `pnpm --version` → 全応答
2. `docker compose up` で postgres / kafka / clickhouse / apicurio の 4 サービスが起動
3. ドメイン bound CT が全 pass
4. `bash tools/docs_lint/run_lint.sh` → exit 0（P2 deliverable）
5. `python3 tools/docs_lint/run_lint.py` → `8 check 全 green`（P2 deliverable）

## 上位フェーズとの bind

### 上位フェーズへの依存
- [05_環境構築 phase index](../README.md): 共通前提（OS / git / bash）を宣言
- [08_開発体制](../../01_企画/08_開発体制/README.md): tier2 ロール定義の source of truth

## 読み筋

- 初読: 本ファイル → 01_責務とスコープ → 02 → 03 → 04 と順に実行（各ページの検収コマンドを green にしてから次へ）
- 障害対応時参照: 12_CI完全再現 → 07_テスト検証環境 → 11_軸固有環境設定

## 関連参照

- [05_環境構築 index](../README.md)
- [docs/00_format/README.md](../../00_format/README.md)
- [docs/01_企画/08_開発体制/README.md](../../01_企画/08_開発体制/README.md)
