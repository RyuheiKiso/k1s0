---
id: plan.tier3.scenario_pre_release_smoke_test
axis: tier3
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.tier3.tier3_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [C]
  proof_classes: []
---

# pre-release smoke test

## 一文方針

tier3 担当者が release 前に担当業務画面の Playwright smoke test を追加・維持し、全画面の golden path が CI で自動検証される状態を保つ。

> 午前 10 時、本社 IT 室の tier3 担当者（ジュニア級）がリリース前日の Playwright CI ダッシュボードで smoke test の一斉 fail を発見し、緊張した面持ちで原因確認を開始する。手元には smoke test の fail ログと tier2 generated stub の差分レポート、Mattermost 越しに tier2 担当者（API 変更確認）と dual reviewer がいる。

## ペルソナ要約

主役: tier3 担当者（ジュニア級）、目的: pre-release smoke test を実施し全シナリオ green をデプロイ前に確認する

## 現状業務での痛み

- smoke test がなくデプロイ後に重大な機能不全が発覚し、ユーザー影響が出てからロールバックする
- テスト環境と本番環境の差異により staging では green だったテストが本番で fail する
- smoke test の手順が属人化しており、担当者によってカバレッジが異なる

## k1s0 でこう変わる

- Playwright smoke test が CI パイプラインに組み込まれ、デプロイ前に全シナリオ green を自動確認する
- 本番同等 staging 環境での smoke test が必須となり、環境差異による本番 fail を事前検知する
- smoke test シナリオが Backstage TechDocs に定義されており、誰が実行しても同一カバレッジが保証される

## Trigger（発火条件）

新業務画面の追加完了時 / リリース milestone 到達時 / smoke test が CI で fail した時

## 想定頻度 / 典型きっかけ

想定頻度: 月次（新画面追加の都度）。典型きっかけ: 「FA 生産指示画面の実装完了後、smoke test に新画面の基本フロー（画面表示 → データ入力 → 送信 → 結果確認）を追加した」「リリース前に smoke test が CI fail し、原因が tier2 API の breaking change であることを確認した」

## 主役 / 関与者

- **主役**: tier3 担当者（ジュニア級）
- **関与**: tier2 担当者（API 変更確認）
- **承認**: dual reviewer（tier3 担当者 1 名 + tier2 担当者 1 名）

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（tier3）| ジュニア | 本社 IT 室 / リモート | Playwright CI / GitHub PR | smoke test 作成・修正 / fail 原因分類 / fixture 更新 / staging 確認 |
| 関与（tier2）| 中堅 | 本社 IT 室 | Backstage Catalog | API 変更内容の migration guide 提供 / smoke test fail 原因確認 / sign-off |
| 承認（dual reviewer）| 中堅〜シニア | 本社 IT 室 / リモート | GitHub PR | CI green / staging smoke pass / a11y green / sign-off |

## 個人 KPI / 達成感

- smoke test 全 green がデプロイ可否ゲートとなり、デプロイ成功率が定量的に向上する
- smoke test 通過率の推移をダッシュボードで確認でき、品質改善の達成感を得られる

## 工数 / 関与人数 / コスト感

- 工数: 0.5〜1 日（smoke test シナリオ設計 2h + Playwright 実装 4h）
- 関与人数: 2〜3 名（tier3・ops 担当者・dual reviewer）
- コスト感: 低。一度実装すると CI で自動実行されるため継続コストは最小化される

## 前提

