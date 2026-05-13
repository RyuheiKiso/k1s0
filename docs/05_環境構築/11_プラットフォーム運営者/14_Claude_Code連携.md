---
id: env.ops.platform_operator_claude_code
axis: ops
phase: env_setup
kind: policy
status: draft
depends_on:
  - env.ops.platform_operator_acceptance
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# Claude Code 連携

## 一文方針

- プラットフォーム運営者は Claude Code を runbook 草案作成 / audit log 初期分析の補助に活用するが、KEK ceremony の実行 / break-glass の実行判断 / audit hash chain の署名は LLM 単独で行うことを禁止し、人間が必ず執行する。

## 活用できる場面

| 作業 | LLM 可 | 人間必須 |
|---|---|---|
| runbook の草案作成 | ○ | - |
| audit log の初期分析・パターン抽出 | ○（補助のみ） | - |
| postmortem の初期草案作成 | ○ | - |
| docs/ の新規ページ草案 | ○ | - |
| KEK ceremony の実行 | 禁止 | ○（M 名の物理参加が必須） |
| break-glass の実行判断 | 禁止 | ○ |
| audit hash chain の署名 | 禁止 | ○ |
| Yubikey PIV スロットの変更 | 禁止 | ○ |

## CLAUDE.md ポリシーの適用

```markdown
# CLAUDE

## ポリシー
技術的な判断で悩んだ際は至高を目指す判断を優先すること。
```

「至高を目指す」 = 運用コスト度外視 / 規律最大化 / ceremony は物理的に人間が執行 / LLM 単独での鍵操作禁止。

## on-call 対応での LLM 活用

on-call アラート受信時に LLM を使って初動分析を補助することができる。ただし以下を厳守する：

- LLM の分析結果は「参考情報」であり、執行判断の代替にはならない
- break-glass の実行は人間の承認が必要
- LLM を使った分析内容は audit log に記録する

## 検収コマンド

```bash
ls ~/.claude/projects/ 2>/dev/null && echo "memory dir 存在"
cat CLAUDE.md  # ポリシーの確認
```

## 関連参照

- [01_責務とスコープ](01_責務とスコープ.md)
- [プラットフォーム運営者 index](README.md)
