# oncall/rotation — PagerDuty Schedule

本ディレクトリは PagerDuty の Schedule（オンコール当番表）を Terraform 管理用 YAML として保管する。
[`docs/04_概要設計/55_運用ライフサイクル方式設計/01_サポート階層方式.md`](../../../docs/04_概要設計/55_運用ライフサイクル方式設計/01_サポート階層方式.md) の DS-OPS-SUP-* に対応する。

## 配置（予定）

```text
rotation/
├── README.md                # 本ファイル
├── 2026-Q2.yaml             # 四半期ごとの Schedule export
├── 2026-Q3.yaml
└── escalation-policy.yaml   # SEV1 / SEV2 のエスカレーションポリシー
```

## 採用契約までの暫定運用

リリース時点では起案者がフルタイムでオンコールを兼任する単一開発者体制であり、PagerDuty Schedule の実体ファイルは
採用組織の SRE 体制が確定し次第（採用契約締結時）に Terraform で生成する。それまでの暫定運用は以下:

- SEV1 / SEV2 アラートは Slack `#incident-sev1` / `#alert-tier1` に直接通知され、起案者がモバイルで応答する。
- 起案者が応答不可な場合のフォールバック先は契約上の協力者（書面合意済み）に SMS 経由で連絡。
- 連絡経路は [`../contacts.md`](../contacts.md) に集約済み。

## Schedule 整備時の YAML スキーマ（雛形）

```yaml
# 2026-Q2.yaml（採用後に整備）
schedule:
  name: k1s0-platform-oncall
  time_zone: Asia/Tokyo
  layers:
    - name: weekday-primary
      start: "2026-04-01T09:00:00+09:00"
      rotation_turn_length_seconds: 86400  # 24h rotation
      users:
        - <user-id-1>
        - <user-id-2>
    - name: weekend-secondary
      start: "2026-04-06T00:00:00+09:00"
      rotation_turn_length_seconds: 172800  # 48h rotation
      users:
        - <user-id-3>
overrides: []
```

`escalation-policy.yaml` は SEV1 = primary 5 分応答 → secondary 10 分 → CTO 直通、を Terraform 化する。

## 関連

- 関連 Runbook: [`../escalation.md`](../escalation.md)
- 連絡先: [`../contacts.md`](../contacts.md)
- Terraform 管理基盤: `infra/iac/terraform/pagerduty/`（採用後の運用拡大時で整備）
