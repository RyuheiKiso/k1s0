---
id: plan.tier2.scenario_tenant_id_injection
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

# tenant_id 強制注入追加

## 一文方針

新 API / 新リポジトリ抽象を追加した際に tenant_id 強制注入の漏れが CI で検出された場合、lint 違反箇所を特定してラッパーを追加し、CloudNativePG RLS との整合・cross-tenant leak の Testcontainers 検証まで完遂する。

> 朝 10 時、本社 IT 室の tier2 担当者（中堅級）が Mattermost `#tier2-ci-alert` で cross-tenant leak test fail の自動通知に気付く。手元には lint レポート・GitHub PR、Mattermost 越しに infra 担当者がいる。

## ペルソナ要約

主役: tier2 担当者（中堅級）、目的: 新 API / 新リポジトリ抽象での tenant_id 強制注入漏れを CI で検出し CloudNativePG RLS 整合・cross-tenant leak の Testcontainers 検証まで完遂する

## 現状業務での痛み

- 新 API 追加時に tenant_id フィルタの追加を忘れ、cross-tenant データ漏洩リスクが本番 API に混入する
- CloudNativePG の RLS FORCE 設定との整合確認が手動で属人的になっており、infra との連携が抜け落ちることがある
- 成りすまし可能な API（任意 tenant_id を引数で受け付ける）の全件スキャンが手動でしか実施できない
- CI で検出されても是正 SLA が不明確で、24h 以上放置されるケースがある

## k1s0 でこう変わる

- tenant_id 強制注入 lint が CI に組み込まれ、注入漏れ API の merge を物理拒否する
- Testcontainers integration test が cross-tenant data leak ゼロを merge 条件として自動検証する
- 全 API の成りすまし可能経路スキャンを lint で自動化し、手動スキャンの属人性をなくす
- 是正 SLA（24h 以内）が Backstage runbook に明文化され、放置を構造的に防止する

## Trigger（発火条件）

新 API / 新リポジトリ抽象を追加した際に tenant_id 強制注入の漏れが CI で検出された時。

## 想定頻度 / 典型きっかけ

- 想定頻度: イベント駆動（lint fail 時）
- 典型きっかけ: 「新規 API エンドポイントに tenant_id フィルタが漏れており、CI の cross-tenant leak test が fail した」
- 典型きっかけ 2: 「`/v1/orders/{id}` エンドポイントに tenant フィルタが欠落し cross-tenant leak test が fail した」

## 主役 / 関与者

- 主役: tier2 担当者（中堅級）
- 関与: infra 担当者（CloudNativePG RLS 設定の整合確認）
- 承認: dual reviewer（tier2 担当者 2 名、変更 PR の author 不可）

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（tier2）| 中堅 | 本社 IT 室 | Mattermost `#tier2-ci-alert` / GitHub PR | lint 違反特定 / tenant_id ラッパー追加 / Testcontainers 検証 |
| 関与（infra）| シニア | 本社 / リモート | GitHub PR | CloudNativePG RLS 設定確認 / 整合確認 |
| 承認（dual reviewer）| 中堅〜シニア | 本社 / リモート | GitHub PR | PR レビュー / sign-off（author 不可） |

## 個人 KPI / 達成感

- テナント分離 lint green（注入漏れ 0 件）
- Testcontainers cross-tenant leak test green（0 件の cross-tenant データアクセス）
- 成りすまし可能 API scan: 0 件検出
- dual reviewer 応答時間 ≤ 24h

## 工数 / 関与人数 / コスト感

- 初回（新 API の lint fail 是正）: 2〜4h（lint 違反特定・ラッパー追加・RLS 確認・Testcontainers 検証）、関与 3〜4 名（主役 + infra 担当者 + dual reviewer 2 名）
- 平常（定期的な全件スキャン）: 半日、関与 2〜3 名
- 失敗時（cross-tenant leak 本番流入疑い）: 即時 incident 対応・postmortem、関与 5 名以上

## 前提

- テナント分離適合仕様が確立済みで、tenant_id 強制注入 lint が CI に組み込まれていること
- CloudNativePG の RLS FORCE 設定が infra 軸で適用済みであること
- Keycloak により tenant_id が JWT claim として発行されていること

## 流れ

1. lint 違反箇所（tenant_id なしで呼べる API）を特定する
2. tier2 リポジトリ抽象に tenant_id 述語を強制注入するラッパーを追加する
3. 生 SQL 経路が tier2 アプリ層に露出していないか確認する（SQL 文字列引数を受け付ける API の提供は禁止）
4. CloudNativePG の RLS（Row Level Security）FORCE 設定と整合していることを infra 担当者と確認する
5. 任意の tenant_id を引数で受け付ける成りすまし可能 API が残っていないか全 API を scan する
6. Testcontainers integration test で cross-tenant data leak がないことを確認する
7. dual reviewer sign-off を得てから merge する

