---
id: env.tier1.tier1_claude_code
axis: tier1
phase: env_setup
kind: policy
status: draft
depends_on:
  - env.tier1.tier1_acceptance
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# Claude Code 連携

## 一文方針

- tier1 エンジニアは Claude Code を補助ツールとして活用するが、proto / OSS 定義の schema 変更は LLM 単独でなく人間 dual sign-off を必須とする。CLAUDE.md のポリシーは機械的に読み込まれ、至高路線が適用される。

## CLAUDE.md ポリシー（root）

```markdown
# CLAUDE

## ポリシー
技術的な判断で悩んだ際は至高を目指す判断を優先すること。

## 変更履歴的な表現について
この文章が削除されるまで変更履歴を明文化しないでください。
```

「至高を目指す」 = 運用コスト度外視 / 規律最大化 / 段階的リリース禁止。

## .claude/ 配下のスキル

tier1 エンジニアが主に使用するスキル:

| スキル名 | 主な用途 |
|---|---|
| `drawio-authoring` | tier1 アーキテクチャ図の作図・検証 |
| `figure-layer-convention` | 複数レイヤ図の記法規約 |
| `simplify` | Rust / Go / C# / TypeScript コードレビューと改善 |
| `security-review` | セキュリティレビュー |
| `review` | PR レビュー |

## LLM 補助での作業分担

| 作業 | LLM 可 | 人間 dual sign-off 必須 |
|---|---|---|
| docs/ の新規ページ草案 | ○ | - |
| frontmatter の生成 | ○ | - |
| Rust / Go / C# / TypeScript コード草案 | ○ | - |
| `buf.yaml` / `buf.gen.yaml` の変更 | 草案のみ | ○ |
| proto スキーマの破壊的変更 | - | ○ |
| OSS バージョン固定変更 | 草案のみ | ○ |

## 検収コマンド

```bash
ls ~/.claude/projects/ 2>/dev/null && echo "memory dir 存在"
cat CLAUDE.md
```

## 関連参照

- [01_責務とスコープ](01_責務とスコープ.md)
- [tier1 index](README.md)
