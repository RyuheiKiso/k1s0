---
runbook_id: RB-COMP-001
title: 法的開示対応（令状 / 提出命令 / ディスカバリー要求）
category: COMP
severity: SEV1
owner: 起案者（法務 と 共同所有）
automation: manual
alertmanager_rule: 該当なし（書面受領による起動）
fmea_id: 該当なし
estimated_recovery: 法的プロセスは期限が書面で指定される（通常 14〜30 日）
last_updated: 2026-05-02
---

# RB-COMP-001: 法的開示対応（令状 / 提出命令 / ディスカバリー要求）

本 Runbook は捜査機関 / 裁判所 / 規制当局からの法的書面（令状・提出命令・ディスカバリー要求等）受領時の対応を定める。
NFR-E-SIR-003 / NFR-G-CLS-001 に対応する。
**技術部門が単独で資料を提供することは厳禁**。法務担当の判断・承認を必ず経ること。

## 1. 前提条件

- 実行者は `legal-officer` または起案者で、CTO・法務担当との連絡経路を確立済み。
- 必要ツール: `mc`（MinIO CLI）/ `logcli` / `kubectl` / `gpg`（証跡暗号化用）/ `sha256sum`。
- kubectl context が `k1s0-prod`。
- MinIO `k1s0-legal-hold` バケットが Object Lock 有効状態で存在すること。
- `ops/legal-records/` ディレクトリが暗号化対象として SOPS で管理されていること（リリース時点では未存在のため初回起動時に整備）。
- 外部法律顧問の連絡先が [`oncall/contacts.md`](../../oncall/contacts.md) §セキュリティ・法務 に登録済み。

## 2. 対象事象

以下のいずれかで起動する:

- 捜査機関（警察・検察・税務当局等）からの **捜索差押許可状 / 提出命令** の受領
- 裁判所からの **文書提出命令 / 証拠保全命令** の受領
- 民事訴訟の相手方弁護士からの **ディスカバリー要求** の受領
- 規制当局（個人情報保護委員会・金融庁等）からの **立入検査予告 / 資料提出要求** の受領
- `legal@<org>.example` 宛ての公式書面受領

**受領後の第一動作**: いかなる対応も法務担当の判断前に行わないこと。
技術部門が単独で資料を提供することは厳禁。

通知経路: 書面受領者 → 法務担当（即時）→ CTO（24h 以内）→ 必要に応じ外部法律顧問。

## 3. 初動手順（5 分以内）

最初の 5 分で書面の真正性を確認し、法務担当への連絡を完了する。

```bash
# 受領書面のスキャン / 写真をデジタル化
# （技術手順なし、書面受領担当者が実施）
```

ステークホルダー通知（必須）:

- 法務担当（[`oncall/contacts.md`](../../oncall/contacts.md) §セキュリティ・法務）にメール + 電話で連絡。
- CTO に Slack DM で「法的書面受領、法務対応中」を投稿。
- 書面受領者は **何も行動せず** 法務判断を待つ（技術操作禁止）。

## 4. 原因特定手順

本 Runbook においては「原因」ではなく **書面の有効性確認** を実施する:

- 令状の場合: 管轄裁判所・裁判官署名・発行日・有効期限・対象範囲を確認する。
- 不審な場合（非公式要求・口頭要求）: 正式書面の提出を求め、口頭では応じない。
- 規制当局からの場合: 立入検査予告期間の確認、対応窓口の確定。
- 民事訴訟の場合: 訴訟番号 / 相手方代理人 / 文書提出命令の根拠条文を確認。

法務判断が必要な場合は **外部法律顧問** に依頼（[`contacts.md`](../../oncall/contacts.md)）。

## 5. 復旧手順

### Step 1: 法務担当への即時連絡（〜1 時間）

書面を受領した担当者は内容を複写・デジタル化して法務担当に転送する:

| 連絡先 | 手段 | 対応時間 |
|---|---|---|
| 法務担当 | [`contacts.md`](../../oncall/contacts.md) §セキュリティ・法務 | 24h 以内（平日）/ 翌営業日 |
| CTO | 電話 + Slack DM | 書面到達後即時 |
| 外部法律顧問 | 法務担当判断で依頼 | 法務判断 |

### Step 2: 証跡保全の即時指示（〜2 時間）

法務担当の指示のもと、対象データの削除・上書きを防ぐ **リーガルホールド** を設定:

```bash
# MinIO Object Lock でリーガルホールドを設定
mc legalhold set minio/k1s0-logs/<target-path>/ ON
mc legalhold set minio/k1s0-audit/<target-path>/ ON
# 現在の保持設定を確認
mc legalhold info minio/k1s0-logs/<target-path>/
```

対象範囲に含まれるデータを定期削除ジョブから除外:

```bash
# Loki retention ルールから対象期間を一時除外
kubectl edit configmap loki-retention -n monitoring
# 対象期間のログを別バケットにコピー
mc cp --recursive minio/k1s0-logs/tier1/<date-range>/ \
  minio/k1s0-legal-hold/incident-<case-id>/
```

### Step 3: 対象データの抽出（法務指示後のみ実施）

法務担当の **書面による承認** を得てから以下を実施:

```bash
# 対象 tenant / user / 期間の監査ログを抽出
logcli query '{namespace="k1s0-tier1", job="audit"}
  | json | tenant_id="<target-tenant>" | user_id="<target-user>"' \
  --from="<start-rfc3339>" --to="<end-rfc3339>" \
  --output jsonl > /tmp/legal-export-<case-id>.jsonl
# ハッシュで完全性を保証
sha256sum /tmp/legal-export-<case-id>.jsonl \
  > /tmp/legal-export-<case-id>.jsonl.sha256
```

提供データを暗号化し、チェーン・オブ・カストディ記録を作成:

```bash
gpg --encrypt --recipient legal@<org>.example \
  /tmp/legal-export-<case-id>.jsonl
```

### Step 4: 提供・非提供の判断ログ

以下のテンプレートで判断記録を作成（`ops/legal-records/<case-id>/` に保管、SOPS 暗号化）:

```
ケース ID: <CASE-YYYYMMDD-001>
書面種別: 捜索差押許可状 / 文書提出命令 / その他
受領日時: YYYY-MM-DD HH:MM JST
発行機関: <機関名>
対象期間: YYYY-MM-DD 〜 YYYY-MM-DD
対象内容: <概要>
法務判断: 対応可 / 異議申立 / 追加確認要
CTO 承認: 承認済 / 未承認
提供日時: YYYY-MM-DD（未提供の場合は「-」）
提供物: <内容・ファイル名>
担当者: <名前>
```

### Step 5: 法的手続終了後

法的手続の終了後、リーガルホールドを解除する（法務承認必須）:

```bash
mc legalhold clear minio/k1s0-legal-hold/incident-<case-id>/
```

ケース ID に紐づく全対応記録を `ops/legal-records/<case-id>/` に集約。
外部提供したデータの内容と提供先を記録し、7 年間保管する。

## 6. 検証手順

対応完了の判定基準:

- 法務担当の最終承認サインが `ops/legal-records/<case-id>/disposition.md` に記録されている。
- リーガルホールドが Step 5 で解除済み（法的手続終了後）、または有効期間内で継続中。
- 提供データのハッシュ（`sha256sum`）が受領機関に通知済み（チェーン・オブ・カストディ）。
- ケース記録が SOPS 暗号化された状態で `ops/legal-records/<case-id>/` に保管。
- 7 年間保管設定が MinIO Lifecycle Policy で確認済み。
- ポストレビュー（1 週間以内）で対応プロセスの問題が抽出済み。

## 7. 予防策

- 法務担当とのポストレビュー（1 週間以内）。
- ケース記録の 7 年保管設定（MinIO Lifecycle Policy）。
- 外部法律顧問との定期連絡（四半期）。
- データ保持ポリシーの見直し（リーガルホールドがかかりやすいデータ種別の保持期間調整）。
- 全担当者への「書面受領時の禁止事項」研修（半期 1 回）。
- 月次 Chaos Drill 対象に「法的書面受領シミュレーション」を追加（table-top 演習）。

## 8. 関連 Runbook

- 関連設計書: [`docs/03_要件定義/30_非機能要件/E_セキュリティ.md`](../../../docs/03_要件定義/30_非機能要件/E_セキュリティ.md) (NFR-E-SIR-003)、[`G_データ保護とプライバシー.md`](../../../docs/03_要件定義/30_非機能要件/G_データ保護とプライバシー.md) (NFR-G-CLS-001)
- 関連 ADR: [ADR-SEC-002 (OpenBao)](../../../docs/02_構想設計/adr/ADR-SEC-002-openbao.md), [ADR-SEC-003 (SPIRE)](../../../docs/02_構想設計/adr/ADR-SEC-003-spire.md)
- 連鎖 Runbook:
  - [`RB-COMP-002-pii-regulatory-disclosure.md`](RB-COMP-002-pii-regulatory-disclosure.md) — PII 漏えい後の規制報告
  - [`RB-SEC-005-pii-leak-detection.md`](RB-SEC-005-pii-leak-detection.md) — PII 漏えい起因の場合
- エスカレーション: [`../../oncall/escalation.md`](../../oncall/escalation.md)
- 連絡先: [`../../oncall/contacts.md`](../../oncall/contacts.md) §セキュリティ・法務
