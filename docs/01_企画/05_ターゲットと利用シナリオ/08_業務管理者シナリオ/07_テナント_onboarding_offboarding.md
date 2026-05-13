---
id: plan.overview.scenario_business_admin_tenant_lifecycle
axis: overview
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.tier2.tier2_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [B, C, D]
  proof_classes: []
---

# テナント_onboarding_offboarding

## 一文方針

新事業部 / 工場の onboarding（RLS FORCE / preservation_class / region / 業界 pack バージョン設定）と、撤退時の offboarding（データ crypto-shred / テナント無効化）を Backstage プラグインから実施する。

> 春の組織改編期、業務管理者は本社から「新工場（東北工場）の k1s0 テナント追加を 4 月 1 日付で実施してほしい」という依頼を受ける。Backstage プラグインのテナント管理画面を開き、onboarding ウィザードを起動する。RLS FORCE の有効化・preservation_class の設定・リージョン（ap-northeast-1）・業界 pack バージョン（manufacturing-v1.2）を順に設定する。tier2 担当者と協働で設定内容を確認し、test テナントとして動作確認を行ってから本番適用する。offboarding 時は crypto-shred 実行を data 担当者 + security 担当者に委任する。

## ペルソナ要約

主役: 業務管理者（シニア級）、目的: 新テナントの安全な onboarding と撤退時の完全な offboarding を tier2 担当者と協働で実施する

## 現状業務での痛み

- 新事業部のシステム追加には IT 部門の数週間の準備が必要で、事業立ち上げのスピードを阻害する
- テナント分離設定（RLS / データ保存要件）のミスが他テナントのデータに影響するリスクがあり、設定確認に工数がかかる
- 撤退時のデータ削除手順が文書化されていないため、契約終了後もデータが残存し規制違反リスクが生じる
- 業界 pack のバージョン管理が手動で行われており、テナントごとに異なるバージョンが混在する

## k1s0 でこう変わる

- Backstage の onboarding ウィザードにより、RLS FORCE / preservation_class / region / 業界 pack を guided に設定でき、設定ミスが構造的に防止される
- test テナントとしての動作確認ステップが onboarding フローに組み込まれており、本番適用前の検証が強制される
- offboarding 時は crypto-shred が tier2 API 経由で実行され、データ残存リスクが排除される
- 全 onboarding / offboarding 操作が audit hash chain に記録されるため、テナント lifecycle の完全な証跡が確保される

## Trigger

事業部追加 / 組織再編 / 撤退が発生した時（本社からの組織変更通知または撤退決定通知をトリガーとする）

## 想定頻度 / 典型きっかけ / 頻度根拠

- 想定頻度: 四半期〜年次
- 典型きっかけ: 「新工場の立ち上げに伴うテナント追加」「組織再編で事業部が合併しテナントを統合する」「海外工場の撤退でテナントを無効化しデータを crypto-shred する」
- 頻度根拠: 製造業では四半期〜年次の組織変動でテナント追加 / 削除が発生する

## 主役 / 関与者

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|------|---|------------|----------------|----------------------|
| 業務管理者（主役） | シニア | 事務所 | Backstage プラグイン | onboarding ウィザード実行 / offboarding 依頼 |
| tier2 担当者 | シニア | 開発拠点 | GitHub PR list | RLS / テナント設定の技術的確認 |
| data 担当者 | シニア | 開発拠点 | Backstage Catalog | crypto-shred 実行（offboarding 最終確認） |
| security 担当者 | シニア | 開発拠点 | セキュリティダッシュボード | RLS policy / crypto-shred 承認 |

## 個人 KPI / 達成感

- onboarding 依頼受領から test テナント動作確認完了までの所要時間（目標: 1 営業日以内）
- 本番適用前の test テナント検証実施率 100%
- offboarding 完了（crypto-shred 実行確認）から audit ログ発行確認までの所要時間（目標: 1 時間以内）
- テナント設定ミスによる後続修正発生件数 / 年（目標: ゼロ）

## 工数 / 関与人数 / コスト感

- onboarding（標準設定）: 半日〜1 営業日（ウィザード + test 確認 + 本番適用）
- offboarding（crypto-shred 含む）: 1〜2 営業日（data 担当者 + security 担当者と協働）
- 失敗時（設定ミス / test 失敗）: +半日〜1 営業日
- 関与人数: 3〜5 名（業務管理者 + tier2 + data + security 担当者）

## 前提

- 本社から組織変更の承認文書（承認番号付き）が発行されている
- tier2 担当者 / data 担当者 / security 担当者が参加可能な状態である
- onboarding の場合: 新テナントの region / preservation_class / 業界 pack バージョンが要件書に明記されている
- offboarding の場合: データ保存期間が終了しており、crypto-shred の実行が法務部門に承認されている

## 流れ（onboarding）

1. 本社から組織変更承認文書（テナント追加要件書）を受領する
2. Backstage プラグインのテナント管理画面を開き、「新規テナント追加」を選択する
3. onboarding ウィザードを起動し、テナント名 / region / preservation_class / 業界 pack バージョンを順に設定する
4. RLS FORCE の有効化設定を確認する（tier2 担当者と協働で技術的整合性を確認）
5. 「test テナント」としてデプロイし、業務管理者と tier2 担当者で動作確認を行う
6. test テナントの動作確認が完了したら「本番適用」ボタンを押下する
7. audit ログ発行を確認し、承認文書番号を記録する
8. 新テナントのデフォルトマスタ設定（シナリオ 01）と決定表設定（シナリオ 06）を実施する