- Playwright が CI（GitHub Actions / Tekton）に組込済み
- tier2 generated stub が最新
- （責務境界: [README 重要原則](./README.md#重要原則-tier3-が所有するもの--所有しないもの) を参照）

## 流れ

1. 担当業務画面の smoke test ファイルを確認する（`e2e/smoke/<business-domain>/<screen-name>.spec.ts`）
2. 新画面の場合: smoke test ファイルを新規作成する
   - golden path（正常系）のテストシナリオを 1〜3 件記述する
   - `page.goto('/manufacturing/fa-production-instruction')` → ログイン → データ入力 → 送信 → 結果確認 の最小フロー
   - tier2 generated stub の mock（MSW）を使い外部依存をモック化する
3. 既存画面の smoke test が fail している場合: fail 原因を分類する
   - tier2 API 変更 → tier2 担当者に変更内容の確認を依頼（Mattermost `#tier3-contract-fail`）
   - UI コンポーネント変更 → selector を修正
   - テストデータ不整合 → fixture を更新
4. axe-core a11y test を smoke test と同時実行し、WCAG 2.1 AA 違反がないことを確認する
5. 全 smoke test が local で green になることを確認してから PR を作成する
6. CI で smoke test + a11y test が green になることを確認する
7. dual reviewer sign-off を取得し、staging 環境でも smoke test を実行して動作確認する

## Timeline

| T+ | actor | action | 通知例 |
|---|---|---|---|
| 0 | tier3 担当者 | Backstage TechDocs で smoke test シナリオ一覧を確認し実装計画を作成 | `smoke test シナリオ N 件 / 実装計画確定` |
| 4h | tier3 担当者 | Playwright で smoke test を実装し staging で green を確認 | `smoke test N 件実装 / staging green` |
| 1d | tier3 担当者 | CI パイプラインに smoke test を組み込み PR 提出 | `CI 統合完了 / PR #NNN 提出` |
| 1d+2h | dual reviewer | CI 統合と smoke test カバレッジを確認し sign-off | `sign-off 完了` |

## 業界 9 業務との紐付け

- **FA 生産指示・設備操作**: FA 生産指示画面の smoke test が golden path（画面表示 → 指示入力 → 送信 → 結果確認）をリリースごとに自動検証し、リリース品質を保証する
- **受注**: 受注一覧・受注入力・承認フローの smoke test が、受注業務の基本動作をリリース前に網羅的に確認する
- **ライン稼働監視・進捗実績**: ライン監視画面の smoke test によりリアルタイム更新 UI の基本動作がリリースごとに担保される
- **警報配信**: 警報通知 UI の smoke test が警報配信フローの golden path をリリース品質として維持する

## 関連適合仕様 / 関連 OSS

- クライアント状態適合仕様: [../../../04_詳細設計/01_適合仕様/11_クライアント状態適合仕様.md](../../../04_詳細設計/01_適合仕様/11_クライアント状態適合仕様.md)
- tier3 強制機構: [../../../04_詳細設計/02_強制機構/03_tier3強制機構.md](../../../04_詳細設計/02_強制機構/03_tier3強制機構.md)
- 関連 OSS: Playwright（E2E / smoke test）/ axe-core（a11y 自動検査）/ MSW（Mock Service Worker）

## 期待結果 / 観測指標

- ci: CI job `tier3-smoke-tests`（release branch）all green（main / feature branch ではなく release branch での green が必須）
- ci: CI job `tier3-a11y-tests`（axe-core）all green（WCAG 2.1 AA 違反 = 0 件）
- env: staging 環境での `playwright test --project=smoke` 全件 pass
- sign-off: dual reviewer（tier3 担当者 1 名 + tier2 担当者 1 名）sign-off 完了

## 失敗時の挙動 / escalation

- **tier2 API 変更により smoke test が一斉 fail**: tier2 担当者に Mattermost `#tier3-contract-fail` で連絡（**SLA: 24h 以内**に tier2 側で migration guide 提供）。smoke test は修正前に merge しない。
- **a11y 違反が検出された**: WCAG 2.1 AA 違反は merge 阻止。デザインシステム担当者に Mattermost `#design-review` で修正方針を確認（**SLA: 48h 以内**）。
- **smoke test が staging のみで fail（local は green）**: 環境差分（API URL / 認証 endpoint）を確認。ops 担当者に Mattermost `#infra-incident` で staging 環境の状態を確認依頼（**SLA: 2h 以内**）。

## 失敗パターン (anti-pattern)

- happy path のみの smoke test: エラーシナリオを省くと本番の failure mode が検知できない
- staging のみで確認: 本番同等環境での実行を省略すると環境差異による本番 fail を防げない

## 関連参照

- [tier3 担当者シナリオ index](./README.md) — tier3 担当者シナリオ全体の構成と dual reviewer 規約
- [新業務画面追加](./01_新業務画面追加.md) — smoke test 追加の前提となる業務画面実装シナリオ
- [a11y / i18n 修正](./08_a11y_i18n修正.md) — smoke test で a11y 違反が検出された場合の修正フロー
- [tier3 設計方針 テスト方針](../../../03_概要設計/04_tier3設計方針/10_テスト方針.md) — Playwright E2E / a11y / Lighthouse の CI 組み込み方針
