---
id: env.meta.author_claude_code_integration
axis: meta
phase: env_setup
kind: policy
status: draft
depends_on:
  - env.meta.author_signing_release_gate
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# Claude Code 連携

## 一文方針

- k1s0 作者は Claude Code を補助ツールとして活用するが、spec merge / schema 変更 / cosign 署名は LLM 単独でなく人間 dual sign-off を必須とする。CLAUDE.md のポリシーは機械的に読み込まれ、至高路線が適用される。

## CLAUDE.md ポリシー（root）

```markdown
# CLAUDE

## ポリシー
技術的な判断で悩んだ際は至高を目指す判断を優先すること。

## 変更履歴的な表現について
この文章が削除されるまで変更履歴を明文化しないでください。
```

「至高を目指す」 = 運用コスト度外視 / 規律最大化 / 段階的リリース禁止 / dimension override 禁止 / dead spec は CI で殺す。

## .claude/ 配下のスキル

| スキル名 | 主な用途 |
|---|---|
| `drawio-authoring` | drawio 図の作図・検証・エクスポート規約 |
| `figure-layer-convention` | 複数レイヤ drawio 図の記法規約 |
| `knowledge` | 技術学習用 Knowledge ドキュメント作成 |
| `update-config` | Claude Code の settings.json 設定 |
| `keybindings-help` | キーバインド設定 |
| `simplify` | コードレビューと改善 |
| `fewer-permission-prompts` | 許可プロンプト最小化 |
| `loop` | 定期実行タスク |
| `schedule` | スケジュール実行 |
| `claude-api` | Claude API / Anthropic SDK |
| `init` | CLAUDE.md 初期化 |
| `review` | PR レビュー |
| `security-review` | セキュリティレビュー |

## memory システム

`~/.claude/projects/.../memory/` 配下のファイルが会話間で永続化される。作者の意向や設計決定で「次回会話でも参照すべき非自明な事実」は自動的に保存される。

保存対象の例:
- 至高路線の運用方法（`feedback_supreme_path.md`）
- 19 軸同型構造の現状（`project_three_axis_isomorphism.md`）

保存不要なもの:
- コードパターンやアーキテクチャ（コード本体から読める）
- git 履歴（`git log` が権威）
- 進行中のタスク（会話内の TodoWrite で管理）

## LLM 補助での作業分担

| 作業 | LLM 可 | 人間 dual sign-off 必須 |
|---|---|---|
| docs/ の新規ページ草案 | ○ | - |
| frontmatter の生成 | ○ | - |
| drawio 図の drawio-lint 検証 | ○ | - |
| `frontmatter_schema.yaml` の変更 | 草案のみ | ○ |
| registry.yaml への entry 追加 | 草案のみ | ○ |
| `release_gate.lock.yaml` AND-gate 更新 | - | ○ |
| cosign 署名 | - | ○ |

## 検収コマンド

```bash
ls ~/.claude/projects/ 2>/dev/null && echo "memory dir 存在"
cat CLAUDE.md  # ポリシーの確認
```

## 関連参照

- [01_責務とスコープ](01_責務とスコープ.md)
- [00_k1s0作者 index](README.md)
