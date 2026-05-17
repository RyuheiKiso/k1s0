---
id: format.meta.docs_xref_check
axis: meta
phase: format
kind: linter
status: draft
depends_on: []
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# docs cross-reference 検証規約

## 一文方針
- 全 `.md` ドキュメントについて、frontmatter `id` の path 整合 / `depends_on` の dangling check / 本文 link の dangling check / `depends_on` と本文 link の一致 check / `lock_artifacts` catalog 整合 / 禁止表現の残存 check を機械的に行い、status: locked で違反 1 件あれば CI fail とする。

## 至高路線における原則
- 文書間整合は文章運用ではなく lint で物理 enforce する。
- 1 件の違反でも CI fail とし、warning 経路を持たない。「軽微なら許容」「あとで直す」を構造的に拒否する。
- 検証は build artifact として `docs_lint_report.lock.yaml` を出力、`release_gate.lock.yaml` の cell として 1.0.0 ship blocker 化する。

## 検証 cell（実装は Phase 5）

### cell-1: frontmatter schema 検証
- 入力: 全 `.md` の frontmatter
- schema: `docs/00_format/frontmatter_schema.yaml`
- 検証:
    - required field 完備
    - forbidden field 不在（changelog / last_updated / author 等）
    - enum 整合（axis / phase / kind / status / defense_in_depth_layers / proof_classes）
- 違反: status: locked で 1 件以上 → CI fail

### cell-2: id ↔ path 導出整合
- 入力: 全 `.md` の frontmatter `id` と配置パス
- 検証:
    - `id` の `<phase_short>` が path の最上位フェーズ番号と一致
    - `id` の `<axis>` が path 内の軸 namespace と一致
    - `id` の `<slug>` がファイル名と一致（番号 prefix を除く部分）
- 違反: 1 件以上 → CI fail
- 例:
    - path `docs/04_詳細設計/01_適合仕様/20_形式検証適合仕様.md` → id `detail.formal.formal_conformance` ✓
    - path `docs/03_概要設計/11_formal設計方針/05_時相安全性方針.md` → id `arch.formal.temporal_safety_policy` ✓

### cell-3: depends_on dangling check
- 入力: 全 `.md` の frontmatter `depends_on` 配列
- 検証:
    - 各 `id` が repo 内の他 `.md` の frontmatter `id` として存在
- 違反: 1 件以上 → CI fail（warning なし）

### cell-4: 本文 link dangling check
- 入力: 全 `.md` の本文中の `[...](relative_path.md)` link
- 検証:
    - link 先のファイルが実在
    - 相対パスが正しい（絶対パス禁止）
    - アンカー（`#section`）を含まない（conventions/crosslink.md 規約）
- 違反: 1 件以上 → CI fail

### cell-5: depends_on ↔ 本文 link 一致 check
- 入力: 全 `.md` の `depends_on` と本文 link
- 検証:
    - `depends_on` に列挙された `id` は本文中にも link が存在
    - 本文中の link 先 `.md` の `id` は `depends_on` にも含まれる（同一フェーズ内・index・90_archive への link は除外）
- 違反: status: locked で 1 件以上 → CI fail

### cell-6: lock_artifacts catalog 整合
- 入力: 全 `.md` の frontmatter `lock_artifacts`
- 検証:
    - 列挙された `*.lock.yaml` が `docs/04_詳細設計/05_lock_yaml体系/` の catalog に登録済み
- 違反: 1 件以上 → CI fail

### cell-7: 禁止表現残存 check
- 入力: status: locked の全 `.md` の本文
- 検証:
    - `TBD` / `TODO` / `未定` / `あとで書く` / `後述` / `将来検討` / `たぶん` / `おそらく` / `〜したほうが良い` / `〜が望ましい` の不在
    - 空セクション（`## ◯◯` 直後に内容なし）の不在
- 違反: 1 件以上 → CI fail

### cell-8: 章立てテンプレート整合
- 入力: 全 `.md` のセクション構造（kind 別）
- 検証:
    - kind=responsibility → `## 一文定義` `## 責務の境界` を含む
    - kind=oss → `## 機能カテゴリ × OSS（v1_l1plus_primary 単一深耕）` を含む
    - kind=conformance_spec → `## 単一の真の N ファイル構成` `## 整合 check（CI 強制）` を含む
    - kind=enforcement → `## 5 層 defense-in-depth` を含む
    - kind=ops_dx → `## ChatOps bot 機能` `## CI runner / 夜間 batch` を含む
    - その他 kind ごとの必須セクションは `docs/00_format/templates/` から導出
- 違反: 1 件以上 → CI fail

### cell-9: 循環参照 check
- 入力: 全 `.md` の `depends_on` を有向 graph として構築
- 検証:
    - DAG 性（許可リスト pair を除く）
- 許可 pair: `(detail.test.verification_conformance, detail.formal.formal_conformance)` 等（明示登録）
- 違反: 1 件以上 → CI fail

## 実装 (Phase 5)

- `tools/docs_lint/`:
    - `frontmatter_validator.py` (cell-1, cell-2, cell-6)
    - `crosslink_check.py` (cell-3, cell-4, cell-5, cell-9)
    - `template_compliance.py` (cell-8)
    - `forbidden_expression_check.py` (cell-7)
- 全 cell の合算結果を `docs_lint_report.lock.yaml` に出力。
- `release_gate.lock.yaml` の cell `docs_lint` として ship blocker 化。

## CI 実行 trigger
- pull request: 全 cell 実行 (status: draft / locked 関係なく warn / fail を judgement)
- merge to main: 全 cell 実行 + status: locked の違反は merge block
- nightly: 全 cell 実行 + report を Artifact registry に push

## 例外 / break-glass
- lint 違反の bypass は禁止。`# textlint-disable` `# noqa` 等の suppress 注釈は CI で検出 + fail。
- 真に例外が必要な場合は規約自体を更新する（conventions / templates / schema の修正）。
