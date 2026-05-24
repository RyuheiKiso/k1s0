---
id: env.client.client_claude_code
axis: client
phase: env_setup
kind: policy
status: draft
depends_on:
  - env.client.client_acceptance
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# Claude Code 連携

## 一文方針

- client 軸エンジニアは Claude Code を SDK wrapper 草案 / Playwright テストスクリプト生成 / conformance test 草案の補助ツールとして活用するが、SDK public API の breaking change / distribution_class の追加・廃止 / conformance test pass 基準の変更は人間 dual sign-off を必須とする。

> **pre-P0 注記**: `src/` は P10 deliverable（pre-P0 時点で実体ゼロ）。以下の手順は P10 完了後に有効。

## CLAUDE.md ポリシー（root）

```markdown
# CLAUDE

## ポリシー
技術的な判断で悩んだ際は至高を目指す判断を優先すること。

## 変更履歴的な表現について
この文章が削除されるまで変更履歴を明文化しないでください。
```

「至高を目指す」 = 5 distribution_class 全て完備 / browser matrix 全ブラウザ対応 / SDK breaking change を安易に許容しない。

## client 軸エンジニアの LLM 活用範囲

| 作業 | LLM 可 | 人間 dual sign-off 必須 |
|---|---|---|
| SDK TypeScript wrapper 草案 | ○ | - |
| Playwright テストスクリプト生成 | ○ | - |
| cargo clippy 警告の修正提案 | ○ | - |
| C# SDK コード生成 | ○ | - |
| conformance test 草案 | ○ | - |
| SDK public API breaking change | 草案のみ | ○ |
| distribution_class の追加・廃止 | 草案のみ | ○ |
| conformance test pass 基準変更 | 草案のみ | ○ |
| WiX インストーラ設定変更 | 草案のみ | ○ |

## 利用可能なスキル

### プロジェクト固有スキル（`.claude/skills/` に配置）（client 軸で活用するもの）

| スキル名 | client 軸での用途 |
|---|---|
| `drawio-authoring` | 5 distribution_class アーキテクチャ図の作図 |
| `knowledge` | Tauri v2.0 / Playwright API 技術調査 |
| `review` | SDK PR レビュー |
| `security-review` | SDK のセキュリティ影響確認 |

## memory システムの活用

`~/.claude/projects/.../memory/` に記録すべき非自明な事実の例:

- 各 distribution_class のビルドの既知の問題と回避策
- browser matrix の既知の Webkit 制限事項
- SDK conformance test の例外ケース

## 検収コマンド

```bash
ls ~/.claude/projects/ 2>/dev/null && echo "memory dir 存在"
cat CLAUDE.md  # ポリシーの確認
```

## 関連参照

- [13_検収基準](13_検収基準.md)
- [01_責務とスコープ](01_責務とスコープ.md)
- [client 軸 index](README.md)
