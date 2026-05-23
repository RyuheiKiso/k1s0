---
id: env.data.data_index
axis: data
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

# data 軸エンジニア 環境構築 index

## 一文方針

- data 軸エンジニアは psql / kafka-topics.sh / clickhouse-client / Apicurio CLI / docker compose の全ツールを手元で動作させ、5 OSS ローカル起動成功 + restore_drill 一通り完了 + docs_lint green を環境構築の検収条件とする。

## 至高路線における立ち位置

- data の環境は「DB が接続できる状態」ではなく「5 preservation_class の保全が手元で再現でき、restore_drill が成功する状態」である。docker compose で 5 サービスを起動し、Apicurio schema 登録演習と restore_drill を完了してから CI と同等の検証を行う。
- 14 ページは責務の 14 側面に 1:1 対応し、各ページが独立した検収コマンドを持つ。ページ単位で green を確認してから次ページへ進む。
- 段階的セットアップ禁止: 03_必須ランタイムが完了した直後に 04_リポジトリ取得を実行し、11_軸固有環境設定で 5 OSS 起動が green になるまで 12 以降に進まない。

## 配下ドキュメント

| 番号 | ドキュメント | 主題 | kind |
|---|---|---|---|
| 01 | [01_責務とスコープ](01_責務とスコープ.md) | data ロールの責務と非責務 | responsibility |
| 02 | [02_前提OS環境](02_前提OS環境.md) | WSL2 / Ubuntu / Docker Engine | policy |
| 03 | [03_必須ランタイム](03_必須ランタイム.md) | psql / kafka CLI / ClickHouse CLI / Docker Compose 2+ | policy |
| 04 | [04_リポジトリ取得手順](04_リポジトリ取得手順.md) | clone / baseline lint green 確認 | enforcement |
| 05 | [05_主要OSS導入](05_主要OSS導入.md) | docker compose で 5 サービス起動 | enforcement |
| 06 | [06_開発エディタIDE設定](06_開発エディタIDE設定.md) | VS Code + DB / Kafka 拡張 | enforcement |
| 07 | [07_テスト検証環境](07_テスト検証環境.md) | docker compose integration test / restore_drill | enforcement |
| 08 | [08_lintとformat適用](08_lintとformat適用.md) | sqlfluff / Apicurio schema compat / avro schema lint | enforcement |
| 09 | [09_docs_lint実行手順](09_docs_lint実行手順.md) | run_lint.sh 7 check + run_lint.py 8 check | enforcement |
| 10 | [10_frontmatter規約適用](10_frontmatter規約適用.md) | id 導出 / 7 required / 8 forbidden | convention |
| 11 | [11_軸固有環境設定](11_軸固有環境設定.md) | 5 OSS 起動 / Apicurio schema 登録 / restore_drill | enforcement |
| 12 | [12_CI完全再現](12_CI完全再現.md) | docs_lint.yml 2 job の手元再現 | enforcement |
| 13 | [13_検収基準](13_検収基準.md) | psql / docker compose / apicurio schema 登録成功 | enforcement |
| 14 | [14_Claude_Code連携](14_Claude_Code連携.md) | CLAUDE.md / skills / memory | policy |

## 検収条件

以下を全て満たすまで「data 環境構築完了」とはならない。

> **pre-P0 注記**: `tools/docs_lint/` は P2 deliverable（pre-P0 時点で実行不可）。src/ 依存手順は P10 完了後に有効。

1. `psql --version` → バージョン応答
2. `docker compose up` で 5 サービス（postgres / kafka / clickhouse / apicurio / valkey）起動
3. Apicurio schema register 成功
4. restore_drill（backup → drop → restore → verify）完了
5. `bash tools/docs_lint/run_lint.sh` → exit 0（P2 deliverable）
6. `python3 tools/docs_lint/run_lint.py` → `8 check 全 green`（P2 deliverable）

## 上位フェーズとの bind

### 上位フェーズへの依存
- [05_環境構築 phase index](../README.md): 共通前提（OS / git / bash）を宣言
- [08_開発体制](../../01_企画/08_開発体制/README.md): data ロール定義の source of truth

## 読み筋

- 初読: 本ファイル → 01_責務とスコープ → 02 → 03 → 04 と順に実行（各ページの検収コマンドを green にしてから次へ）
- 障害対応時参照: 12_CI完全再現 → 09_docs_lint実行手順 → 07_テスト検証環境

## 関連参照

- [05_環境構築 index](../README.md)
- [docs/00_format/README.md](../../00_format/README.md)
- [docs/01_企画/08_開発体制/README.md](../../01_企画/08_開発体制/README.md)
