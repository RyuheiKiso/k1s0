---
id: plan.data.scenario_tenant_onboarding
axis: data
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.data.data_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [B, C, E]
  proof_classes: []
---

# テナント onboarding（RLS / DEK / preservation_class 割当）

## 一文方針

data 担当者が新規テナント onboarding 時に PostgreSQL の RLS 設定 / DEK 生成 / preservation_class 割当 / 初回 restore_drill を実施し、テナントのデータ物理隔離と保全が開始前から保証された状態を確立する。

> 朝 10 時、本社 IT 室の data 担当者（シニア級）が Mattermost `#data-ops` で tier2 担当者からの「`mfg-yamada-kk` テナント onboarding 開始依頼」メッセージを確認する。手元には CloudNativePG dashboard と OpenBao 管理画面、`tenant_onboarding.lock.yaml`、Mattermost 越しに infra 担当者・tier2 担当者・security 担当者・dual reviewer がいる。

## Trigger（発火条件）

新規テナント契約完了後、tier2 担当者から「テナント onboarding を開始してほしい」と連絡が来た時。

## 想定頻度 / 典型きっかけ

想定頻度: 月次〜四半期（新規テナント onboarding 時）。典型きっかけ: 「`mfg-yamada-kk` テナント（中堅製造業、工場 3 拠点）の契約が完了し、製造業 pack の本番環境への onboarding を実施することになった」

## 主役 / 関与者

- 主役: data 担当者（シニア級）
- 関与: infra 担当者（Kubernetes namespace / network policy 設定）
- 関与: tier2 担当者（tenant_id の tier2 への登録）
- 関与: security 担当者（KEK 確認）
- 承認: dual reviewer（data 担当者 2 名、変更 PR の author 不可）

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（data）| シニア | 本社 IT 室 | CloudNativePG dashboard / Mattermost `#data-ops` | preservation_class 決定・RLS 設定・DEK 生成・restore_drill 実施 |
| 関与（infra）| シニア | 本社 IT 室 / リモート | Argo CD / Kyverno | Kubernetes namespace / network policy 設定 |
| 関与（tier2）| ミドル〜シニア | 本社 / リモート | Backstage TechDocs | tenant_id の tier2 登録・cross-tenant leak test 実行 |
| 関与（security）| シニア | 本社 / リモート | OpenBao 管理画面 | KEK 確認 |
| 承認（dual reviewer）| シニア | 本社 / リモート | Mattermost `#data-ops` | 変更 PR sign-off（data 担当者 2 名、author 不可） |

## 前提

- infra 担当者が新テナント用の Kubernetes namespace と network policy を設定済み
- CloudNativePG Cluster が `v1_zone_replicated` 以上の preservation_class で運用中
- OpenBao の KeySpace がテナント別に分離可能な状態

## 流れ

1. テナントの preservation_class を業務要件（SLA / データ量 / 規制）から決定し、`preservation_class.lock.yaml` に記録する
   - デフォルト: 製造業 pack 新規テナントは `v1_zone_replicated`（RPO 60 秒）
   - 規制対象（医薬品 GMP / 食品 HACCP）: `v1_cross_region_replicated`（RPO 5 分 + 監査証跡 7 年保持）
2. PostgreSQL の RLS（Row Level Security）FORCE を新テナント用に設定する
   - `CREATE POLICY tenant_isolation ON <table> USING (tenant_id = current_setting('app.current_tenant')::uuid)` を全テーブルに適用
   - `ALTER TABLE <table> ENABLE ROW LEVEL SECURITY` + `ALTER TABLE <table> FORCE ROW LEVEL SECURITY` を実施
3. テナント専用 DEK（Data Encryption Key）を OpenBao で生成し、KEK で wrap して保管する
   - `bao kv put kv/dek/<tenant_id> key=<generated_dek>`
   - DEK は tier1 の鍵管理適合仕様に従い AES-256-GCM を使用
