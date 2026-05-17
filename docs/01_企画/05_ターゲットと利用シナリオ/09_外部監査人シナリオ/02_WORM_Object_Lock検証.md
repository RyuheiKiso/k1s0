---
id: plan.overview.scenario_external_auditor_worm_verification
axis: overview
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.security.security_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [D, E]
  proof_classes: []
---

# WORM_Object_Lock検証

## 一文方針

WORM retention period 中の物理書込不可能性を独立確認する。

> 年次監査の 2 日目、外部監査人は audit ログの長期保存が物理的に改竄不可能であることを独立確認する。AWS S3 / Azure Blob の Object Lock 設定を外部監査人専用 read-only credential で参照し、retention mode（COMPLIANCE）と retention period が要件通りであることを確認する。retention period 内の上書き / 削除を実際に試みて拒否されることを確認し、その証跡を chain-of-custody に記録する。

## ペルソナ要約

主役: 外部監査人（公認会計士 / 監査法人スタッフ）、目的: WORM Object Lock が設定通りに機能しており、retention period 中の物理書込が不可能であることを独立に確認する

## 現状業務での痛み

- ストレージの WORM 設定が「設定されている」という主張だけで証明されることが多く、実際に物理書込が拒否されることを独立に確認する手順が存在しない
- COMPLIANCE モードと GOVERNANCE モードの違いを監査人が理解していないため、適切な強度の WORM が適用されているかの判断ができない
- retention period の設定値が規制要件（例: 7 年保存）と一致しているかを定量的に確認する手順がない
- WORM 検証の記録が口頭確認にとどまり、監査報告書に証拠として記載できない

## k1s0 でこう変わる

- COMPLIANCE モードの Object Lock が適用されており、ルートアカウントを含む誰も retention period 中は削除 / 上書きできないことを AWS / Azure の API で独立確認できる
- retention period が規制要件（GDPR / 食品衛生法 / ISO 9001 等）に対応して設定されており、設定値の根拠ドキュメントが提供される
- 外部監査人が read-only credential で Object Lock 設定を直接参照できるため、内部関係者の説明に依存しない
- 上書き試行の拒否記録が API レスポンスとして取得でき、chain-of-custody に記録できる

## Trigger

WORM 保存期間中の物理書込不可能性確認時（年次監査スケジュールの一部として実施）

## 想定頻度 / 典型きっかけ / 頻度根拠

- 想定頻度: 年次
- 典型きっかけ: 「年次監査で audit ログの物理的改竄不可能性の証明が必要になった」「規制当局からデータ保存の integrity 証明を求められた」

## 主役 / 関与者

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|------|---|------------|----------------|----------------------|
| 外部監査人（主役） | 外部専門家 | 監査現場 / リモート | CLI 環境（aws s3api / azure storage） | Object Lock 設定確認・書込拒否確認 |
| security 担当者 | シニア | 開発拠点 | セキュリティダッシュボード | read-only credential の発行・技術説明 |

## 個人 KPI / 達成感

- Object Lock 設定確認完了（COMPLIANCE モード / retention period の定量確認）
- 書込拒否の API レスポンス取得と chain-of-custody への記録
- 検証完了から監査報告書への記載完了までの所要時間（目標: 半日以内）

## 工数 / 関与人数 / コスト感

- 標準的な WORM 検証: 2〜4 時間
- 複数バケット / コンテナの確認が必要な場合: 半日
- 関与人数: 1〜2 名（外部監査人のみ、または補助者 1 名）

## 前提

- 外部監査人に AWS S3 / Azure Blob への read-only credential が発行されている
- WORM が適用されているバケット / コンテナのリストが提供されている
- k1s0 の retention period 設定根拠ドキュメントが提供されている

## 流れ

1. security 担当者から WORM 適用バケット / コンテナのリストと read-only credential を受け取る
2. AWS CLI / Azure CLI で Object Lock 設定を参照し、COMPLIANCE モードと retention period を確認する
3. retention period が規制要件（設定根拠ドキュメントと照合）と一致していることを確認する
4. テスト用のダミーオブジェクトを read-only credential では書き込めないことを確認する（拒否される）
5. COMPLIANCE モードの Object Lock では書込拒否の API エラーを取得し、chain-of-custody に記録する
6. 検証結果（設定確認 / 拒否確認）を chain-of-custody ドキュメントに記録する
7. 監査報告書の Draft に物理書込不可能性の独立確認結果を記載する

## Timeline