## Timeline

| T+ | actor | action | 通知例 |
|---|---|---|---|
| 0 | tier2 担当者 | lint レポート確認・注入漏れ API 特定 | `lint fail: /v1/orders/{id} に tenant フィルタ欠落 / ラッパー追加開始` |
| 1〜2h | tier2 担当者 | tenant_id 述語ラッパー追加・生 SQL 経路確認 | `ラッパー追加完了 / 生 SQL 経路: なし` |
| 2〜3h | tier2 担当者 / infra 担当者 | CloudNativePG RLS FORCE 整合確認 | `RLS FORCE 整合確認完了 / EXPLAIN ANALYZE: RLS filter 付き` |
| 3〜4h | tier2 担当者 | Testcontainers cross-tenant leak test 実行 | `cross-tenant leak test: green (0 件)` |
| 1 日 | dual reviewer | sign-off | `dual sign-off 完了 / テナント分離 lint: green` |

## 業界 9 業務との紐付け

- **FA 生産指示・設備操作**: tenant_id 注入の漏れを修正することで、特定テナントの生産指示が他テナントの設備 API に誤送信されるリスクを排除する。
- **在庫**: 在庫リポジトリ抽象での tenant_id 漏れは cross-tenant 在庫参照につながるため、修正により全テナントの在庫分離が保証される。
- **受注**: 受注 API の tenant_id 漏れ修正により、テナント間での受注データ混在を防止する。
- **計量装置・出荷指示**: 出荷指示 API に tenant_id 注入が追加されることで、テナント別出荷データの厳密分離が実現する。

## 関連適合仕様 / 関連 OSS

- [テナント分離適合仕様](../../../04_詳細設計/01_適合仕様/10_テナント分離適合仕様.md)
- [tier2 強制機構](../../../04_詳細設計/02_強制機構/02_tier2強制機構.md)
- CloudNativePG（RLS FORCE によるデータアクセス制御）
- Keycloak（tenant_id = JWT claim として発行）
- Testcontainers（cross-tenant leak の integration test）

## 期待結果 / 観測指標

- テナント分離 lint が green になること
- 全 API で tenant_id 強制注入が確認されること
- Testcontainers integration test で cross-tenant data leak が検出されないこと
- 生 SQL 経路が tier2 アプリ層に露出していないことが確認されること

## 失敗時の挙動 / escalation

- **テナント分離 lint fail**: merge 阻止。escalate 先: Mattermost `#tier2-ci-alert`（SLA: 24h 以内に是正 PR 提出）。runbook: Backstage `tenant-isolation-incident-procedure`
- **cross-tenant leak Testcontainers 検出**: 即時 incident として security 担当と tier2 担当者が連携対応（本番流入防止が最優先）。escalate 先: Mattermost `#security-incident`（SLA: 15 分以内に security 担当者へ通報、postmortem は 3 営業日以内）。runbook: Backstage `tenant-isolation-incident-procedure`
- **成りすまし可能 API scan 検出**: API を即時無効化する。escalate 先: Mattermost `#security-incident`（SLA: 1h 以内に security 担当者へ連絡）。runbook: Backstage `tenant-isolation-incident-procedure`

## 失敗パターン (anti-pattern)

- **生 SQL 経路を tier2 アプリ層に残す**: SQL 文字列引数を受け付ける API が残存し、呼び出し側が tenant_id フィルタを付け忘れると cross-tenant 漏洩が発生する。生 SQL 経路の tier2 アプリ層への露出を lint で物理検出し、全て tier2 リポジトリ抽象経由に統一する。
- **Testcontainers なしで「コードレビューのみ」で確認**: ラッパーを追加したつもりでも別の経路から直接アクセスされる可能性が残る。cross-tenant leak の Testcontainers integration test を是正 PR の merge 条件として強制し、コードレビューのみによる確認を構造的に不可能にする。
- **cross-tenant leak 検出を「可能性あり」のまま放置**: 本番への影響が確認できないまま時間が経ち、被害範囲が拡大する。cross-tenant leak が Testcontainers で検出された時点で即時 security incident として security 担当者へ通報し、本番流入防止を最優先で対処する。

## 関連参照

- [tier2 担当者シナリオ index](README.md)
- [tier2 設計方針](../../../03_概要設計/03_tier2設計方針/README.md)
- [テナント別 override 拡張点](03_テナント別override拡張点.md)
