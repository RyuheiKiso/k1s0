# docs ドキュメント規約

本ファイルは `docs/` 配下のドキュメント作成・編集時に適用する規約を定める。索引と探索動線（階層ナビ・ID 体系・Explore 委譲の基準・典型パターン）は [`INDEX.md`](INDEX.md) に集約してあるため、docs 作業の最初に必ず開くこと。本ファイルはあくまで「docs 作業時の規約」に絞る。

## 絶対原則（最初に読む）

どんな docs 作業でも、開始時に `docs-delivery-principles` Skill を**必ず**読むこと。これは書式ルールではなく**作業の姿勢**に関する原則で、過去セッション `72d524f1`（2026-04-14）で「量を言い訳にした段階対応」で叱責を受けた経緯から、全 docs 作業の土台として扱う。

## 基本方針

- アスキーによる図や表の表現は禁止。
- 図表を掲載する場合は、md と同じ階層に `img` フォルダを作成し、drawio を作成してから SVG を出力して md 内に埋め込むこと。端末には drawio がインストールされているためこれを利用する。
- 資料は可能な限り細分化する。
- 表を並べるだけの構成は不可。部外者が読んで文脈を理解できる品質を必須とする。
- 各章・各節の冒頭に、何を解決するかの導入段落を必ず置くこと。
- 表の前後に「なぜこの分類なのか」「どう読むか」「重要な関係性」の散文を添えること。
- 重要な概念・関係性・フロー・構造は drawio 図を作成し SVG で埋め込むこと。

## 作業別 Skill（呼び忘れ厳禁）

文書タイプは `docs/00_format/document_standards.md` に準拠（タイプ A: 要件定義書 / タイプ B: ADR / タイプ C: Runbook / タイプ D: 技術説明書・設計書）。作業と呼ぶ Skill の対応は次表のとおり:

| 作業 | Skill / コマンド | 対象範囲 |
|------|------------------|----------|
| 全 docs 作業の前提 | `docs-delivery-principles` | docs 配下すべて |
| 要件定義書（タイプ A） | `docs-requirement-writing` / `/requirement` | `03_要件定義/` |
| ADR 起票・編集（タイプ B） | `docs-adr-authoring` / `/adr` | `02_構想設計/adr/` |
| 技術説明書・設計書（タイプ D） | `docs-design-spec` / `/design` | `05_実装/` / `04_概要設計/` / `02_構想設計/` |
| Knowledge ドキュメント | `/knowledge` | `90_knowledge/` |
| ポストモーテム | `docs-postmortem` | インシデント後（`docs/40_運用ライフサイクル/postmortems/` 予定） |
| PR レビュー / セルフチェック | `docs-review-checklist` / 組み込み `/review` | 変更範囲のみ通読 |
| drawio 図の新規作成・編集 | `drawio-authoring` | 全 `*.drawio` |
| 複数レイヤが登場する drawio | `figure-layer-convention` | 2 層以上が登場する図 |

作業が複数タイプに跨る場合は該当 Skill を**両方**読み込むこと。スラッシュコマンドは雛形展開の自動化で、内部で対応する Skill を自動で呼ぶ。

## 探索動線の要点（詳細は [INDEX.md](INDEX.md)）

docs 配下は 600 ファイル超・53,000 行超の規模で、全 md を親コンテキストに読むとトークンを急速に消費する。最小動線は [`INDEX.md`](INDEX.md) → 該当ディレクトリの `README.md` → 目的ファイル、の 3 段で辿ること。**2 ファイル以上を読む見込みになった時点で `Agent(subagent_type=docs-explorer)` に委譲**し、親には要約のみを返させる（定義: [`.claude/agents/docs-explorer.md`](../.claude/agents/docs-explorer.md)、Haiku ベースの専用 Explore）。ID 既知のピンポイント参照は親で直読してよい（サブエージェント起動コストの方が高くつく）。階層ナビゲーション表・ID 体系表・ADR 系列・典型パターンは `INDEX.md` に集約してあるため、本ファイルには重複して置かない。

なお、docs/ 配下の Read が連続するとハーネス側の hook（`.claude/hooks/docs-read-guard.py`）が警告を出し、一定回数を超えるとブロックして `docs-explorer` への委譲を強制する。警告が出た時点で親での直読を止め、サブエージェントに切り替えること。

## 参照

- フォーマット本体: `docs/00_format/`
  - `document_standards.md` — ドキュメント記述標準
  - `drawio_layer_convention.md` — 図解レイヤ記法規約（正典）
  - `review_checklist.md` — レビューチェックリスト
