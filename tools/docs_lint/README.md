# tools/docs_lint

ドキュメント lint 仕様 + 実装。CI 統合用。

## 一文方針
- 全 .md ドキュメントを 7 種 check（frontmatter / id 一意性 / id 導出整合 / depends_on 参照 / cross-link dangling / 禁止表現 / 空セクション / TBD 残存）で機械検査し、`release_gate.lock.yaml` の `documentation_integrity` cell を ship blocker として物理 enforce する。

## 7 check

### 1. frontmatter schema 検査
- `docs/00_format/frontmatter_schema.yaml`（JSON Schema draft 2020-12）に従い全 .md frontmatter を検証
- required: `id` / `axis` / `phase` / `kind` / `status` / `depends_on` / `covered_by`
- forbidden: `changelog` / `last_updated` / `last_modified` / `author` / `reviewers`
- optional: `lock_artifacts`（pattern `^[a-z][a-z0-9_]*\.lock\.yaml$`）

### 2. id 一意性検査
- 全 .md の `id` フィールドが globally 一意であること
- 重複は CI fail

### 3. id 導出整合
- `id` が file path から機械的に導出可能であること
- pattern: `<phase_short>.<axis>.<slug>`
- 不一致は CI fail

### 4. depends_on 参照整合
- 全 .md の `depends_on` フィールドが他 .md の `id` を指していること
- dangling reference（存在しない id）は CI fail
- 循環依存検出（DAG 検査）

### 5. cross-link dangling 検査
- 全 .md 内の `[text](path)` 形式のリンクが存在するファイルを指すこと
- dangling link は CI fail（外部 URL は除外）

### 6. 禁止表現検査
- 文書内に次の禁止表現が含まれないこと:
  - `changelog` / `last_updated` / `last_modified` / `author` / `reviewers`（frontmatter）
  - 「Phase X で...Phase Y で追記」型の段階的表現（CLAUDE.md 「変更履歴を明文化しない」と整合）
  - 自由テキスト attribute 名（観測適合仕様 規律）

### 7. 空セクション / TBD 残存検査
- `## section` 直後に空 body / 「TBD」「未定」「後述」のみのセクションは `status: locked` ドキュメントで CI fail
- `status: draft` は warn のみ

## 実装

```bash
# 実行
bash tools/docs_lint/run_lint.sh

# 個別 check
bash tools/docs_lint/check_frontmatter.sh
bash tools/docs_lint/check_id_uniqueness.sh
bash tools/docs_lint/check_depends_on.sh
bash tools/docs_lint/check_dangling_links.sh
bash tools/docs_lint/check_forbidden_expressions.sh
bash tools/docs_lint/check_empty_sections.sh
```

## CI 統合
- GitHub Actions / Tekton で `bash tools/docs_lint/run_lint.sh` を実行
- exit code != 0 で CI fail
- `release_gate.lock.yaml` の `documentation_integrity` cell は本 lint の green を入力とする

## release_gate との接合
- `documentation_integrity` cell は次の 7 dimension を AND-gate:
  - `frontmatter_valid: true`
  - `id_unique: true`
  - `id_derivation_match: true`
  - `depends_on_resolved: true`
  - `no_dangling_links: true`
  - `no_forbidden_expressions: true`
  - `no_empty_sections_in_locked: true`
- 全 green で `release_gate.documentation_integrity.status = green`
- 1 つでも fail で release pipeline 物理 block

## 至高路線における立ち位置
- 文書整合性は文章運用ではなく lint で物理 enforce
- `status: locked` ドキュメントの完成度を CI で物理保証
- CLAUDE.md「変更履歴を明文化しない」「機能削減なし」「1.0.0 で完璧」の整合を文書 lint で機械化

## 関連参照
- [00_format/frontmatter_schema.yaml](../../docs/00_format/frontmatter_schema.yaml)
- [00_format/conventions/](../../docs/00_format/conventions/)
- [release_gate 体系](../../docs/04_詳細設計/05_lock_yaml体系/03_release_gate体系.md)
