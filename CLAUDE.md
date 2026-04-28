# プロジェクト規約

本ファイルは k1s0 プロジェクトのルート規約を定める。詳細は作業種別ごとにサブツリー `CLAUDE.md` と Skill に分割されている。ルートファイルは毎ターン常駐するため、**全体を通して必要な最小限**のみを記載する。

## 技術スタック

- Rust Edition 2024（自作領域: ZEN Engine 統合 / crypto / 雛形 CLI / JTC 固有機能）
- Go（Dapr ファサード層: stable Dapr Go SDK を使用）
- tier1 内部のサービス間通信は Protobuf gRPC 必須。
- tier2/tier3 から内部言語は不可視（クライアントライブラリと gRPC エンドポイントのみ公開）。

依存方向: `tier3 → tier2 → (sdk ← contracts) → tier1 → infra` の一方向。逆向きは禁止。

## プロジェクト構造（要点）

モノレポ一次ディレクトリは `src/`、クラスタ構成は `infra/`、GitOps 配信は `deploy/`、運用は `ops/`、横断ツールは `tools/`、tier 横断テストは `tests/`、ドキュメントは `docs/`。スパースチェックアウト（cone mode）での役割別運用が標準（`./tools/sparse/checkout-role.sh <role>`）。

詳細構造:

- ディレクトリ構成の詳細: `docs/05_実装/00_ディレクトリ設計/`
- docs 配下の索引: [docs/INDEX.md](docs/INDEX.md)
- ディレクトリ系 ADR: `docs/02_構想設計/adr/ADR-DIR-001` / `002` / `003`

## 作業別の規約読み込み

作業を始める時、該当区分のサブツリー `CLAUDE.md` と Skill を必ず読み込むこと。呼び忘れは規約違反に直結する。

| 作業区分 | 読むもの / 使うコマンド |
|----------|--------------------------|
| `src/` 配下のコード作成・編集 | [`src/CLAUDE.md`](src/CLAUDE.md) |
| `docs/` 配下のドキュメント作成・編集（**全 docs 作業の前提**） | [`docs/CLAUDE.md`](docs/CLAUDE.md) + Skill `docs-delivery-principles` |
| 要件定義書（`03_要件定義/`） | Skill `docs-requirement-writing` / コマンド `/requirement` |
| ADR 起票・編集（`02_構想設計/adr/`） | Skill `docs-adr-authoring` / コマンド `/adr` |
| 技術説明書・設計書（`05_実装/` / `04_概要設計/` / `02_構想設計/`） | Skill `docs-design-spec` / コマンド `/design` |
| Knowledge ドキュメント（`90_knowledge/`） | コマンド `/knowledge` |
| ポストモーテム | Skill `docs-postmortem` |
| Runbook（`docs/40_運用ライフサイクル/`、タイプ C: 検出 / 初動 / 復旧 / 原因調査 / 事後処理 5 段構成） | Skill `docs-design-spec`（共通フォーマット）+ `docs-postmortem`（インシデント由来の場合） |
| PR レビュー / セルフチェック | Skill `docs-review-checklist`（汎用レビューは組み込み `/review`） |
| drawio 図の新規作成・編集 | Skill `drawio-authoring` |
| 複数レイヤが登場する drawio | Skill `figure-layer-convention` |
| docs の横断調査（複数ディレクトリ / 複数ファイル / ID 網羅探索） | `Agent(subagent_type=docs-explorer)` に委譲（定義: [.claude/agents/docs-explorer.md](.claude/agents/docs-explorer.md)、Haiku ベースで `INDEX.md → README.md → 目的ファイル` を遵守し要約のみ返す）。ID 既知の単一ファイル直読は親で Read する方が軽い。 |

**docs 作業の絶対原則**: 開始時に必ず `docs-delivery-principles` Skill を読むこと。過去に「量を言い訳にした段階対応」で叱責を受けた経緯があり、納品品質は最初から果たすべき責務として扱う。

drawio を編集する時は必ず `drawio-authoring` Skill を呼ぶこと（過去の「SVG 背景透過」「矢印が要素を覆い隠す」事故の再発防止のため）。

## コンテキスト管理

- コンテキストウィンドウの限界に近づいた場合、Claude Code ハーネスが自動的にコンパクト（古いツール出力の削除 → 会話の要約）を実行する。モデル側からの制御はできない。
- 作業の区切りなどで明示的にコンパクトしたい場合は、`/compact` スラッシュコマンドを利用する。焦点を指定したい場合は `/compact <focus>` の形式で実行する。
- 会話履歴を完全に破棄して作業を再開したい場合は、`/clear` を利用する。

## ポリシー

目先の楽な提案や作業ではなく、プロフェッショナルとしての最適解を追求すること。
プロジェクトの成功にとって重要なことは何かを常に考え、短期的な妥協を避けること。
プロジェクトの品質と成功に対して妥協しないこと。必要な場合は、困難な決定を下すことを恐れないこと。
一切の妥協は認めない。
未来への先送りは許さない。
