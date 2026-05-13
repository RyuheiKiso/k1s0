---
id: env.data.data_claude_code
axis: data
phase: env_setup
kind: policy
status: draft
depends_on:
  - env.data.data_acceptance
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# Claude Code 連携

## 一文方針

- data 軸エンジニアは Claude Code を SQL 草案・スキーマ設計・migration スクリプト生成の補助ツールとして活用するが、production DB への migration 実行 / restore_drill の結果確定は人間確認を必須とする。

## CLAUDE.md ポリシー（root）の確認

```bash
cat CLAUDE.md
# ポリシー: 技術的な判断で悩んだ際は至高を目指す判断を優先する
```

「至高を目指す」 = 運用コスト度外視 / 規律最大化。data 軸では「とりあえず動く DB 設定」より「5 preservation_class が全て満たされ restore_drill が常時成功する最高可用性設計」を優先する。

## data 軸での LLM 補助の活用場面

| 作業 | LLM 可 | 人間確認必須 |
|---|---|---|
| PostgreSQL DDL の草案生成 | ○ | sqlfluff lint 後にマージ |
| Kafka topic 設定の草案 | ○ | クラスタ設定確認後にマージ |
| Apicurio スキーマの草案 | ○ | schema compatibility check 後に登録 |
| ClickHouse DDL の草案 | ○ | テスト環境確認後にマージ |
| migration スクリプトの草案 | ○ | dry-run + dual sign-off 必須 |
| production DB への migration 実行 | - | dual sign-off 必須 |
| restore_drill の結果確定 | - | 人間が手動確認 |
| backup ポリシーの変更 | 草案のみ | ○ |

## .claude/ 配下のスキル（data 軸で有用なもの）

| スキル名 | data での用途 |
|---|---|
| `drawio-authoring` | ER 図 / データフロー図の作図 |
| `figure-layer-convention` | 複数レイヤ（app/data/infra）の drawio 図 |
| `simplify` | SQL / スキーマのレビューと改善 |
| `review` | PR レビュー（migration スクリプト変更など） |
| `security-review` | データアクセス権限・暗号化の観点でのレビュー |

## memory システム

`~/.claude/projects/.../memory/` 配下で永続化される情報。data 軸で保存すべき非自明な事実の例:

- 5 preservation_class の設計決定（各 OSS に対する保全戦略の選択理由）
- restore_drill の頻度・手順の変更理由
- スキーマ evolution の backward 互換性判断基準

## 検収コマンド

```bash
ls ~/.claude/projects/ 2>/dev/null && echo "memory dir 存在"
cat CLAUDE.md
```

## 関連参照

- [01_責務とスコープ](01_責務とスコープ.md)
- [data index](README.md)
- [13_検収基準](13_検収基準.md)
