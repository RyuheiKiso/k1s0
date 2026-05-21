---
id: env.tier3.tier3_claude_code
axis: tier3
phase: env_setup
kind: policy
status: published
depends_on:
  - env.tier3.tier3_acceptance
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# Claude Code 連携

## 一文方針

- tier3 エンジニアは Claude Code を補助ツールとして活用するが、業務シナリオの画面フロー定義 / 帳票レイアウトの最終承認は LLM 単独でなく人間 dual sign-off を必須とする。

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

tier3 エンジニアが主に使用するスキル:

| スキル名 | 主な用途 |
|---|---|
| `drawio-authoring` | UI フロー図 / 画面遷移図の作図・検証 |
| `figure-layer-convention` | 複数レイヤ図の記法規約 |
| `simplify` | TypeScript / React / C# コードレビューと改善 |
| `security-review` | UI セキュリティ（XSS / CSRF 等）レビュー |
| `review` | PR レビュー |

## LLM 補助での作業分担

| 作業 | LLM 可 | 人間 dual sign-off 必須 |
|---|---|---|
| docs/ の新規ページ草案 | ○ | - |
| frontmatter の生成 | ○ | - |
| React / TypeScript コンポーネント草案 | ○ | - |
| WPF XAML レイアウト草案 | ○ | - |
| Tauri コマンド実装草案 | ○ | - |
| 業務シナリオ画面フロー定義 | 草案のみ | ○ |
| 帳票レイアウト最終承認 | - | ○ |
| Playwright E2E シナリオ設計 | 草案のみ | ○ |

## 検収コマンド

```bash
ls ~/.claude/projects/ 2>/dev/null && echo "memory dir 存在"
cat CLAUDE.md
```

## 関連参照

- [01_責務とスコープ](01_責務とスコープ.md)
- [tier3 index](README.md)
