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
  defense_in_depth_layers: []
  proof_classes: []
---

# tenant_id 強制注入追加

## 一文方針

新 API / 新リポジトリ抽象を追加した際に tenant_id 強制注入の漏れが CI で検出された場合、lint 違反箇所を特定してラッパーを追加し、CloudNativePG RLS との整合・cross-tenant leak の Testcontainers 検証まで完遂する。

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

## 関連参照

- [tier2 担当者シナリオ index](README.md)
- [tier2 設計方針](../../../03_概要設計/03_tier2設計方針/README.md)
- [テナント別 override 拡張点](03_テナント別override拡張点.md)
