---
id: env.ops.ops_claude_code
axis: ops
phase: env_setup
kind: policy
status: draft
depends_on:
  - env.ops.ops_acceptance
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# Claude Code 連携

## 一文方針

- ops 軸エンジニアは Claude Code を runbook 草案 / SLO 計算スクリプト生成 / alert rule 草案作成の補助ツールとして活用するが、SLO target 変更 / escalation policy 変更 / postmortem action item の owner 決定は人間 dual sign-off を必須とする。

## CLAUDE.md ポリシー（root）

```markdown
# CLAUDE

## ポリシー
技術的な判断で悩んだ際は至高を目指す判断を優先すること。

## 変更履歴的な表現について
この文章が削除されるまで変更履歴を明文化しないでください。
```

「至高を目指す」 = 運用コスト度外視 / 規律最大化 / SLO target を安易に下げない / toil 自動化を先行させる。

## ops 軸エンジニアの LLM 活用範囲

| 作業 | LLM 可 | 人間 dual sign-off 必須 |
|---|---|---|
| runbook 草案作成 | ○ | - |
| SLO error budget 計算スクリプト | ○ | - |
| Prometheus alert rule 草案 | ○ | - |
| k6 load test スクリプト | ○ | - |
| Backstage catalog-info.yaml 草案 | ○ | - |
| SLO target 値の変更 | 草案のみ | ○ |
| on-call escalation policy の変更 | 草案のみ | ○ |
| Alertmanager routing rule の変更 | 草案のみ | ○ |
| postmortem action item の owner 決定 | - | ○ |

## 利用可能なスキル

### プロジェクト固有スキル（`.claude/skills/` に配置）（ops 軸で活用するもの）

| スキル名 | ops 軸での用途 |
|---|---|
| `drawio-authoring` | ops loop / SLO architecture 図の作図 |
| `knowledge` | 技術調査（Argo Rollouts / Backstage 仕様） |
| `review` | runbook / alert rule の PR レビュー |
| `security-review` | alert rule の security implications 確認 |

## memory システムの活用

`~/.claude/projects/.../memory/` 配下のファイルが会話間で永続化される。ops 軸エンジニアが記録すべき非自明な事実の例:

- SLO target の変更決定とその根拠
- on-call rotation の例外ルール
- 特定サービスの既知の false positive alert

## 検収コマンド

```bash
ls ~/.claude/projects/ 2>/dev/null && echo "memory dir 存在"
cat CLAUDE.md  # ポリシーの確認
```

## 関連参照

- [13_検収基準](13_検収基準.md)
- [01_責務とスコープ](01_責務とスコープ.md)
- [ops 軸 index](README.md)
