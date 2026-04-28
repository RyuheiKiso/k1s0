# 法的開示対応 Runbook

> **severity**: SEV1
> **owner**: legal
> **estimated_mttr**: N/A（法的プロセスは期限が書面で指定される）
> **last_updated**: 2026-04-28

## 1. 検出 (Detection)

以下のいずれかで起動する。

- 捜査機関（警察・検察・税務当局等）からの **捜索差押許可状 / 提出命令** の受領
- 裁判所からの **文書提出命令 / 証拠保全命令** の受領
- 民事訴訟の相手方弁護士からの **ディスカバリー要求** の受領
- 規制当局（個人情報保護委員会・金融庁等）からの **立入検査予告 / 資料提出要求** の受領
- `legal@k1s0.io` 宛ての公式書面受領

**受領後の第一動作**: いかなる対応も法務担当の判断前に行わないこと。
技術部門が単独で資料を提供することは厳禁。

## 2. 初動 (Immediate Action)

### Step 1: 法務担当への即時連絡（〜1 時間）

1. 書面を受領した担当者は内容を複写・デジタル化して法務担当に転送する。

   | 連絡先 | 手段 | 対応時間 |
   |---|---|---|
   | 法務担当 | `legal@k1s0.io` + 電話（要置換） | 24h 以内（平日）/ 翌営業日 |
   | CTO | 電話 + Slack DM | 書面到達後即時 |
   | 外部法律顧問 | `outside-counsel@<law-firm>.example`（要置換） | 法務担当判断で依頼 |

2. 法務担当は書面の **有効性・合法性** を確認する。
   - 令状の場合: 管轄裁判所・裁判官署名・発行日・有効期限・対象範囲を確認する。
   - 不審な場合（非公式要求・口頭要求）: 正式書面の提出を求め、口頭では応じない。

### Step 2: 証跡保全の即時指示（〜2 時間）

3. 法務担当の指示のもと、対象データの削除・上書きを防ぐ **リーガルホールド** を設定する。

   ```bash
   # MinIO Object Lock でリーガルホールドを設定
   mc legalhold set minio/k1s0-logs/<target-path>/ ON
   mc legalhold set minio/k1s0-audit/<target-path>/ ON
   # 現在の保持設定を確認
   mc legalhold info minio/k1s0-logs/<target-path>/
   ```

4. 対象範囲に含まれるデータを定期削除ジョブから除外する。

   ```bash
   # Loki retention ルールから対象期間を一時除外
   kubectl edit configmap loki-retention -n monitoring
   # 対象期間のログを別バケットにコピー
   mc cp --recursive minio/k1s0-logs/tier1/<date-range>/ \
     minio/k1s0-legal-hold/incident-<case-id>/
   ```

### Step 3: 対象データの抽出（法務指示後のみ実施）

5. 法務担当の **書面による承認** を得てから以下を実施する。

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

6. 提供データを暗号化し、チェーン・オブ・カストディ記録を作成する。

   ```bash
   gpg --encrypt --recipient legal@k1s0.io \
     /tmp/legal-export-<case-id>.jsonl
   ```

### Step 4: 提供・非提供の判断ログ

7. 以下のテンプレートで判断記録を作成する（`ops/legal-records/` に保管）。

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

## 3. 復旧 (Recovery)

1. 法的手続の終了後、リーガルホールドを解除する（法務承認必須）。

   ```bash
   mc legalhold clear minio/k1s0-legal-hold/incident-<case-id>/
   ```

2. ケース ID に紐づく全対応記録を `ops/legal-records/<case-id>/` に集約する。
3. 外部提供したデータの内容と提供先を記録し、7 年間保管する。

## 4. 原因調査 (Root Cause Analysis)

- 本 Runbook においては「原因」ではなく **対応品質のレビュー** を実施する。
- 令状の有効性確認に時間を要した場合は確認プロセスを改善する。
- データ抽出に予想外の時間がかかった場合は監査ログ体系を見直す。
- 法務担当の連絡が取れなかった場合は代理対応フローを整備する。

## 5. 事後処理 (Post-incident)

- 法務担当とのポストレビュー（1 週間以内）
- ケース記録の 7 年保管設定
- 外部法律顧問との定期連絡（四半期）
- データ保持ポリシーの見直し（リーガルホールドがかかりやすいデータ種別の保持期間調整）

## 関連

- 関連設計書: docs/03_要件定義/30_非機能要件/E_セキュリティ.md (NFR-E-SIR-003)
- 関連設計書: docs/03_要件定義/30_非機能要件/G_データ保護とプライバシー.md (NFR-G-CLS-001)
- 関連 ADR: ADR-SEC-002 (OpenBao), ADR-SEC-003 (SPIRE)
- 関連 Runbook: escalation-contacts.md, pii-regulatory-disclosure.md
