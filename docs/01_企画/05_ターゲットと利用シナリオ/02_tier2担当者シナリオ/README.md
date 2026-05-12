---
id: plan.tier2.scenario_index
axis: tier2
phase: plan
kind: index
status: draft
depends_on:
  - arch.tier2.tier2_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# tier2 担当者シナリオ index

## 一文方針

tier2 担当者（中堅級）が日常的に踏む 10 シナリオを 1 ファイル 1 シナリオで列挙する。業務資産（Domain Event / Workflow / 決定表 / 業務マスタ / DB schema）の所有と、tier3 が tier1 を迂回しない 8 層強制機構の維持が主責務。

## 担当者プロフィール

- 級: 中堅
- 想定人数: 5-10 名（業界 pack ごとに増減）
- 必須スキル: Rust / C# / Go / TypeScript（4 言語）+ ドメイン業務 / 業界 pack

tier2 担当者は以下の責務を横断的に担う:

- 業務資産（Domain Event / Workflow / 決定表 / 業務マスタ / DB schema）の所有・設計・維持
- 業界横断 / 業界共通 / 業界固有 / テナント固有の 4 抽象化レベルに沿った業務語彙の分類・管理
- atomic 三表書込（state / outbox / audit）を全 aggregate に適用する責務
- 8 層強制機構の維持（tier3 が tier1 を迂回しないための境界保護）
- 業界中立性の番人（業界固有概念の業界横断層への混入を CI で阻止）
- テナント識別子（tenant_id）強制注入経路の全 API への徹底

## dual reviewer 規約

- tier2 シナリオにおける dual reviewer = **tier2 担当者 2 名（変更 PR の author 不可、同一人物不可）**
- 重要性の高い変更（security/compliance 関連）は + security 担当者 1 名の 3 名確認を推奨
- 詳細: [層別エンジニア要件](../../../02_要件定義/05_開発体制要件/01_層別エンジニア要件.md)

## 8 層強制機構（tier3 が tier2 を迂回できない仕組み）

1. tier3 リポジトリ側 import lint（tier1 Library の直接 import を禁止）
2. tier2 公開 API 表面 lint（業務管理 API を業務 UI に expose しない）
3. tier2 内部依存方向 lint（業界固有概念 → 業界横断概念への逆方向依存禁止）
4. 第二業界 stub conformance test（業界横断層 API が stub から消費可能であることを CI で保証）
5. リポジトリ抽象強制注入（tenant_id 述語を全 API に強制 — 成りすまし不可）
6. 内部レジストリ（tier2 内で使える OSS を allowlist で管理）
7. Library Service 等価性 test（Library 版と Service 版の API が等価であることを Testcontainers で確認）
8. 監査スキーマ整合（audit table schema と Domain Event schema の自動整合チェック）

詳細: [tier2 強制機構](../../../04_詳細設計/02_強制機構/02_tier2強制機構.md)

## シナリオ一覧

| # | シナリオ名 | trigger | 想定頻度 | 主たる関連適合仕様 | 種別 |
|---|-----------|---------|---------|-----------------|------|
| 01 | [業界pack追加](01_業界pack追加.md) | 新業界の正式 pack 追加要求が来た時 | 年次〜バージョン毎 | テナント分離適合仕様 / tier2強制機構 | [計画] |
| 02 | [ドメイン分割_BoundedContext](02_ドメイン分割_BoundedContext.md) | ドメインが肥大化し単一 Bounded Context に複数業務語彙が混在した時 | 四半期〜年次 | スキーマ進化適合仕様 / tier2強制機構 | [計画] |
| 03 | [テナント別override拡張点](03_テナント別override拡張点.md) | 特定テナントが業務ルールのカスタマイズを要求した時 | 月次（テナント追加時） | テナント分離適合仕様 / tier2強制機構 | [計画] |
| 04 | [Domain_Event_Workflow_決定表追加](04_Domain_Event_Workflow_決定表追加.md) | 業務フロー変更により新規 Domain Event / Workflow / 決定表の追加が必要になった時 | 月次 | protoc_gen_go_fsm / スキーマ進化適合仕様 | [計画] |
| 05 | [業界中立性違反検出_是正](05_業界中立性違反検出_是正.md) | CI の業界中立性 lint が fail した時 | イベント駆動（CI fail 時） | tier2強制機構 | [緊急] |
| 06 | [tenant_id強制注入追加](06_tenant_id強制注入追加.md) | 新 API 追加時に tenant_id 強制注入の漏れが CI で検出された時 | イベント駆動（lint fail 時） | テナント分離適合仕様 / tier2強制機構 | [緊急] |
| 07 | [BusinessConflict_subtype追加](07_BusinessConflict_subtype追加.md) | 並行編集シナリオの新 conflict subtype を追加する必要が生じた時 | 四半期〜年次 | スキーマ進化適合仕様 | [計画] |
| 08 | [外部システム統合](08_外部システム統合.md) | 既存基幹システムとの新規連携が要求された時 | 年次〜不定期 | スキーマ進化適合仕様 / tier2強制機構 | [計画] |
| 09 | [業務マスタCSV一括import](09_業務マスタCSV一括import.md) | 新テナント追加または業界 pack 更新時に業務マスタ bulk import | 月次 | テナント分離適合仕様 | [計画] |
| 10 | [Scheduler_CronWorkflow追加](10_Scheduler_CronWorkflow追加.md) | 定期バッチ要件が tier2 に起票された時 | 月次〜四半期 | SLO 適合仕様 | [計画] |

## 新規参画者向けオンボーディング

中堅級エンジニアとして着任した際の推奨学習順序:

- **Day 1-3**: README 全体読了 → 06（tenant_id 強制注入）→ 05（業界中立性違反）を通読（最頻 lint fail への備え）
- **Day 4-7**: 03（テナント override）→ 04（Domain Event 追加）をペア担当者と演習
- **Week 2**: 01（業界 pack 追加）→ 02（ドメイン分割）の設計判断を先輩担当者と議論
- **Week 3-4**: 09（業務マスタ CSV import）→ 10（Scheduler 追加）を実タスクとして担当
- **Month 2 以降**: 07（BusinessConflict）→ 08（外部システム統合）は発生時に担当

## シナリオ間の依存関係

- **01（業界 pack）→ 03（テナント override）**: 新業界 pack 追加後、テナント固有 override 拡張点が必要になると 03 が発火。
- **04（Domain Event 追加）→ 10（Scheduler 追加）**: Domain Event を定期的に集計するバッチが必要になると 10 が発火。
- **10（Scheduler 追加）→ 05（業界中立性違反）**: バッチ処理が業界固有概念を業界横断層に持ち込むと 05 が発火。
- **09（CSV import）の先行条件**: data-10（tenant_onboarding）完了後に実施（data 担当者が RLS / DEK を設定済みであること）。

## 関連参照

- [tier2 設計方針](../../../03_概要設計/03_tier2設計方針/README.md)
- [ターゲットと利用シナリオ index](../README.md)
