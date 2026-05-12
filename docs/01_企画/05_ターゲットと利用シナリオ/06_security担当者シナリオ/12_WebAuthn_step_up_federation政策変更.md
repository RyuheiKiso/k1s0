---
id: plan.security.scenario_webauthn_step_up_federation_policy_change
axis: security
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.security.security_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [B, D, E]
  proof_classes: []
---

# WebAuthn step_up / federation 政策変更

## 一文方針

IdP federation の追加・step_up 要求境界の変更・auth_class の追加が必要になった時に、Keycloak 設定 + Envoy `jwt_authn` filter + `idp_capabilities.lock.yaml` を同期更新し、認証適合仕様の 5 auth_class × 5 dimension cross-product が CI で pass することを確認してクローズする。

## Trigger（発火条件）

新規 IdP federation の接続依頼が来た時、または特定業務操作（発注承認 / 緊急操作）に step_up（WebAuthn / TOTP）を追加 / 変更する設計変更 PR が tier2 から来た時。

## 想定頻度 / 典型きっかけ

想定頻度: 不定期（年 1-3 件）。典型きっかけ: 「取引先パートナー企業（親会社）の Okta を IdP として federation 追加したい。`v1_federated_exchange` auth_class として認証適合仕様に収容し、Keycloak の Identity Provider として Okta OIDC を追加する」

## 主役 / 関与者

- 主役: security 担当者（シニア級）
- 関与: infra 担当者（Keycloak config の GitOps 更新、Envoy `jwt_authn` filter の設定変更）
- 関与: tier1 担当者（`idp_capabilities.lock.yaml` の build artifact 更新確認）
- 確認: tier2 担当者（step_up 要求境界変更が API 設計に与える影響の確認）

## 前提

- `idp_capabilities.lock.yaml` が build artifact として存在し、auth_class × dimension の cross-product が CI で管理済み
- Keycloak の設定は GitOps 管理（Argo CD 経由でのみ反映可能、手動 Keycloak UI 変更は禁止）
- Envoy `jwt_authn` filter の設定が `backends.lock.yaml` で管理済み
- DPoP（Demonstration of Proof of Possession）が全 auth_class に必須

## 流れ

1. **federation 追加の場合**:
   - 接続先 IdP（Okta / Azure AD 等）のメタデータ（OIDC Discovery URL / JWKS endpoint）を取得し、Keycloak の GitOps config に IdP entry として追加する PR を起票する
   - Keycloak から発行される federated token の claim mapping（`sub` / `email` / `groups`）を `idp_capabilities.lock.yaml` に宣言する
   - Envoy `jwt_authn` filter の `providers` section に federated issuer を追加し、JWKS URI を指定する
   - DPoP bind の強制: federated token に DPoP nonce が bind されることを Envoy filter config で確認する
2. **step_up 要求境界変更の場合**:
   - 変更対象の tier2 API endpoint（または Temporal workflow activity）を特定し、現在の auth_class（`v1_human_session`）に step_up trigger 条件を追加する
   - Keycloak の Authentication Flow に WebAuthn Authenticator step を追加（または TOTP challenger を追加）する GitOps PR を起票する
   - `threat_model.lock.yaml` の対応 cell（asset = `v1_business_data` / surface = `v1_north_south`）の mitigation_class = `v1_authn_authz` pointer を更新する（シナリオ 01 の手順）
3. staging 環境で新 IdP / 新 step_up の E2E 確認を Playwright で実施する（既存テストに federation / step_up シナリオを追加）
4. CI の認証適合仕様 coverage check（5 auth_class × 5 dimension が全 pointer を持つか）を通過させる
5. security 担当者 2 名の dual reviewer sign-off を取得し、`idp_capabilities.lock.yaml` を更新してクローズする

## 関連適合仕様 / 関連 OSS

- 認証適合仕様: [../../../04_詳細設計/01_適合仕様/04_認証適合仕様.md](../../../04_詳細設計/01_適合仕様/04_認証適合仕様.md)
- security 強制機構: [../../../04_詳細設計/02_強制機構/06_security強制機構.md](../../../04_詳細設計/02_強制機構/06_security強制機構.md)
- 関連 OSS: Keycloak / Envoy Gateway `jwt_authn` filter / WebAuthn / DPoP / Playwright / Argo CD

## 期待結果 / 観測指標

- `idp_capabilities.lock.yaml` に新 federation / step_up が 5 dimension すべての pointer と共に記録済み
- Keycloak + Envoy `jwt_authn` filter の staging E2E が green（Playwright）
- CI の認証適合仕様 coverage check が green
- DPoP bind が federated token に強制されていることを確認済み
- dual reviewer sign-off 記録済み

## 失敗時の挙動 / escalation

- **federated token に DPoP bind が欠落している**: DPoP なしの federation は採用しない。接続先 IdP が DPoP をサポートしない場合、federation を拒否し、代替案（サービスアカウント + mTLS）を提示する。Mattermost `#security` に理由を記録する
- **Keycloak GitOps 反映後に既存 auth flow が壊れた（SLO 影響）**: Argo CD rollback で前 config を restore する（`argocd app rollback keycloak --revision <prev>`）。staging での regression test を全 auth_class で通過させてから再適用する
- **Playwright E2E で WebAuthn が自動化できない**: WebAuthn の hardware authenticator を staging 環境で Virtual Authenticator（Chrome DevTools Protocol）に差し替えて E2E を実施する。production は hardware authenticator のみ許容

## 関連参照

- [security 担当者シナリオ index](./README.md) — security 担当者シナリオ全体の構成と主要分類一覧
- [認証適合仕様](../../../04_詳細設計/01_適合仕様/04_認証適合仕様.md) — 5 auth_class × 5 dimension の structural spec
- [security 設計方針: 境界制御方針](../../../03_概要設計/07_security設計方針/02_境界制御方針.md) — 6 surface × 4 mitigation の trust boundary 設計
- [BFF auth-edge](../../../04_詳細設計/03_クロスカッティング適合仕様/06_BFF_auth_edge.md) — httpOnly cookie / silent renew / DPoP の技術閉鎖 cross-cutting 仕様
