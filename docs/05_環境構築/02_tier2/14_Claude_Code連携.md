---
id: env.tier2.tier2_claude_code
axis: tier2
phase: env_setup
kind: policy
status: draft
depends_on:
  - env.tier2.tier2_acceptance
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# Claude Code 連携

## 一文方針

- tier2 エンジニアは Claude Code を補助ツールとして活用するが、Domain Event schema の破壊的変更 / 業務マスタの変更は LLM 単独でなく人間 dual sign-off を必須とする。

## CLAUDE.md ポリシー（root）

```markdown
# CLAUDE

## ポリシー
技術的な判断で悩んだ際は至高を目指す判断を優先すること。

## 変更履歴的な表現について
この文章が削除されるまで変更履歴を明文化しないでください。
```

## 利用可能なスキル

### プロジェクト固有スキル（`.claude/skills/` に配置）

tier2 エンジニアが主に使用するスキル:

| スキル名 | 主な用途 |
|---|---|
| `simplify` | Rust / Go / C# / TypeScript コードレビューと改善 |
| `security-review` | 業務ロジック・DB 操作のセキュリティレビュー |
| `review` | PR レビュー |
| `drawio-authoring` | 業務フロー図の作図・検証 |

## LLM 補助での作業分担

| 作業 | LLM 可 | 人間 dual sign-off 必須 |
|---|---|---|
| docs/ の新規ページ草案 | ○ | - |
| frontmatter の生成 | ○ | - |
| Domain Event スキーマ草案 | ○ | - |
| 決定表 / Workflow 草案 | ○ | - |
| Domain Event schema 破壊的変更 | - | ○ |
| 業務マスタの変更 | 草案のみ | ○ |
| DB schema の破壊的マイグレーション | - | ○ |
| atomic 三表書込の実装変更 | 草案のみ | ○ |

## 検収コマンド

```bash
ls ~/.claude/projects/ 2>/dev/null && echo "memory dir 存在"
cat CLAUDE.md
```

## 関連参照

- [01_責務とスコープ](01_責務とスコープ.md)
- [tier2 index](README.md)
