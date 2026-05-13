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
  defense_in_depth_layers: [A, B, C, D]
  proof_classes: []
---

# テナント別 override 拡張点

## 一文方針

特定テナントが業務ルール（マスタ / 決定表 / Workflow）のカスタマイズを要求した際、4 抽象化レベルの境界を守りながら tenant_id 強制注入付き拡張点 contract を定義し、上位レベルへの漏洩を dual reviewer と CI の二重防衛で阻止する。

> 朝 10 時、本社 IT 室の tier2 担当者（中堅級）が Backstage Software Catalog で `mfg-acme-jp` テナントの override 要求チケットに気付く。手元には GitHub PR・Keycloak 管理コンソール、Mattermost 越しに security 担当者がいる。

## ペルソナ要約

主役: tier2 担当者（中堅級）、目的: テナント固有業務ルールのカスタマイズを 4 抽象化レベルの境界内に収め tenant_id 強制注入付き拡張点 contract で上位レベルへの漏洩を物理拒否する

## 現状業務での痛み

- テナント固有カスタマイズが上位レベル（業界固有以上）に漏洩していても CI がなく、後のテナント追加時に予期しない挙動が発生する
- 4 言語の DI 基盤が統一されておらず、言語ごとに異なる注入パターンが混在して contract の整合性が失われる
- tenant_id なしで呼べる API が残っていて成りすまし経路が生まれるが、手動スキャンでしか検出できない
- contract test が一部テナント拡張実装でのみ実行され、全テナントを網羅していない

## k1s0 でこう変わる

- 業界中立性 lint が上位レベルへの漏洩を merge 前に物理拒否し、カスタマイズの境界違反をゼロにする
- 4 言語 DI 基盤（Rust trait / C# interface / Go interface / TypeScript interface）で拡張点 contract を統一し、言語ごとの差異をなくす
- tenant_id 注入経路 lint が全 API に対して実行され、成りすまし可能な経路を CI で物理検出する
- contract test を全テナント拡張実装に対して実行し、invariants 抵触を事前に検出する

## Trigger（発火条件）

特定テナントが業務ルール（マスタ / 決定表 / Workflow）のカスタマイズを要求した時。

## 想定頻度 / 典型きっかけ

- 想定頻度: 月次（新テナントオンボーディング時）
- 典型きっかけ: 「A 工場テナントが発注承認のフローを 2 段階から 3 段階に変更する要求を出した」
- 典型きっかけ 2: 「`mfg-acme-jp` テナントが品質検査の閾値を標準 ±3% から ±2% に厳格化した」

## 主役 / 関与者

- 主役: tier2 担当者（中堅級）
- 承認: dual reviewer（tier2 担当者 + security 担当者、変更 PR の author 不可）

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（tier2）| 中堅 | 本社 IT 室 | Backstage Catalog / Mattermost `#tier2-ops` | 拡張点 contract 定義 / tenant_id 強制注入 / contract test 実行 |
| 承認（security 担当者）| シニア | 本社 / リモート | GitHub PR | 権限漏洩 review / sign-off |

## 個人 KPI / 達成感

- テナント固有カスタマイズが上位 Lv に影響しないこと（業界中立性 lint green）
- 4 言語 contract test が全テナント拡張実装で green
- tenant_id 注入経路 lint green（全 API）
- security 担当者 sign-off 取得

## 工数 / 関与人数 / コスト感

- 初回（新テナント override 要求）: 1〜2 日（要件精査・4 言語拡張点 contract 定義・contract test）、関与 3〜4 名（主役 + security 担当者 + dual reviewer）
- 平常（既存 override の変更）: 半日〜1 日、関与 3 名
- 失敗時（上位 Lv 漏洩・tenant_id 漏れ）: +半日〜1 日、関与 4 名（+ security incident 対応）

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

## Timeline

| T+ | actor | action | 通知例 |
|---|---|---|---|
| 0 | tier2 担当者 | カスタマイズ要求を 4 Lv に分類・上位 Lv 漏洩確認 | `mfg-acme-jp override 要求: テナント固有 Lv OK / 漏洩なし` |
| 1 日 | tier2 担当者 | 4 言語 DI 拡張点 contract 定義・tenant_id 強制注入 | `Rust trait / C# interface / Go interface / TS interface 定義完了` |
| 1.5 日 | tier2 担当者 | contract test 実行・invariants 確認 | `contract test: green / invariants 抵触なし` |
| 2 日 | security 担当者 | 権限漏洩 review・sign-off | `security sign-off 完了 / tenant_id 注入経路 lint: green` |

## 業界 9 業務との紐付け

- **品質検査結果**: テナント固有の検査閾値 override（例: ±2% 厳格化）が品質検査結果 aggregate に反映され、テナント間で閾値が混在しない。
- **FA 生産指示・設備操作**: テナント固有の承認フロー段数変更が生産指示 Workflow に override として適用される。
- **受注**: テナント固有の発注承認ルール override が受注エンティティの決定表に反映される。

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

## 失敗パターン (anti-pattern)

- **業界固有 Lv のカスタマイズを業界横断層に配置**: 製造業固有の検査閾値が医療業 pack にも影響するようになり、pack 間の独立性が失われる。テナント固有カスタマイズは必ずテナント固有 Lv 内に閉じることを 4 Lv 分類の先行確認で担保する。
- **4 言語のうち 1 言語のみ拡張点 contract を実装**: 他言語での同等性が担保されず、言語切り替え時に挙動差異が発生する。4 言語 DI 基盤での contract 定義を全言語同時 PR の必須条件とする。
- **security 担当者レビューなしで merge**: 権限漏洩の見落としが発生し、後から security incident 対応が必要になる。security 担当者 sign-off を merge の物理前提条件として CI gate に設定する。

## 関連参照

- [tier2 担当者シナリオ index](README.md)
- [tier2 設計方針](../../../03_概要設計/03_tier2設計方針/README.md)
- [tenant_id 強制注入追加](06_tenant_id強制注入追加.md)
