---
id: env.meta.author_index
axis: meta
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

# k1s0 作者 環境構築 index

## 一文方針

- k1s0 作者は toolchain authoring / 19 軸 registry governance / cosign release 署名の 3 責務を持つ最高権限ロールであり、docs_lint / drawio-lint / lock_yaml 生成器を含む全 CI 強制機構を手元で完全再現できることが環境構築の検収条件である。

## 至高路線における立ち位置

- k1s0 作者の環境は「CI が実行できる状態」ではなく「CI を作れる状態」である。lint スクリプトの改訂・schema 拡張・新軸登録の全てを手元で完結させ、push 前に全 green を確認する。
- 14 ページは責務の 14 側面に 1:1 対応し、それぞれが独立した検収コマンドを持つ。一括ではなくページ単位で green を確認してから次ページへ進む。
- 段階的セットアップ禁止: 03_必須ランタイムが終わった直後に 04_リポジトリ取得を実行し、05 の lint が green になるまで次に進まない。

## 配下ドキュメント

| 番号 | ドキュメント | 主題 | kind |
|---|---|---|---|
| 01 | [01_責務とスコープ](01_責務とスコープ.md) | 作者ロールの責務 4 本と非責務 | responsibility |
| 02 | [02_前提OS環境](02_前提OS環境.md) | WSL2 / Ubuntu / Windows 11 / bash | policy |
| 03 | [03_必須ランタイム](03_必須ランタイム.md) | Python 3.12 / venv / PyYAML | policy |
| 04 | [04_リポジトリ取得手順](04_リポジトリ取得手順.md) | clone / baseline lint green 確認 | enforcement |
| 05 | [05_docs_lint実行手順](05_docs_lint実行手順.md) | run_lint.sh 7 check + run_lint.py 8 check | enforcement |
| 06 | [06_markdownlint_textlint適用](06_markdownlint_textlint適用.md) | markdown / textlint ローカル適用 | enforcement |
| 07 | [07_drawio作図ツール手順](07_drawio作図ツール手順.md) | drawio-lint / drawio-export / svg-postcheck | enforcement |
| 08 | [08_lock_yaml生成器手順](08_lock_yaml生成器手順.md) | generate_release_gate.py / 20 cell catalog | enforcement |
| 09 | [09_frontmatter規約適用](09_frontmatter規約適用.md) | id 導出 / 7 required / 8 forbidden | convention |
| 10 | [10_crosslinkと依存グラフ](10_crosslinkと依存グラフ.md) | depends_on DAG / body link parity | convention |
| 11 | [11_19軸登録手順](11_19軸登録手順.md) | registry.yaml 操作 / v1 cap 20 残 1 | enforcement |
| 12 | [12_CI完全再現](12_CI完全再現.md) | docs_lint.yml 2 job の手元再現 | enforcement |
| 13 | [13_署名とReleaseGate](13_署名とReleaseGate.md) | cosign signed tag / AND-gate / 責務分離 | enforcement |
| 14 | [14_Claude_Code連携](14_Claude_Code連携.md) | .claude/skills / CLAUDE.md / memory | policy |

## 検収条件

以下を全て満たすまで「作者の環境構築完了」とはならない。

1. `bash tools/docs_lint/run_lint.sh` → exit 0
2. `python3 tools/docs_lint/run_lint.py` → `8 check 全 green`
3. `python3 .claude/skills/drawio-authoring/bin/drawio-lint --help` が起動する
4. `/mnt/c/Program\ Files/draw.io/draw.io.exe --version` または `$DRAWIO_BIN --version` が応答する
5. `python3 tools/lock_yaml_generator/generate_release_gate.py` → `tools/lock_yaml_generator/samples/release_gate.lock.yaml` が出力される
6. git tag で cosign を使った署名が `cosign sign-blob` で実行できる（鍵の確保含む）

## 上位フェーズとの bind

### 上位フェーズへの依存
- [05_環境構築 phase index](../README.md): 共通前提（OS / git / bash）を宣言
- [08_開発体制](../../01_企画/08_開発体制/README.md): k1s0 作者ロール定義の source of truth

## 読み筋

- 初読: 本ファイル → 01_責務とスコープ → 02 → 03 → 04 と順に実行（各ページの検収コマンドを green にしてから次へ）
- 障害対応時参照: 12_CI完全再現 → 05_docs_lint実行手順 → 07_drawio作図ツール手順

## 関連参照

- [05_環境構築 index](../README.md)
- [docs/00_format/README.md](../../00_format/README.md)
- [docs/01_企画/08_開発体制/README.md](../../01_企画/08_開発体制/README.md)
