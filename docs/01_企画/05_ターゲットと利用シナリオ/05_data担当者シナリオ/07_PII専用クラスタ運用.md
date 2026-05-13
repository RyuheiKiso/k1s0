---
id: plan.data.scenario_pii_dedicated_cluster
axis: data
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.data.data_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [D, E]
  proof_classes: []
---

# PII 専用クラスタ運用

## 一文方針

新規 PII 種別を [5 PII class](../../../04_詳細設計/03_クロスカッティング適合仕様/08_PII_dedicated_cluster.md) に分類し、物理隔離クラスタへの配置・RLS FORCE・envelope 暗号化・audit hash chain 記録・restore_drill を完結させたうえで security 担当者レビューを含む dual reviewer sign-off まで到達する。

> 午前 10 時、本社 IT 室の data 担当者（シニア級）が Mattermost `#data-ops` で security 担当者からの「生体認証データ（指紋スキャン）保存要件の追加依頼」メッセージを確認する。手元には `pii_cluster.lock.yaml` と CloudNativePG dashboard、Mattermost 越しに security 担当者・tier2 担当者・tier3 担当者・dual reviewer がいる。

## ペルソナ要約

主役: data 担当者（シニア級）、目的: PII 専用クラスタを構成し一般データとの isolation を強制する

## 現状業務での痛み

- PII が一般テーブルと混在しており、isolation がないため PII へのアクセス制御が粗粒度になる
- PII データの範囲が不明確で、DSAR 対応時にどのデータが対象かの特定に時間がかかる
- PII テーブルへのアクセスログが分散しており、データアクセスの監査が困難

## k1s0 でこう変わる

- PII 専用クラスタへの isolation が pii_cluster.lock.yaml で強制管理され、混在が CI で物理拒否される
- PII データの範囲が lock.yaml で明確に定義され、DSAR 対応時の対象特定が即時に可能になる
- PII クラスタへのアクセスが audit hash chain に一元記録され、データアクセス監査が効率化される

## Trigger（発火条件）

新規 PII 種別の追加（5 PII class への分類）、または PII 専用クラスタの設定変更が必要になった時。

## 想定頻度 / 典型きっかけ

想定頻度: 不定期（新 PII 種別追加時）。典型きっかけ: 「生体認証データ（指紋スキャン）を保存する要件が生じ、biometric PII class として物理クラスタに隔離する必要が生じた」

## 主役 / 関与者

- 主役: data 担当者（シニア級）
- 関与: security 担当者（PII class 分類の協議・レビュー必須・DEK KeySpace 設計）
- 関与: tier2 担当者（RLS 設定確認 / 13 層強制機構 lint 確認）
- 関与: tier3 担当者（localStorage / sessionStorage / IndexedDB への平文保管禁止の確認）
- 関与: dual reviewer（sign-off。security 担当者レビューが必須条件）

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（data）| シニア | 本社 IT 室 | `pii_cluster.lock.yaml` / CloudNativePG dashboard | PII class 分類・RLS FORCE 適用・envelope 暗号化設定・restore_drill 実施 |
| 関与（security）| シニア | 本社 / リモート | OpenBao 管理画面 / Mattermost `#security-ops` | PII class 分類協議・DEK KeySpace 設計・レビュー必須 |
| 関与（tier2）| ミドル〜シニア | 本社 / リモート | Backstage TechDocs | RLS 設定確認・13 層強制機構 lint 確認 |
| 関与（tier3）| ミドル〜シニア | 本社 / リモート | Backstage TechDocs | localStorage / sessionStorage / IndexedDB 平文保管禁止確認 |
| 承認（dual reviewer）| シニア | 本社 / リモート | Mattermost `#data-ops` | sign-off（security 担当者レビュー必須条件） |

## 個人 KPI / 達成感

- PII 混在 0 件が CI で定量確認でき、isolation 達成の達成感を得られる
- DSAR 対応時間の短縮を数値で確認でき、compliance 対応効率の改善を実感できる

## 工数 / 関与人数 / コスト感

- 工数: 2〜3 日（クラスタ設計 4h + isolation 設定 4h + migration 4h + CI 統合 4h）
- 関与人数: 3〜4 名（data 担当者・security 担当者・compliance 担当者・dual reviewer）
- コスト感: 中〜高（初回設定）。設定後の継続コストは lock.yaml 更新のみになる

## 前提

- PII 専用クラスタ（物理隔離）が `pii_cluster.lock.yaml` で管理済み
- 5 PII class が security 軸と bind して定義済み（例: individual / biometric / financial / medical / location）
- CloudNativePG の RLS FORCE が PII table に適用済み
- 13 層強制機構 lint が CI に組み込み済み
- PII 専用 KeySpace（OpenBao）が一般 data の DEK KeySpace と分離済み

## 流れ

