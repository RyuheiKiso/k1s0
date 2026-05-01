# oncall — オンコール体制と連絡先

本ディレクトリは k1s0 のオンコール体制（PagerDuty Schedule 連動）、SEV1 エスカレーション手順、SOPS 鍵運用を集約する。

[`docs/05_実装/00_ディレクトリ設計/60_operationレイアウト/05_ops配置_Runbook_Chaos_DR.md`](../../docs/05_実装/00_ディレクトリ設計/60_operationレイアウト/05_ops配置_Runbook_Chaos_DR.md) と
[`docs/04_概要設計/55_運用ライフサイクル方式設計/01_サポート階層方式.md`](../../docs/04_概要設計/55_運用ライフサイクル方式設計/01_サポート階層方式.md) に対応する。

## 配置

```text
oncall/
├── README.md           # 本ファイル
├── escalation.md       # SEV1 エスカレーション手順（プロセス）
├── contacts.md         # 連絡先一覧（データ）
├── rotation/           # PagerDuty Schedule export（Terraform 管理）
│   └── README.md
└── sops-key/           # SOPS AGE 鍵の運用手順（OpenBao 経由）
    └── README.md
```

## ファイル責務

- [`escalation.md`](escalation.md) — SEV1 確定後 30 分タイムラインのプロセス手順。連絡先は contacts.md 参照。
- [`contacts.md`](contacts.md) — 技術 / セキュリティ・法務 / 外部機関 / インフラベンダの連絡先一覧。Kyverno で読取制限。
- [`rotation/`](rotation/) — PagerDuty Schedule の Terraform 管理ファイル群（採用組織のオンコール体制が決まり次第整備）。
- [`sops-key/`](sops-key/) — SOPS AGE 鍵の生成・分散・ローテ手順。OpenBao 経由で配布。

## 起動条件

オンコール体制への接続点:

1. **PagerDuty 経由**: `KafkaBrokerDown` / `PostgresPrimaryDown` 等の Alertmanager rule が PagerDuty 発火 → 当番 SRE が対応 Runbook を起動。
2. **SEV1 確定経由**: [`runbooks/incidents/severity-decision-tree.md`](../runbooks/incidents/severity-decision-tree.md) で SEV1 確定 → [`escalation.md`](escalation.md) を実行。
3. **外部報告経由**: 顧客報告 / セキュリティリサーチャー通報 → [`escalation.md`](escalation.md) で IC 指名。

## 関連

- 関連設計書: [`docs/04_概要設計/55_運用ライフサイクル方式設計/01_サポート階層方式.md`](../../docs/04_概要設計/55_運用ライフサイクル方式設計/01_サポート階層方式.md), [`02_インシデント対応方式.md`](../../docs/04_概要設計/55_運用ライフサイクル方式設計/02_インシデント対応方式.md)
- 関連 NFR: [NFR-C-OPS-001](../../docs/03_要件定義/30_非機能要件/C_運用.md), [NFR-E-SIR-001 / NFR-E-SIR-002](../../docs/03_要件定義/30_非機能要件/E_セキュリティ.md)
- 関連 ADR: [ADR-OBS-003 Incident Taxonomy](../../docs/02_構想設計/adr/ADR-OBS-003-incident-taxonomy.md)