4. Outbox / audit table のテナント分離設定を確認する（audit hash chain がテナント別に独立していることを確認）
5. 初回 restore_drill を staging 隔離環境で実施する（`v1_zone_replicated` の drill cadence に従い drill 要件を満たすことを確認）
6. cross-tenant data leak test を Testcontainers で実施する（`mfg-yamada-kk` のデータが他テナントから見えないことを確認）
7. `tenant_onboarding.lock.yaml` に onboarding 完了記録を追加し、dual reviewer sign-off を取得する
8. tier2 担当者に onboarding 完了を通知し、業務マスタ CSV bulk import シナリオへ連携する

## 業界 9 業務との紐付け

全 9 業務に共通基盤として影響（data は全業務の PostgreSQL / Kafka / ClickHouse の永続化基盤を担うため）。特に影響度が高い 2 業務:

- **受注管理（新テナント）**: onboarding 完了後に新テナントが最初に開始する業務が受注処理であるケースが多く、RLS FORCE と DEK 設定が正しくなければ受注データの物理隔離が機能しない。
- **品質検査（新テナント）**: 規制対象（医薬品 GMP / 食品 HACCP）テナントでは onboarding 時に `v1_cross_region_replicated` 以上の preservation_class と 7 年監査証跡保持が必要であり、初回 restore_drill が特に重要となる。

## 関連適合仕様 / 関連 OSS

- データ保全適合仕様: [../../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md](../../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md)
- PII 専用クラスタ: [../../../04_詳細設計/03_クロスカッティング適合仕様/08_PII_dedicated_cluster.md](../../../04_詳細設計/03_クロスカッティング適合仕様/08_PII_dedicated_cluster.md)
- 鍵管理適合仕様: [../../../04_詳細設計/01_適合仕様/05_鍵管理適合仕様.md](../../../04_詳細設計/01_適合仕様/05_鍵管理適合仕様.md)
- data 強制機構: [../../../04_詳細設計/02_強制機構/05_data強制機構.md](../../../04_詳細設計/02_強制機構/05_data強制機構.md)
- 関連 OSS: CloudNativePG（PostgreSQL RLS）/ OpenBao（DEK 管理）/ Testcontainers（cross-tenant leak test）

## 期待結果 / 観測指標

- RLS FORCE 設定が全テーブルに適用されていることを確認できる
- DEK 生成完了 + OpenBao への保管を確認できる
- `preservation_class.lock.yaml` が更新済み
- 初回 restore_drill が green であることを確認できる
- cross-tenant data leak test が全 case green であることを確認できる
- `tenant_onboarding.lock.yaml` が更新済み
- dual reviewer 2 名の sign-off が記録済み

## 失敗時の挙動 / escalation

- **cross-tenant data leak が検出された**: onboarding を即時中断。security 担当者に Mattermost `#security-incident` で即時通報（**SLA: 15 分以内**）。Backstage runbook `tenant-isolation-breach` を起動。本番テナントデータを一切触らせない状態を維持。**postmortem 期限: 2 営業日以内**。
- **初回 restore_drill fail**: onboarding を保留し restore_drill が green になるまで本番 onboarding を延期。infra 担当者と協力して CloudNativePG backup 設定を修正（**SLA: 3 営業日以内**）。
- **DEK 生成で OpenBao が応答しない**: infra 担当者に Mattermost `#infra-incident` で即時連絡（**SLA: 30 分以内**）。OpenBao の unseal 状態を確認し復旧後に onboarding を再開。

## 関連参照

- [data 担当者シナリオ index](./README.md) — data 担当者シナリオ全体の構成と 5 preservation_class 一覧
- [preservation_class 変更](./02_preservation_class変更.md) — onboarding 後にテナントの SLA が変更された場合のシナリオ
- [restore drill](./03_restore_drill.md) — 初回 restore_drill の詳細手順
- [PII 専用クラスタ運用](./07_PII専用クラスタ運用.md) — PII 対象データを含むテナントの追加設定シナリオ
- [業務マスタ CSV 一括 import（tier2-09）](../02_tier2担当者シナリオ/09_業務マスタCSV一括import.md) — 本シナリオ完了後に tier2 担当者が実施する次の工程
