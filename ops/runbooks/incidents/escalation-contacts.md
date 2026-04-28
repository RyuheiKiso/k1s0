# エスカレーション連絡先 Runbook

> **severity**: SEV1
> **owner**: tier1-platform-team
> **estimated_mttr**: 30min（初回連絡完了まで）
> **last_updated**: 2026-04-28

## 1. 検出 (Detection)

本 Runbook は `severity-decision-tree.md` で **SEV1** が確定した直後に起動する。
自律的な起動トリガーは存在しない。必ず判定フローを経ること。

## 2. 初動 (Immediate Action)

SEV1 確定後 **30 分以内** に以下の順序で連絡を完了する。

### Step 1: インシデント指揮者（IC）の指名（〜5 分）

当番 SRE が IC を引き受けるか、または上位者に委譲する。IC は以下を宣言する。

- Slack `#incident-<YYYYMMDD>-<slug>` チャンネルを作成
- チャンネルに本 Runbook リンクと状況サマリを投稿

### Step 2: 技術エスカレーション（〜15 分）

| 役割 | 連絡先 | 手段 | 期限 |
|------|--------|------|------|
| CTO | `cto@k1s0.io`（要置換） | 電話 → Slack | 即時 |
| Engineering Manager | `em-oncall@k1s0.io`（要置換） | Slack DM | 即時 |
| Security SRE 当番 | `security-oncall@k1s0.io`（要置換） | PagerDuty エスカレーション | 即時 |
| プラットフォーム担当 | `platform-oncall@k1s0.io`（要置換） | PagerDuty | 即時 |

```bash
# PagerDuty CLI でオンコールを確認
pd oncall list --schedule-id <PLATFORM_SCHEDULE_ID>
pd oncall list --schedule-id <SECURITY_SCHEDULE_ID>
```

### Step 3: セキュリティ・法務エスカレーション（〜30 分）

PII 漏えい / テナント越境 / 法的開示が絡む場合は追加連絡が必要。

| 役割 | 連絡先 | 手段 | 期限 |
|------|--------|------|------|
| 法務担当 | `legal@k1s0.io` | メール + 電話 | SEV1 確定後 1h |
| 個人情報保護責任者（CPO） | `privacy@k1s0.io`（要置換） | 電話 | PII 漏えい疑いの場合即時 |
| 採用組織セキュリティ担当 | `customer-security@<adopter>.example`（要置換） | メール | テナント越境確認後 2h |

### Step 4: 外部連絡先（必要な場合）

| 機関 | 連絡先 | 条件 |
|------|--------|------|
| 個人情報保護委員会 | `https://www.ppc.go.jp/` 報告システム | PII 漏えい確定後 72h 以内（速報） |
| JPCERT/CC | `https://www.jpcert.or.jp/` | サイバー攻撃の疑いがある場合 |
| 警察（サイバー犯罪相談窓口） | 都道府県警察本部（要置換） | 不正アクセス禁止法違反が疑われる場合 |

### Step 5: ステータス更新テンプレート

```
【SEV1 インシデント開始】YYYY-MM-DD HH:MM JST
概要: <1 行サマリ>
影響範囲: <テナント / ユーザー数 / サービス>
IC: <名前>
現状: 調査中 / 封じ込め中 / 復旧中
次回更新: HH:MM（15 分後）
```

## 3. 復旧 (Recovery)

1. IC は 15 分ごとに Slack チャンネルへ状況更新を投稿する。
2. 技術対応チームと連絡チームを分離し、IC が情報集約を一元化する。
3. 役員・法務への報告は IC または EM が実施し、技術担当は対応に集中させる。
4. SEV1 クローズ条件: 根本原因封じ込め確認 + CTO 承認。

## 4. 原因調査 (Root Cause Analysis)

- 連絡履歴（メール / Slack / 電話ログ）をインシデント記録に添付する。
- 外部報告が発生した場合は報告内容を記録する。
- エスカレーション対応の遅延原因（連絡つながらなかった等）を特定する。

## 5. 事後処理 (Post-incident)

- ポストモーテム（24h 以内）に連絡対応の振り返りを含める
- オンコールローテーション・連絡先の最新化を四半期ごとに実施
- PII 漏えいが確定した場合: 個人情報保護委員会への 30 日確報を提出

## 関連

- 関連設計書: docs/03_要件定義/30_非機能要件/E_セキュリティ.md (NFR-E-SIR-001, NFR-E-SIR-002)
- 関連 ADR: ADR-SEC-001 (Keycloak)
- 関連 Runbook: severity-decision-tree.md, pii-regulatory-disclosure.md, legal-disclosure.md