1. 新 PII 種別を 5 PII class（`individual` / `biometric` / `financial` / `medical` / `location`）に分類する。security 担当者と協議し、分類根拠を記録する
2. PII 専用クラスタへの配置を `pii_cluster.lock.yaml` に宣言する。物理的に一般クラスタと分離されていることを manifest で確認する
3. CloudNativePG の RLS FORCE を対象 PII table に適用し、cross-tenant アクセスを物理拒否する
4. PII data の application-layer envelope 暗号化を設定する。DEK を PII 専用 KeySpace（OpenBao）で管理し、一般データの DEK とは別 KeySpace に格納する
5. tier3 の localStorage / sessionStorage / IndexedDB（非暗号化）への PII 平文保管を 13 層強制機構 lint で禁止されていることを確認する
6. 監査確認: PII アクセス全件が audit hash chain に記録されていることを確認する
7. PII cluster の restore_drill を staging で実施する（preservation_class は `v1_cross_region_replicated` 以上が必須）
8. security 担当者レビューを必須条件として dual reviewer sign-off を得る。`pii_cluster.lock.yaml` を更新する

## Timeline

| T+ | actor | action | 通知例 |
|---|---|---|---|
| 0 | data 担当者 | PII クラスタ構成を設計し pii_cluster.lock.yaml に PII テーブル一覧を登録 | `PII クラスタ設計完了 / lock.yaml 登録` |
| 4h | data 担当者 | PII テーブルを専用クラスタに migration しアクセス isolation を設定 | `migration 完了 / isolation 設定` |
| 1d | data 担当者 | CI の PII 混在チェックと audit trail を確認して PR 提出 | `PII 混在 0 件 / audit trail 確認 / PR #NNN` |
| 1d+4h | dual reviewer + security 担当者 | isolation 設定と lock.yaml を確認し sign-off | `sign-off 完了` |

## 業界 9 業務との紐付け

全 9 業務に共通基盤として影響（data は全業務の PostgreSQL / Kafka / ClickHouse の永続化基盤を担うため）。特に影響度が高い 2 業務:

- **品質検査**: 検査員の個人情報（指紋スキャン・顔認証）が biometric PII class として物理隔離クラスタに格納され、RLS FORCE による cross-tenant アクセス防止が GMP 準拠の証跡となる。
- **受注管理**: 担当者の個人情報（氏名・連絡先）が individual PII class として管理され、受注処理での PII アクセスが audit hash chain に全件記録される。

## 関連適合仕様 / 関連 OSS

- データ保全適合仕様: [../../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md](../../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md)
- PII dedicated cluster: [../../../04_詳細設計/03_クロスカッティング適合仕様/08_PII_dedicated_cluster.md](../../../04_詳細設計/03_クロスカッティング適合仕様/08_PII_dedicated_cluster.md)
- 関連 OSS: CloudNativePG（RLS FORCE）/ OpenBao（PII 専用 KeySpace）/ Rook+Ceph

## 期待結果 / 観測指標

- `pii_cluster.lock.yaml` に新 PII 種別が記録済み
- RLS FORCE が対象 PII table に適用され、cross-tenant アクセスが拒否されることを確認できる
- 13 層強制機構 lint が CI で green
- audit hash chain に PII アクセスが記録されていることを確認できる
- staging での restore_drill が green
- security 担当者レビューを含む dual reviewer 2 名の sign-off が記録済み

## 失敗時の挙動 / escalation

- **RLS FORCE 漏れ**: PII table への cross-tenant アクセスが可能な状態は security incident として扱い、security 担当者に即時 escalate する
- **audit hash chain への記録欠落**: compliance incident として扱い、security / ops 担当者に即時 escalate する。欠落原因の特定まで当該 PII 種別の運用を停止する
- **restore_drill fail**: security 担当者と協議し、preservation_class を下げることなく原因を修正してから再 drill を実施する。drill が green になるまで新規 PII 種別の本番投入を保留する
- **PII data が一般クラスタに混入した疑い**: security 担当者 + ops 担当者に Mattermost `#security-incident` で即時通報（**SLA: 15 分以内**）。Backstage runbook `pii-isolation-breach` を起動。compliance incident として扱い **postmortem 期限: 2 営業日以内**。
- **PII cluster restore_drill fail**: 1.0.0 ship blocker 認定。Backstage ticket `pii-drill-fail-<date>` を起票（**SLA: 5 営業日以内**に改善計画提出）。

## 失敗パターン (anti-pattern)

- PII を一般テーブルに混在保存: isolation check CI が PII カラムを検知し merge 阻止する
- アクセスログなしの PII クラスタ: audit trail なしのアクセスは compliance check が fail する

## 関連参照

- [data 設計方針](../../../03_概要設計/06_data設計方針/README.md) — PII 専用クラスタ物理隔離の設計思想と 5 preservation_class の全体方針
- [restore drill シナリオ](03_restore_drill.md) — PII cluster の restore_drill 実施手順と AND-gate の維持方法
- [KEK / DEK rotation シナリオ](05_暗号化変更_KEK_DEK_rotation.md) — PII 専用 KeySpace の DEK rotation 手順
- [シナリオ index](README.md) — data 担当者シナリオ全体の構成と preservation_class 一覧
