---
id: plan.tier2.scenario_permission_model_change
axis: tier2
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.tier2.tier2_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers:
    - B
    - C
  proof_classes: []
---

# 権限モデル変更 / ABAC Delegation

## 一文方針

tier2 担当者が Keycloak ABAC 権限ルールの追加・変更および Delegation 設定を実施し、最小権限原則と業務ロールの整合を維持する。

## Trigger（発火条件）

新業務ロールの追加 / 既存ロールの権限範囲変更 / 業務担当者からの権限追加申請が承認された時

## 想定頻度 / 典型きっかけ

- 想定頻度: 月次〜四半期
- 典型きっかけ: 「製造業 pack FA ドメインで「設備管理リーダー」ロールを新設し、設備マスタの read-write 権限と緊急停止 API の呼び出し権限を付与する必要が生じた」「取引先パートナーへの閲覧専用アクセス（Delegation）設定で、発注情報のみ参照できるロールを追加した」

## 主役 / 関与者

- 主役: tier2 担当者（中堅級）
- 関与: security 担当者（権限設計 review）/ 業務管理者（ロール定義の確認）
- 承認: dual reviewer（tier2 担当者 + security 担当者、変更 PR の author 不可）

## 前提

- [権限モデル方針](../../../03_概要設計/03_tier2設計方針/09_権限モデル.md)（Keycloak ABAC）が確立済み
- Keycloak の Realm / Client 設定が GitOps 管理済み

## 流れ

1. 新ロールまたは権限変更の要件を業務管理者から受領し、最小権限原則に照らして精査する
   - 必要なリソース（API endpoint / DB table / UI 機能）を列挙
   - 不要な権限が含まれていないか security 担当者と協議
2. Keycloak の Realm / Client 設定（`keycloak/realm-export.json`）に新ロールを追加する
   - ABAC の属性ルール（例: `tenant_id = current_tenant AND role = fa-equipment-leader`）を定義
   - Delegation の場合: scope を限定した token（例: `orders:read-only` スコープのみ）を設定
3. tier2 API の認可ポリシー（`authorization/policies/<domain>.rego`）を更新する
   - 新ロールが許可するエンドポイントのみ OPA policy を追加
   - 「deny-by-default」原則を維持（許可リスト形式）
4. テストユーザーで新ロールの動作を確認する（Testcontainers + Keycloak TestContainer）
   - 許可されるべき API が 2xx を返すこと
   - 禁止されるべき API が 403 を返すこと
5. security 担当者のレビューを受ける（Mattermost `#security-review` で PR を共有）
6. dual reviewer sign-off を取得して merge する
7. staging 環境で業務管理者に実際の動作を確認してもらい、承認を得る

## 関連適合仕様 / 関連 OSS

- 関連適合仕様: [認証適合仕様](../../../04_詳細設計/01_適合仕様/04_認証適合仕様.md) / [テナント分離適合仕様](../../../04_詳細設計/01_適合仕様/10_テナント分離適合仕様.md) / [tier2 強制機構](../../../04_詳細設計/02_強制機構/02_tier2強制機構.md)
- 関連 OSS: Keycloak（IdP / ABAC）/ OPA（Open Policy Agent）/ Testcontainers（認可テスト）

## 期待結果 / 観測指標

- ci: Testcontainers 認可テスト（許可 API = 2xx / 禁止 API = 403）all green
- security: security 担当者 review 済み
- staging: 業務管理者が staging 環境での動作確認済み
- sign-off: dual reviewer（tier2 + security）sign-off 完了

## 失敗時の挙動 / escalation

- **テストで禁止 API が 2xx を返した（権限過剰）**: 変更を即時 rollback し、security 担当者に Mattermost `#security-incident` で通報（**SLA: 30 分以内**）。**postmortem 期限: 2 営業日以内**。
- **Delegation 設定で本来許可されていない API にアクセスできた**: security incident として security 担当者と ops 担当者に即時通報（**SLA: 15 分以内**）。

## 関連参照

- [tier2 担当者シナリオ index](./README.md) — tier2 担当者シナリオ全体の構成と dual reviewer 規約
- [テナント別 override 拡張点](./03_テナント別override拡張点.md) — テナント固有の権限設定を override 拡張点で実装する場合のシナリオ
- [tier2 設計方針 権限モデル](../../../03_概要設計/03_tier2設計方針/09_権限モデル.md) — Keycloak ABAC / Delegation の設計指針
