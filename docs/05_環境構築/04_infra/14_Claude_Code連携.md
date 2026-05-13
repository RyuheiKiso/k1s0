---
id: env.infra.infra_claude_code
axis: infra
phase: env_setup
kind: policy
status: draft
depends_on:
  - env.infra.infra_acceptance
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# Claude Code 連携

## 一文方針

- infra 軸エンジニアは Claude Code を k8s manifest 草案・Kyverno policy 草案・IaC テンプレート生成の補助ツールとして活用するが、production cluster への Argo CD sync / Kyverno deny rule 追加は人間 dual sign-off を必須とする。

## CLAUDE.md ポリシー（root）の確認

```bash
cat CLAUDE.md
# ポリシー: 技術的な判断で悩んだ際は至高を目指す判断を優先する
```

「至高を目指す」 = 運用コスト度外視 / 規律最大化 / 段階的リリース禁止 / dead spec は CI で殺す。infra 軸では「とりあえず動く cluster」より「25+ Kyverno policy が全適用された最高セキュリティ cluster」を優先する。

## infra 軸での LLM 補助の活用場面

| 作業 | LLM 可 | 人間確認必須 |
|---|---|---|
| k8s manifest の草案生成 | ○ | kubeconform で検証後にマージ |
| Kyverno ClusterPolicy の草案 | ○ | audit mode で動作確認後に enforce mode へ |
| helm values.yaml の調整 | ○ | `helm lint` 後にマージ |
| Argo CD Application 定義 | ○ | dry-run 確認後にマージ |
| production cluster への sync | - | dual sign-off 必須 |
| Kyverno deny rule の追加（enforce） | 草案のみ | ○ |
| kind cluster 設定の変更 | ○ | - |
| IaC (Terraform/Pulumi) の草案 | ○ | plan 確認後にマージ |

## .claude/ 配下のスキル（infra 軸で有用なもの）

| スキル名 | infra での用途 |
|---|---|
| `drawio-authoring` | cluster topology / network 図の作図 |
| `figure-layer-convention` | 複数レイヤ（app/network/infra）の drawio 図 |
| `simplify` | manifest / policy のレビューと改善 |
| `review` | PR レビュー（Kyverno policy 変更など） |
| `security-review` | セキュリティ観点でのマニフェストレビュー |

## memory システム

`~/.claude/projects/.../memory/` 配下で永続化される情報。infra 軸で保存すべき非自明な事実の例:

- cluster topology の設計決定（なぜ 5 topology_class を採用したか）
- Kyverno policy の除外（exemption）の理由
- Argo CD ApplicationSet の設計方針

## 検収コマンド

```bash
ls ~/.claude/projects/ 2>/dev/null && echo "memory dir 存在"
cat CLAUDE.md
```

## 関連参照

- [01_責務とスコープ](01_責務とスコープ.md)
- [infra index](README.md)
- [13_検収基準](13_検収基準.md)
