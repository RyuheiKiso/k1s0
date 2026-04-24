# docs ドキュメント規約

本ファイルは `docs/` 配下のドキュメント作成・編集時に適用する規約を定める。ルート `CLAUDE.md` の補完として、docs 作業時にのみ読まれることを想定する。

## 基本方針（常に適用）

- アスキーによる図や表の表現は禁止。
- 図表を掲載する場合は、md と同じ階層に `img` フォルダを作成し、drawio を作成してから svg を出力して md 内に埋め込むこと。端末には drawio がインストールされているためこれを利用する。
- 資料は可能な限り細分化すること。
- 表を並べるだけの構成は不可。部外者が読んで文脈を理解できる品質が必須。
- 各章・各節の冒頭に、何を解決するかの導入段落を必ず置くこと。
- 表の前後に「なぜこの分類なのか」「どう読むか」「重要な関係性」の散文を添えること。
- 重要な概念・関係性・フロー・構造は drawio 図を作成し SVG で埋め込むこと。

## docs 作業の絶対原則（最初に読む）

どんな docs 作業でも、開始時に `docs-delivery-principles` Skill を**必ず**読むこと。これは書式ルールではなく**作業の姿勢**に関する原則で、過去セッション `72d524f1`（2026-04-14）で「量を言い訳にした段階対応」で叱責を受けた経緯から、全 docs 作業の土台として扱う。

## 作業別の追加規約（Skill を呼ぶ）

docs 配下での作業種別ごとに、以下の Skill を呼び出して該当規約を読み込むこと。呼び忘れを防ぐため、作業開始時にチェックする。文書タイプは `docs/00_format/document_standards.md` に準拠（タイプ A: 要件定義書 / タイプ B: ADR / タイプ C: Runbook / タイプ D: 技術説明書・設計書）。

| 作業 | 呼ぶ Skill / コマンド | 対象範囲 |
|------|------------------------|----------|
| 全 docs 作業の前提 | Skill `docs-delivery-principles` | docs 配下すべて |
| 要件定義書（タイプ A） | Skill `docs-requirement-writing` / コマンド `/requirement` | `03_要件定義/`（`BR-` / `FR-` / `NFR-A〜I-` / `OR-` / `DX-` / `BC-` / `RISK-`） |
| ADR 起票・編集（タイプ B） | Skill `docs-adr-authoring` / コマンド `/adr` | `02_構想設計/adr/`（`ADR-DOMAIN-NNN-short-name.md` 形式） |
| 技術説明書・設計書（タイプ D） | Skill `docs-design-spec` / コマンド `/design` | `05_実装/`、`04_概要設計/`、`02_構想設計/` の設計資料 |
| Knowledge ドキュメント | コマンド `/knowledge` | `90_knowledge/` 配下 |
| ポストモーテム | Skill `docs-postmortem` | インシデント後、`docs/40_運用ライフサイクル/postmortems/`（予定） |
| PR レビュー / セルフチェック | Skill `docs-review-checklist` / 組み込み `/review` | 変更範囲に該当するセクションのみ通読 |
| drawio 図の新規作成・編集 | Skill `drawio-authoring` | 全 `*.drawio` |
| 複数レイヤが登場する drawio | Skill `figure-layer-convention` | アプリ/ネットワーク/インフラ/データの 2 層以上が登場する図 |

作業が複数タイプに跨る場合（例: ADR を書きつつ設計書も更新する）は、該当 Skill を**両方**読み込むこと。スラッシュコマンドは雛形展開の自動化で、内部で対応する Skill を自動で呼ぶ。

## docs 配下の構造

```
docs/
├── 00_format/          # フォーマットテンプレート・規約
├── 01_企画/            # 稟議向け企画資料
├── 02_構想設計/        # 技術深掘り資料（ADR 索引含む）
│   └── adr/            # ADR 全体（ADR-DIR-001/002/003 含む）
├── 03_要件定義/        # 要件定義書（IPA 共通フレーム 2013 準拠）
├── 04_概要設計/        # 概要設計（DS-SW-COMP-*）
├── 05_実装/            # 実装フェーズ設計（IMP-DIR-* 等）
├── 90_knowledge/       # 技術学習用ドキュメント（/knowledge コマンドで生成）
└── 99_壁打ち/          # ブレスト・検討メモ
```

主要ファイルと ID 体系の索引は [INDEX.md](INDEX.md) を参照すること。

## 参照

- フォーマット本体: `docs/00_format/`
  - `document_standards.md` — ドキュメント記述標準
  - `drawio_layer_convention.md` — 図解レイヤ記法規約（正典）
  - `review_checklist.md` — レビューチェックリスト