## 流れ（offboarding）

1. 本社から撤退承認文書（crypto-shred 実行承認付き）を受領する
2. Backstage プラグインのテナント管理画面でテナントを選択し、「offboarding 開始」を選択する
3. テナント無効化を実行し、業務担当者のアクセスを停止する
4. crypto-shred 実行依頼を data 担当者 + security 担当者に送付する（Backstage の escalation フォームを使用）
5. data 担当者 + security 担当者が crypto-shred を実行し完了通知を送付する
6. audit ログ（crypto-shred 完了記録）を確認し、承認文書と紐付けて保存する
7. テナント完全無効化を確認する

## Timeline（onboarding）

| T+ | actor | action | 通知例 |
|----|-------|--------|--------|
| T+0m | 業務管理者 | 承認文書受領・Backstage 開く | — |
| T+30m | 業務管理者 + tier2 | onboarding ウィザード実行・設定確認 | — |
| T+90m | 業務管理者 | test テナント動作確認 | Backstage: 「test テナント 東北工場: 動作確認完了」 |
| T+100m | 業務管理者 | 本番適用ボタン押下 | — |
| T+101m | tier2 | テナント本番作成・audit emit | Backstage: 「テナント作成完了。audit ログ発行済み」 |
| T+110m | 業務管理者 | 承認番号記録・完了報告 | Mattermost `#tenant-lifecycle`: 「東北工場テナント onboarding 完了（#ORG-2024-011）」 |

## 業界 9 業務との紐付け

| 業務名 | 影響度 | 紐付き内容 |
|--------|--------|----------|
| FA 生産指示 | 高 | 新テナントが製造業 pack で FA 生産指示業務を使用開始するベースとなる |
| ライン稼働監視 | 高 | テナントの region 設定が SCADA テレメトリ収集のネットワーク経路に影響する |
| 品質検査結果配信 | 中 | preservation_class 設定が品質検査データの保存期間に直結する |
| 受注 sub | 中 | テナントの業界 pack バージョンが受注処理ロジックを規定する |

## Backstage プラグイン操作 UI

- **テナント管理画面**: 全テナント一覧（テナント名 / region / status / 業界 pack バージョン）
- **onboarding ウィザード**: 5 ステップ（基本設定 → RLS 設定 → preservation 設定 → 業界 pack → test 確認）の guided フロー
- **test テナントダッシュボード**: test デプロイ後の動作確認チェックリスト（マスタ疎通 / RLS 動作 / audit emit）を表示
- **offboarding escalation フォーム**: crypto-shred 依頼内容（テナント名 / 対象データ範囲 / 承認番号）を入力して data + security 担当者へ送付

## 業務管理者の決定権限境界

- **実施可（tier2 担当者と協働）**: onboarding ウィザード実行 / test テナント確認 / 本番適用 / テナント無効化
- **escalation 必要（data 担当者 + security 担当者）**: crypto-shred 実行（offboarding 最終確認）/ RLS policy の技術的設定変更
- **escalation 必要（tier2 担当者）**: onboarding ウィザードでエラーが発生した場合の技術的調査

## 関連適合仕様

- [テナント分離適合仕様](../../../04_詳細設計/01_適合仕様/06_テナント分離適合仕様.md)
- [テナント容量適合仕様](../../../04_詳細設計/01_適合仕様/07_テナント容量適合仕様.md)

## 期待結果 / 観測指標 / 受入条件

- onboarding: test テナントの動作確認チェックリストが全 pass で、本番テナントが正常稼働している
- offboarding: crypto-shred 完了後に対象テナントのデータが参照不可能であることが確認できる
- 全 onboarding / offboarding 操作が audit hash chain に承認番号とともに記録されている
- テナント lifecycle の全ステップが SLA 内（onboarding: 1 営業日 / offboarding: 2 営業日）で完了している
- 受入条件: 上記 4 点が E2E テストで全て pass

## 失敗時の挙動 / escalation

- **onboarding ウィザードでエラー**: Backstage が具体的なエラー内容（例: region 設定不正）を表示。escalation 先: tier2 担当者 Mattermost `#tier2-support`（SLA: 4 時間以内）
- **test テナント動作確認失敗**: 本番適用ボタンが非活性のまま。tier2 担当者と協働で設定を修正し、test テナントを再デプロイする
- **crypto-shred 実行失敗（offboarding）**: data 担当者 + security 担当者が調査し、失敗原因を特定して再実行する。SLA: offboarding 依頼から 2 営業日以内に完了

## 失敗パターン（3 例）

1. **test テナント確認をスキップして本番適用する**: RLS 設定ミスが本番で発覚し、他テナントのデータが参照できてしまうリスクがある。test テナントのチェックリスト全 pass を本番適用の必須前提として Backstage で強制する
2. **offboarding 時に crypto-shred 依頼を後回しにする**: 契約終了後にデータが残存し、規制違反リスクが生じる。テナント無効化と同日に crypto-shred 依頼を必ず送付する SOP とする
3. **承認番号を記録せずに onboarding を完了する**: 組織変更の根拠が audit に残らず、後続の組織監査で変更正当性を説明できなくなる。ウィザードの最終ステップで承認番号入力を必須バリデーションにする

## 関連参照

- [業務管理者シナリオ INDEX](README.md)
- [01_マスタ更新_品目仕入先BOM](01_マスタ更新_品目仕入先BOM.md)
- [04_partner連携設定_IdP_federation](04_partner連携設定_IdP_federation.md)
- `arch.tier2.tier2_index`
- `req.team.tier_engineer_requirement`
