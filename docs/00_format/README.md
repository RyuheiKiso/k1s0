---
id: format.meta.format_index
axis: meta
phase: format
kind: index
status: draft
depends_on: []
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# 00_format 規約層 index

## 一文方針
- `docs/00_format/` は正式層（`docs/01_企画/` 〜 `docs/04_詳細設計/`）の単一の真となる規約 / テンプレート / schema / lint 規約を集約する。文章 only の合意は禁止し、全規約を機械可読 + CI enforce 可能な形で物理化する。

## 至高路線における立ち位置
- 規約層は「規約の規約」として、各フェーズフォルダの章立て / frontmatter / link / 禁止表現 / 番号体系を構造的に固定する。
- リポジトリ root [CLAUDE.md](../../CLAUDE.md) の「至高路線」「運用コスト度外視」「1.0.0 で完璧」「段階的 release 禁止」「機能削減なし」「変更履歴を明文化しない」を frontmatter schema / style guide / lint 規約で物理 enforce する。

## ディレクトリ構造

```
docs/00_format/
├── README.md                        # 本ファイル
├── style_guide.md                   # 文体・記法 style guide
├── frontmatter_schema.yaml          # frontmatter JSON Schema
├── conventions/
│   ├── frontmatter.md               # frontmatter 規約
│   ├── section_marker.md            # ■ → ## のネスト規約
│   ├── crosslink.md                 # 軸間 / フェーズ間 cross link 規約
│   └── numbering.md                 # 番号体系規約
├── templates/
│   ├── 00_責務.md.tpl
│   ├── 01_位置づけ.md.tpl
│   ├── 02_非提供スコープ.md.tpl
│   ├── 03_採用OSS.md.tpl
│   ├── 04_方針.md.tpl
│   ├── 05_運用UIと開発者体験.md.tpl
│   ├── 06_強制機構.md.tpl
│   ├── 07_適合仕様.md.tpl
│   ├── 08_クロスカッティング適合仕様.md.tpl
│   ├── 09_index.md.tpl
│   ├── 10_企画ドキュメント.md.tpl
│   ├── 11_要件定義.md.tpl
│   ├── 12_概要設計.md.tpl
│   └── 13_詳細設計.md.tpl
└── linters/
    ├── markdownlint.json            # MD 構造規約 (markdownlint config)
    ├── textlint.config.mjs          # 日本語文体 + 禁止表現規約 (textlint config)
    └── docs_xref_check.md           # cross-reference 検証規約
```

## テンプレート選択 flowchart

新規ドキュメント作成時、以下の判断 tree でテンプレートを選ぶ。

```
新規 .md を作る
│
├─ ファイル名が README.md（フォルダ index）？
│   └─ → templates/09_index.md.tpl
│
├─ 配置先は 01_企画/ 配下？
│   └─ → templates/10_企画ドキュメント.md.tpl
│
├─ 配置先は 02_要件定義/ 配下？
│   ├─ kind=responsibility / non_scope / oss が当てはまる？
│   │   ├─ responsibility → templates/00_責務.md.tpl（軸別 README として使う場合）
│   │   ├─ non_scope      → templates/02_非提供スコープ.md.tpl
│   │   └─ oss            → templates/03_採用OSS.md.tpl
│   └─ それ以外 → templates/11_要件定義.md.tpl
│
├─ 配置先は 03_概要設計/ 配下？
│   ├─ 各軸の方針 .md → templates/04_方針.md.tpl
│   ├─ クロスカッティング設計 .md → templates/12_概要設計.md.tpl
│   └─ アーキテクチャ概観 .md → templates/12_概要設計.md.tpl
│
├─ 配置先は 04_詳細設計/ 配下？
│   ├─ 01_適合仕様/ 配下 → templates/07_適合仕様.md.tpl
│   ├─ 02_強制機構/ 配下 → templates/06_強制機構.md.tpl
│   ├─ 03_クロスカッティング適合仕様/ 配下 → templates/08_クロスカッティング適合仕様.md.tpl
│   ├─ 04_運用UI開発者体験/ 配下 → templates/05_運用UIと開発者体験.md.tpl
│   └─ 05_lock_yaml体系/ 配下 → templates/13_詳細設計.md.tpl
│
└─ 配置先は 00_format/ 配下？ → 規約層追加（本 README に index 追加 + frontmatter は kind=convention/style/linter/template を選ぶ）
```

## 新規ドキュメント作成手順

1. テンプレート選択 flowchart で対応テンプレートを選ぶ
2. テンプレート全文をコピーして `<...>` placeholder を埋める
3. frontmatter の `id` を [conventions/frontmatter.md](conventions/frontmatter.md) の規則に従って配置パスから導出
4. `depends_on` を [conventions/crosslink.md](conventions/crosslink.md) の規則に従って列挙
5. 本文を [style_guide.md](style_guide.md) の文体規約に従って執筆
6. 章立ては [conventions/section_marker.md](conventions/section_marker.md) に従う（■ → ## の対応）
7. `status: draft` で commit。`status: locked` への昇格は当該 Phase 完了時に lint green を確認後に行う

## 規約変更手順

- 規約の変更は規約層内で完結する。正式層側の lint 規約だけ変えて文書側の章立てが追従していないことを禁止。
- 規約変更 PR は以下を一括で含める:
    1. 規約ファイル（conventions / templates / schema / linters）の変更
    2. 影響を受ける正式層ドキュメントの追従変更
    3. CI lint green 確認
- 規約変更の意思決定は本 README の trail には残さず、git log を単一の真とする（CLAUDE.md「変更履歴を明文化しない」）。

## CI / lint 実装状況

- 規約宣言: 完了（Phase 0 の本作業）
- lint 実装: Phase 5 で `tools/docs_lint/` 配下に実装
- CI 統合: Phase 5

## 関連参照
- [conventions/frontmatter.md](conventions/frontmatter.md)
- [conventions/section_marker.md](conventions/section_marker.md)
- [conventions/crosslink.md](conventions/crosslink.md)
- [conventions/numbering.md](conventions/numbering.md)
- [style_guide.md](style_guide.md)
- [frontmatter_schema.yaml](frontmatter_schema.yaml)
- [linters/docs_xref_check.md](linters/docs_xref_check.md)
- [リポジトリ root CLAUDE.md](../../CLAUDE.md)
