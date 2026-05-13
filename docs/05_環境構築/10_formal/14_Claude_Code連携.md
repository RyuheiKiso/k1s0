---
id: env.formal.formal_claude_code
axis: formal
phase: env_setup
kind: policy
status: draft
depends_on:
  - env.formal.formal_acceptance
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# Claude Code 連携

## 一文方針

- formal 軸エンジニアは Claude Code を spec 草案作成 / proof_matrix YAML 生成補助に活用するが、proof_matrix cell の green 判定・counter-example 閉鎖の最終確認・cryptographic proof のレビューは LLM 単独で行うことを禁止し、人間 dual sign-off を必須とする。

## 活用できる場面

| 作業 | LLM 可 | 人間 dual sign-off 必須 |
|---|---|---|
| TLA+ / Dafny / Lean 4 spec 草案の生成 | ○ | - |
| proof_matrix YAML 雛形の生成 | ○ | - |
| counter-example の初期分析 | ○（補助のみ） | - |
| docs/ の新規ページ草案 | ○ | - |
| proof_matrix cell を green に変更 | 草案のみ | ○ |
| cryptographic proof の合否判定 | - | ○（domain expert 必須） |
| counter-example closure の最終確認 | - | ○ |
| formal15 軸の cell 変更 | - | ○ |

## CLAUDE.md ポリシーの適用

```markdown
# CLAUDE

## ポリシー
技術的な判断で悩んだ際は至高を目指す判断を優先すること。
```

「至高を目指す」 = 運用コスト度外視 / 規律最大化 / LLM 単独の proof 判定禁止 / dual reviewer sign-off の厳守。

## formal 軸固有のスキル活用

`.claude/skills/` 配下のスキルのうち、formal 軸エンジニアが主に使うもの:

| スキル | 用途 |
|---|---|
| `knowledge` | 形式検証手法の技術学習ドキュメント作成 |
| `review` | PR レビュー時の spec チェック補助 |
| `security-review` | 暗号プリミティブを含む proof の security 観点レビュー |

## dual reviewer フローでの LLM 活用

dual reviewer フローにおいて、LLM はレビューの「補助ツール」として使うことができる。ただし以下を厳守する：

- LLM の出力は「一次確認の補助」であり、sign-off の代替にはならない
- LLM が「証明が正しい」と判定しても、人間 2 名の sign-off がなければ cell は green にならない
- LLM の出力を含む PR body は、LLM を使った旨を明示する

## 検収コマンド

```bash
ls ~/.claude/projects/ 2>/dev/null && echo "memory dir 存在"
cat CLAUDE.md  # ポリシーの確認
```

## 関連参照

- [01_責務とスコープ](01_責務とスコープ.md)
- [formal index](README.md)
