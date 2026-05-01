# SEV1 エスカレーション手順

本ファイルは SEV1 インシデント確定後 30 分以内に実施するエスカレーション手順を定める。
[`docs/04_概要設計/55_運用ライフサイクル方式設計/01_サポート階層方式.md`](../../docs/04_概要設計/55_運用ライフサイクル方式設計/01_サポート階層方式.md) の DS-OPS-SUP-* および
[`docs/04_概要設計/55_運用ライフサイクル方式設計/02_インシデント対応方式.md`](../../docs/04_概要設計/55_運用ライフサイクル方式設計/02_インシデント対応方式.md) の段階対応に対応する。
個別の連絡先は本ファイルではなく [`contacts.md`](contacts.md) に分離する（Kyverno policy で contacts.md は読取制限可能）。

> 起動条件: [`runbooks/incidents/RB-INC-001-severity-decision-tree.md`](../runbooks/incidents/RB-INC-001-severity-decision-tree.md) で **SEV1** が確定した直後に必ず起動する。
> 自律的な起動トリガーは存在しない。判定フローを経ること。

## 30 分タイムライン

```
T+0min   SEV1 確定
T+5min   IC 指名 + Slack インシデントチャンネル作成
T+15min  技術エスカレーション完了（CTO / EM / Security SRE / Platform）
T+30min  セキュリティ・法務エスカレーション完了（PII・越境・法的開示時）
T+60min  外部連絡先への通知（必要な場合）
```

## Step 1: インシデント指揮者（IC）の指名（〜5 分）

当番 SRE が IC を引き受けるか、または上位者に委譲する。IC は以下を宣言する:

- Slack `#incident-<YYYYMMDD>-<slug>` チャンネルを作成
- チャンネルに本ファイルへのリンクと、起動した Runbook の RB-* ID、状況サマリを投稿
- IC は 15 分ごとに状況更新を投稿し続ける（次節のテンプレ）

## Step 2: 技術エスカレーション（〜15 分）

連絡先の実体は [`contacts.md`](contacts.md) を参照。手段と期限は以下:

| 役割 | 手段 | 期限 |
|------|------|------|
| CTO | 電話 → Slack DM | 即時 |
| Engineering Manager | Slack DM | 即時 |
| Security SRE 当番 | PagerDuty エスカレーション | 即時 |
| プラットフォーム担当 | PagerDuty | 即時 |

```bash
# PagerDuty CLI で当番を確認
pd oncall list --schedule-id <PLATFORM_SCHEDULE_ID>
pd oncall list --schedule-id <SECURITY_SCHEDULE_ID>
```

## Step 3: セキュリティ・法務エスカレーション（〜30 分）

PII 漏えい / テナント越境 / 法的開示が絡む場合は追加連絡が必要。

| 役割 | 手段 | 期限 |
|------|------|------|
| 法務担当 | メール + 電話 | SEV1 確定後 1h |
| 個人情報保護責任者（CPO） | 電話 | PII 漏えい疑いの場合即時 |
| 採用組織セキュリティ担当 | メール | テナント越境確認後 2h |

連絡先は [`contacts.md`](contacts.md) §セキュリティ・法務 を参照。

## Step 4: 外部連絡先（必要な場合）

| 機関 | 条件 |
|------|------|
| 個人情報保護委員会 | PII 漏えい確定後 72h 以内（速報）— [RB-COMP-002 参照](../runbooks/incidents/RB-COMP-002-pii-regulatory-disclosure.md) |
| JPCERT/CC | サイバー攻撃の疑いがある場合 |
| 警察（サイバー犯罪相談窓口） | 不正アクセス禁止法違反が疑われる場合 |

連絡先 URL は [`contacts.md`](contacts.md) §外部機関 を参照。

## Step 5: ステータス更新テンプレート

IC は 15 分ごとに Slack `#incident-<YYYYMMDD>-<slug>` に投稿する:

```
【SEV1 インシデント】YYYY-MM-DD HH:MM JST
概要: <1 行サマリ>
影響範囲: <テナント / ユーザー数 / サービス>
IC: <名前>
現状: 調査中 / 封じ込め中 / 復旧中
進行中: <現在実行中の Runbook ID と Phase>
次回更新: HH:MM（15 分後）
```

## クローズ条件

SEV1 クローズには以下 3 点が必要:

1. 根本原因の封じ込めが確認されている（Runbook の §6. 検証手順 完了）
2. CTO 承認
3. ポストモーテム起票（24h 以内）が予定されている

## 振り返り

- 連絡履歴（メール / Slack / 電話ログ）をインシデント記録に添付する。
- 外部報告が発生した場合は報告内容と受領日時を記録する。
- エスカレーション対応の遅延原因（連絡つながらなかった等）を特定し、次四半期の `contacts.md` 棚卸しに反映する。
- ポストモーテムに「連絡対応の所要時間」を含める（30 分目標との乖離を記録）。

## 関連

- 関連 NFR: [NFR-E-SIR-001 / NFR-E-SIR-002](../../docs/03_要件定義/30_非機能要件/E_セキュリティ.md)、[NFR-C-OPS-001](../../docs/03_要件定義/30_非機能要件/C_運用.md)
- 関連設計書: [`docs/04_概要設計/55_運用ライフサイクル方式設計/01_サポート階層方式.md`](../../docs/04_概要設計/55_運用ライフサイクル方式設計/01_サポート階層方式.md)、[`docs/04_概要設計/55_運用ライフサイクル方式設計/02_インシデント対応方式.md`](../../docs/04_概要設計/55_運用ライフサイクル方式設計/02_インシデント対応方式.md)
- 関連 ADR: [ADR-SEC-001（Keycloak）](../../docs/02_構想設計/adr/ADR-SEC-001-keycloak.md)
- 関連 Runbook:
  - [`runbooks/incidents/RB-INC-001-severity-decision-tree.md`](../runbooks/incidents/RB-INC-001-severity-decision-tree.md)
  - [`runbooks/incidents/RB-COMP-002-pii-regulatory-disclosure.md`](../runbooks/incidents/RB-COMP-002-pii-regulatory-disclosure.md)
  - [`runbooks/incidents/RB-COMP-001-legal-disclosure.md`](../runbooks/incidents/RB-COMP-001-legal-disclosure.md)
- 連絡先実体: [`contacts.md`](contacts.md)