| T+ | actor | action | 通知例 |
|----|-------|--------|--------|
| T+0m | 外部監査人 | credential 受領・CLI 設定 | — |
| T+20m | 外部監査人 | Object Lock 設定参照（全バケット） | — |
| T+60m | 外部監査人 | retention period と規制要件の照合 | — |
| T+70m | 外部監査人 | 書込拒否確認・API エラー取得 | CLI: `AccessDenied: Object Lock configuration prevents action` |
| T+90m | 外部監査人 | chain-of-custody 記録・監査 Draft 記載 | — |

## 業界 9 業務との紐付け

| 業務名 | 影響度 | 紐付き内容 |
|--------|--------|----------|
| 品質検査結果配信 | 高 | 品質検査記録の長期保存が GMP / ISO 9001 の retention 要件の対象 |
| FA 生産指示 | 中 | 生産指示記録の WORM 保存が食品 / 製薬規制の audit trail 要件を満たすか |

## 独立性宣言

外部監査人は本シナリオにおいても独立性宣言を確認済みであること。WORM 検証のために取得した read-only credential で書込以外の操作（読み取り / 設定確認）のみを実施し、検証結果を改変せずに報告書に記載する。

## 証拠保全 chain-of-custody

| 証拠番号 | 証拠種別 | 取得日時 | 取得方法 | 保管場所 | 完全性確認 |
|---------|---------|---------|---------|---------|-----------|
| EVD-010 | Object Lock 設定 JSON | 取得日時を記録 | aws s3api get-object-lock-configuration | 監査人ラップトップ（暗号化） | SHA-256 hash を記録 |
| EVD-011 | 書込拒否 API エラーログ | 確認日時を記録 | CLI 出力 | 監査人ラップトップ（暗号化） | SHA-256 hash を記録 |
| EVD-012 | retention period 設定根拠ドキュメント | 受領日時を記録 | 業務管理者から受領 | 監査人ラップトップ（暗号化） | SHA-256 hash を記録 |

## 監査人側 tool セット

```bash
# S3 Object Lock 設定確認
aws s3api get-object-lock-configuration \
  --bucket k1s0-audit-logs-prod \
  --profile auditor-readonly

# 出力例:
# {"ObjectLockConfiguration":{"ObjectLockEnabled":"Enabled",
#   "Rule":{"DefaultRetention":{"Mode":"COMPLIANCE","Years":7}}}}

# 書込拒否確認（read-only credential では PutObject が拒否される）
aws s3api put-object \
  --bucket k1s0-audit-logs-prod \
  --key test-write-check.txt \
  --body /dev/null \
  --profile auditor-readonly
# Expected: AccessDenied
```

## 関連適合仕様

- [データ保全適合仕様](../../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md)

## 期待結果 / 観測指標 / 受入条件

- COMPLIANCE モードの Object Lock が全対象バケット / コンテナに設定されていることを確認
- retention period が規制要件（設定根拠ドキュメントと一致）と一致していることを確認
- 書込拒否の API エラーが chain-of-custody に記録されている
- 受入条件: 外部監査人が「物理書込不可能性」を独立に確認できたことを監査報告書に記載できること

## 失敗時の挙動 / escalation

- **COMPLIANCE モードではなく GOVERNANCE モードが設定されている**: GOVERNANCE モードは特権ユーザが削除できるため、COMPLIANCE モードより保護強度が低い。外部監査人は書面で security 担当者に COMPLIANCE への変更を勧告する
- **retention period が規制要件より短い**: 設定根拠ドキュメントとの不一致を書面で報告し、修正を勧告する
- **read-only credential での設定参照が拒否される**: credential 権限が不足している。security 担当者に必要な読み取り権限の付与を依頼する

## 失敗パターン（3 例）

1. **COMPLIANCE / GOVERNANCE モードの違いを確認せずに「WORM 設定あり」と報告する**: GOVERNANCE モードは特権ユーザが削除できるため、改竄不可能性の証明にならない。必ずモードを明示して報告する
2. **retention period の数値のみ確認して規制要件との照合をスキップする**: 規制要件（例: GMP の 3 年保存）と設定値（例: 1 年）が一致していない場合に気づかない。設定根拠ドキュメントとの照合を必須ステップとする
3. **書込拒否確認を実施せずに設定確認のみで完了とする**: 設定が有効でも実際に書込が拒否されることの確認が抜けると、設定の形式的確認にとどまる。書込拒否の API レスポンス取得を必須ステップとする

## 関連参照

- [外部監査人シナリオ INDEX](README.md)
- [01_audit_hash_chain独立検証](01_audit_hash_chain独立検証.md)
- [10_監査報告書発行_証跡保全](10_監査報告書発行_証跡保全.md)
