---
id: env.overview.business_admin_index
axis: overview
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

# 業務管理者 環境構築 index

## 一文方針

- 業務管理者は Backstage UI へのブラウザアクセス / tier2 admin API CLI（`k1s0-admin`）/ 監査検索 UI の 3 接続を確立し、tenant マスタ操作 demo / 監査検索 1 件ヒット / 決定表編集フローを完走することを環境構築の検収条件とする。

## 至高路線における立ち位置

- 業務管理者の環境は「ブラウザが開ける状態」ではなく「tenant マスタを安全に操作し、変更内容を監査ログで追跡できる状態」である。Backstage UI / tier2 admin API / 監査検索の 3 ツールを組み合わせて業務フローを体得することが目標。
- 14 ページは責務の 14 側面に 1:1 対応し、各ページが独立した検収コマンドを持つ。
- 段階的セットアップ禁止: 03_必須ランタイムが完了した直後に 04_リポジトリ取得を実行し、11_軸固有環境設定で 3 demo が完走するまで 12 以降に進まない。

## 配下ドキュメント

| 番号 | ドキュメント | 主題 | kind |
|---|---|---|---|
| 01 | [01_責務とスコープ](01_責務とスコープ.md) | テナントマスタ管理・決定表・監査・partner 連携 | responsibility |
| 02 | [02_前提OS環境](02_前提OS環境.md) | Windows 11 + ブラウザ（WSL2 任意） | policy |
| 03 | [03_必須ランタイム](03_必須ランタイム.md) | ブラウザ / tier2 admin API CLI | policy |
| 04 | [04_リポジトリ取得手順](04_リポジトリ取得手順.md) | clone / baseline lint green 確認 | enforcement |
| 05 | [05_主要OSS導入](05_主要OSS導入.md) | Backstage ブラウザ操作 / k1s0-admin CLI 導入 | enforcement |
| 06 | [06_開発エディタIDE設定](06_開発エディタIDE設定.md) | ブラウザのみ / 決定表エディタ（Backstage プラグイン） | enforcement |
| 07 | [07_テスト検証環境](07_テスト検証環境.md) | tier2 admin API staging demo / 決定表編集フロー | enforcement |
| 08 | [08_lintとformat適用](08_lintとformat適用.md) | schema validation / 決定表形式 lint | enforcement |
| 09 | [09_docs_lint実行手順](09_docs_lint実行手順.md) | run_lint.sh / run_lint.py の手元実行 | enforcement |
| 10 | [10_frontmatter規約適用](10_frontmatter規約適用.md) | id 導出 / 7 required / 8 forbidden | convention |
| 11 | [11_軸固有環境設定](11_軸固有環境設定.md) | tenant マスタ作成・編集・削除 demo / 監査検索 demo | enforcement |
| 12 | [12_CI完全再現](12_CI完全再現.md) | docs_lint.yml 2 job の手元再現 | enforcement |
| 13 | [13_検収基準](13_検収基準.md) | Backstage アクセス / API 接続 / demo 完了 | enforcement |
| 14 | [14_Claude_Code連携](14_Claude_Code連携.md) | LLM 補助の範囲 / マスタ操作は人間が実行 | policy |

## 検収条件

以下を全て満たすまで「業務管理者の環境構築完了」とはならない。

1. Backstage UI へのアクセス成功
2. tier2 admin API 接続成功（staging 環境）
3. tenant マスタ操作 demo 完了（作成・編集・削除）
4. 監査検索 1 件ヒット
5. `bash tools/docs_lint/run_lint.sh` → exit 0

## 上位フェーズとの bind

### 上位フェーズへの依存
- [05_環境構築 phase index](../README.md): 共通前提（OS / git / bash）を宣言
- [08_開発体制](../../01_企画/08_開発体制/README.md): 業務管理者ロール定義の source of truth

## 読み筋

- 初読: 本ファイル → 01_責務とスコープ → 02 → 03 → 04 と順に実行
- 障害対応時参照: 12_CI完全再現 → 09_docs_lint実行手順 → 07_テスト検証環境

## 関連参照

- [05_環境構築 index](../README.md)
- [docs/00_format/README.md](../../00_format/README.md)
- [docs/01_企画/08_開発体制/README.md](../../01_企画/08_開発体制/README.md)
