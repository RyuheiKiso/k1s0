---
id: plan.tier2.scenario_tenant_override_extension
axis: tier2
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.tier2.tier2_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# テナント別 override 拡張点

## 一文方針

特定テナントが業務ルール（マスタ / 決定表 / Workflow）のカスタマイズを要求した際、4 抽象化レベルの境界を守りながら tenant_id 強制注入付き拡張点 contract を定義し、上位レベルへの漏洩を dual reviewer と CI の二重防衛で阻止する。

## Trigger（発火条件）

特定テナントが業務ルール（マスタ / 決定表 / Workflow）のカスタマイズを要求した時。

## 想定頻度 / 典型きっかけ

- 想定頻度: 月次（新テナントオンボーディング時）
- 典型きっかけ: 「A 工場テナントが発注承認のフローを 2 段階から 3 段階に変更する要求を出した」
- 典型きっかけ 2: 「`mfg-acme-jp` テナントが品質検査の閾値を標準 ±3% から ±2% に厳格化した」

## 主役 / 関与者

- 主役: tier2 担当者（中堅級）
- 承認: dual reviewer（tier2 担当者 + security 担当者、変更 PR の author 不可）

## 前提

- 業務資産所有権（[06_業務資産所有権.md](../../../03_概要設計/03_tier2設計方針/06_業務資産所有権.md)）で規定された「既定実装と override 拡張点」が確立済みであること
- 4 言語 DI 基盤（Rust trait / C# interface / Go interface / TypeScript interface）が整備済みであること
- Keycloak により tenant_id が JWT claim として発行されていること
- CloudNativePG の [RLS FORCE](../../../04_詳細設計/01_適合仕様/10_テナント分離適合仕様.md)（Row Level Security の強制設定）が適用済みであること

## 流れ

1. カスタマイズ要求を 4 抽象化レベル（業界横断 / 業界共通 / 業界固有 / テナント固有）に分類する
2. テナント固有 Lv のみ override 拡張点経由でカスタマイズ可能か確認する（上位 Lv への漏洩は禁止）
3. 4 言語 DI / 注入経路（Rust trait / C# interface / Go interface / TypeScript interface）で拡張点 contract を定義する
   - Rust: trait bound（`OverrideExt: TenantOverride`）で定義
   - C#: interface（`ITenantOverride`）と DI container（`services.AddScoped<ITenantOverride, CustomImpl>()`）で注入
   - Go: interface + wire 生成
   - TypeScript: injection token + decorator pattern
4. テナント識別子（tenant_id）を拡張点インタフェースで強制注入し、成りすましを不可能化する
5. contract test をテナント別拡張実装に対して実行し、invariants 抵触を検出する
6. override 実装の PR に dual reviewer（tier2 担当 + security 担当）の sign-off を得てから merge する

## 関連適合仕様 / 関連 OSS

- [テナント分離適合仕様](../../../04_詳細設計/01_適合仕様/10_テナント分離適合仕様.md)
- [tier2 強制機構](../../../04_詳細設計/02_強制機構/02_tier2強制機構.md)
- Keycloak（tenant_id = JWT claim として発行）
- CloudNativePG（RLS FORCE によるデータアクセス制御）

## 期待結果 / 観測指標

- テナント固有カスタマイズが上位 Lv（業界固有以上）に影響しないこと
- 全 4 言語で拡張点 contract が同一の invariants を満たすこと
- contract test が全テナント拡張実装で green になること
- tenant_id 注入経路が lint により確認されること

## 失敗時の挙動 / escalation

- **tenant_id 注入経路欠落 lint fail**: lint fail → merge 阻止。escalate 先: Mattermost `#tier2-ci-alert`（SLA: 24h 以内に是正 PR 提出）。runbook: Backstage `tenant-override-incident-procedure`
- **contract test invariants 抵触 fail**: contract test fail → 実装修正を要求。escalate 先: Mattermost `#tier2-ci-alert`（SLA: 24h 以内に是正 PR 提出）。runbook: Backstage `tenant-override-incident-procedure`
- **カスタマイズ上位 Lv 漏洩 fail**: 業界中立性 lint fail → merge 阻止。escalate 先: Mattermost `#security-incident`（SLA: security 担当への連絡は 1h 以内）。runbook: Backstage `tenant-override-incident-procedure`

## 関連参照

- [tier2 担当者シナリオ index](README.md)
- [tier2 設計方針](../../../03_概要設計/03_tier2設計方針/README.md)
- [tenant_id 強制注入追加](06_tenant_id強制注入追加.md)
