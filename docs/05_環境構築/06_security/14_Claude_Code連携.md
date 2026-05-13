---
id: env.security.security_claude_code
axis: security
phase: env_setup
kind: policy
status: draft
depends_on:
  - env.security.security_acceptance
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# Claude Code 連携

## 一文方針

- security 軸エンジニアは Claude Code を脅威モデル草案・セキュリティポリシー草案・penetration testing レポート生成の補助ツールとして活用するが、SC-7 / SC-8 の鍵材料操作 / Kyverno deny rule（enforce mode）追加 / penetration testing 結果の公式確認は人間 dual sign-off を必須とする。

## CLAUDE.md ポリシー（root）の確認

```bash
cat CLAUDE.md
# ポリシー: 技術的な判断で悩んだ際は至高を目指す判断を優先する
```

「至高を目指す」 = 運用コスト度外視 / 規律最大化。security 軸では「とりあえず動くセキュリティ設定」より「5⁴ cell 脅威モデルが全て評価され 8 secret class が全て適切に管理された最高セキュリティ設計」を優先する。

## security 軸での LLM 補助の活用場面

| 作業 | LLM 可 | 人間確認必須 |
|---|---|---|
| 脅威モデル 5⁴ cell の draft 作成 | ○ | セキュリティ専門家レビュー後に確定 |
| セキュリティポリシー文書の草案 | ○ | dual sign-off 後に確定 |
| Semgrep ルールの草案 | ○ | テスト環境で検証後にマージ |
| checkov / conftest policy 草案 | ○ | テスト環境で検証後にマージ |
| penetration testing チェックリスト | ○ | 人間が実際にテストして確認 |
| OWASP ZAP スキャン設定 | ○ | 対象環境の承認を得てから実行 |
| SC-7 / SC-8 の鍵材料操作 | - | dual sign-off 必須（LLM は禁止） |
| Kyverno deny rule（enforce mode） | 草案のみ | ○ |
| penetration testing 結果の公式確認 | - | 人間が確認 |
| incident response の実行 | - | 人間が実行 |

## セキュリティ上の注意

LLM を使う際に以下のものを入力しない:

- 実際の secret / credential / 鍵マテリアル
- production システムのアクセス情報
- 個人情報・機密情報を含むログ
- 未公開の脆弱性情報

## 利用可能なスキル

### プロジェクト固有スキル（`.claude/skills/` に配置）（security 軸で有用なもの）

| スキル名 | security での用途 |
|---|---|
| `security-review` | セキュリティ観点でのコードレビュー |
| `review` | PR レビュー（セキュリティ設定変更など） |
| `drawio-authoring` | 脅威モデル図 / セキュリティアーキテクチャ図 |
| `figure-layer-convention` | 複数レイヤ（app/network/security）の drawio 図 |
| `simplify` | セキュリティポリシー文書のレビューと改善 |

## memory システム

`~/.claude/projects/.../memory/` 配下で永続化される情報。security 軸で保存すべき非自明な事実の例:

- 5⁴ cell 脅威モデルの設計根拠（なぜ特定の cell を high risk と評価したか）
- 8 secret class の運用決定（各 class のローテーション頻度など）
- penetration testing で発見した既知の false positive パターン

## 検収コマンド

```bash
ls ~/.claude/projects/ 2>/dev/null && echo "memory dir 存在"
cat CLAUDE.md
```

## 関連参照

- [01_責務とスコープ](01_責務とスコープ.md)
- [security index](README.md)
- [13_検収基準](13_検収基準.md)
