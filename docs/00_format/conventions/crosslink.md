# 軸間 / フェーズ間 cross link 規約

## 一文方針
- ドキュメント間の参照は frontmatter `depends_on`（機械可読）と本文内 link（読者向け）の二経路で表現する。文章中の「〇〇を参照」だけの記述は禁止し、必ず実際の link を伴わせる。dangling link は CI fail。

## 至高路線における原則
- 参照は機械的に検証可能でなければならない。文章 only の参照は drift 源であり、リネーム / 削除 / 移動を検知できない。
- 19 軸 × 4 フェーズの cross 構造は格子であり、全 link が `depends_on` graph に投影されることで「軸間整合」「フェーズ間整合」が CI で物理 enforce される。
- 参照記法は単一の真とする。複数記法を許すと lint が複雑化し、整合確認が破綻する。

## 参照の二経路

### frontmatter `depends_on`（機械可読、必須）

```yaml
depends_on:
  - arch.formal.responsibility
  - detail.test.verification_conformance
```

- 同一 repo 内の他 `.md` の `id` を配列で列挙する。
- 何を `depends_on` に含めるか:
  - 当ドキュメントの主張が前提とする他軸の規律
  - 当ドキュメントが参照する他軸の概念定義
  - 当ドキュメントから直接 link する他ドキュメント
- 含めないもの:
  - 同一フェーズ内の隣接ドキュメント（同じ親フォルダ配下）
  - README.md 等の index ドキュメント
  - 90_knowledge への参照（後述の別経路）

### 本文内 link（読者向け、必須伴走）

`depends_on` に列挙したものは、本文中で初出時に必ず link を張る。link 記法は以下の単一形式のみ。

```markdown
[<表示文言>](<相対パス>.md)
```

例:
```markdown
[formal 責務](../../03_概要設計/11_formal設計方針/01_責務.md) で宣言された通り、...
```

- 相対パスを使う（絶対パス禁止）。
- `.md` 拡張子は省略しない。
- アンカー（`#section-name`）は使わない（章立てが変わると壊れるため、ファイル単位の link に留める）。
- 表示文言は対象ドキュメントのタイトルまたは主張の引用に揃える。

## 知識層への参照

### `docs/90_knowledge/` への参照
- 知識層は技術前提の reference として参照可。`depends_on` には含めず、本文 link のみ。
  ```markdown
  詳細は [Kyverno admission policy](../../90_knowledge/tool/kyverno.md) を参照。
  ```

## 軸間 link の典型パターン

### 軸 A の概要設計 → 軸 B の適合仕様
```yaml
# in: docs/03_概要設計/11_formal設計方針/05_時相安全性方針.md
depends_on:
  - detail.tier1.bidi_conformance        # 07_Bidi 適合仕様の state machine を proof 対象として参照
```

### フェーズ間 link（同一軸内）
```yaml
# in: docs/04_詳細設計/01_適合仕様/20_形式検証適合仕様.md
depends_on:
  - req.formal.verification_requirement   # 02_要件定義/03_非機能要件/09_検証規律要件.md
  - arch.formal.responsibility            # 03_概要設計/11_formal設計方針/README.md
```

### クロスカッティング（複数軸を bind）
```yaml
# in: docs/04_詳細設計/03_クロスカッティング適合仕様/01_HTTP2_enforcement.md
depends_on:
  - detail.tier1.bidi_conformance
  - arch.tier3.realtime_update_ux
  - arch.tier3.application_form
  - arch.tier3.legacy_integration
```

## CI 検証規約（Phase 5 で実装）

- `tools/docs_lint/crosslink_check.py`（仮称）が以下を検証:
  - `depends_on` 中の全 `id` が repo 内に存在する（dangling 検出）
  - 本文内 `[...](relative_path.md)` link 先のファイルが実在する
  - `depends_on` に含まれる `id` は本文中にも link が存在する（一致性）
  - 本文中の link 先 `.md` の `id` は `depends_on` にも含まれる（漏れ検出）
- 違反 1 件で CI fail。

## 循環参照

- `depends_on` の DAG 化は強制しない（メタ循環があり得る: test ↔ formal）。
- ただし循環は明示的に許容される pair のみとし、CI で許可リスト管理する。
  - 許可: `(detail.test.verification_conformance, detail.formal.formal_conformance)`
  - その他の循環は CI fail。

## アンカー / 見出し ID

- MD 見出しからの自動生成アンカーには依存しない（章立て変更で壊れるため）。
- 「ファイル単位」が link の最小粒度。セクション参照が必要な場合は当該セクションを別ファイルに分割する。
