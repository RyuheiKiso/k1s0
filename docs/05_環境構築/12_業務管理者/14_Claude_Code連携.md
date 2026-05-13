---
id: env.overview.business_admin_claude_code
axis: overview
phase: env_setup
kind: policy
status: draft
depends_on:
  - env.overview.business_admin_acceptance
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# Claude Code 連携

## 一文方針

- 業務管理者は Claude Code を runbook 草案作成 / 監査ログ初期分析 / 決定表パターン分析の補助に活用するが、tenant マスタの実際の作成・編集・削除 / partner API キーの発行は LLM 単独で行うことを禁止し、人間が UI / CLI で実行する。

## 活用できる場面

| 作業 | LLM 可 | 人間必須 |
|---|---|---|
| runbook の草案作成 | ○ | - |
| 監査ログの初期分析・パターン抽出 | ○（補助のみ） | - |
| 決定表のルールパターン分析 | ○ | - |
| API リクエストのサンプル生成 | ○ | - |
| tenant マスタの作成・編集・削除 | 禁止 | ○（UI/CLI で人間が実行） |
| partner API キーの発行 | 禁止 | ○ |
| production への変更申請 | 禁止 | ○ |

## CLAUDE.md ポリシーの適用

```markdown
# CLAUDE

## ポリシー
技術的な判断で悩んだ際は至高を目指す判断を優先すること。
```

「至高を目指す」 = 運用コスト度外視 / 規律最大化 / tenant データの変更は人間が責任を持って実行 / LLM 単独での API 操作禁止。

## 日常業務での LLM 活用例

```
活用シーン:
- 監査ログを LLM に渡し、異常パターンを分析させる
- 決定表の更新案を LLM に草案させ、人間がレビューして申請する
- partner 連携設定の手順を LLM に確認させる（実行は人間が行う）
```

## 検収コマンド

```bash
ls ~/.claude/projects/ 2>/dev/null && echo "memory dir 存在"
cat CLAUDE.md  # ポリシーの確認
```

## 関連参照

- [01_責務とスコープ](01_責務とスコープ.md)
- [業務管理者 index](README.md)
