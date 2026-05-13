---
id: env.test.test_claude_code
axis: test
phase: env_setup
kind: policy
status: draft
depends_on:
  - env.test.test_acceptance
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# Claude Code 連携

## 一文方針

- test 軸エンジニアは Claude Code を property-based test 草案 / Pact contract 草案 / Litmus experiment YAML 草案の補助ツールとして活用するが、mutation score 閾値の変更 / contract test pass 基準変更 / chaos experiment の blast radius 拡大は人間 dual sign-off を必須とする。

## CLAUDE.md ポリシー（root）

```markdown
# CLAUDE

## ポリシー
技術的な判断で悩んだ際は至高を目指す判断を優先すること。

## 変更履歴的な表現について
この文章が削除されるまで変更履歴を明文化しないでください。
```

「至高を目指す」 = 95 cell coverage matrix を妥協なく埋める / mutation score 閾値を安易に下げない / chaos experiment の blast radius を慎重に管理する。

## test 軸エンジニアの LLM 活用範囲

| 作業 | LLM 可 | 人間 dual sign-off 必須 |
|---|---|---|
| property-based test 草案（Hypothesis / fast-check / proptest） | ○ | - |
| Pact contract YAML 草案 | ○ | - |
| Litmus chaos experiment YAML 草案 | ○ | - |
| pytest / Vitest テストコード生成 | ○ | - |
| cargo-mutants 実行スクリプト | ○ | - |
| mutation score 閾値の変更 | 草案のみ | ○ |
| contract test pass 基準変更 | 草案のみ | ○ |
| chaos experiment blast radius 拡大 | 草案のみ | ○ |
| coverage matrix の cell 除外 | 草案のみ | ○ |

## 利用可能なスキル

### プロジェクト固有スキル（`.claude/skills/` に配置）（test 軸で活用するもの）

| スキル名 | test 軸での用途 |
|---|---|
| `drawio-authoring` | 95 cell coverage matrix 可視化図の作図 |
| `knowledge` | Pact / Litmus / Hypothesis 仕様の技術調査 |
| `review` | test コード PR レビュー |
| `security-review` | chaos experiment の security implications 確認 |

## memory システムの活用

`~/.claude/projects/.../memory/` に記録すべき非自明な事実の例:

- 各軸の mutation score の既知の false positive パターン
- Pact contract の既知の破れパターンと対処
- Litmus chaos experiment の安全な blast radius 設定
- 95 cell matrix で意図的に除外した cell とその根拠

## 検収コマンド

```bash
ls ~/.claude/projects/ 2>/dev/null && echo "memory dir 存在"
cat CLAUDE.md  # ポリシーの確認
```

## 関連参照

- [13_検収基準](13_検収基準.md)
- [01_責務とスコープ](01_責務とスコープ.md)
- [test 軸 index](README.md)
