---
id: plan.tier3.scenario_webauthn_step_up_federation
axis: tier3
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.tier3.tier3_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# WebAuthn step_up + IdP federation

## 一文方針

発注承認など high-risk 操作では WebAuthn step_up チャレンジを強制し、親会社 IdP federation では Keycloak token exchange で限定スコープを付与することで、tier3 が token を保持せずに高強度の認証・認可を実現する。

## Trigger（発火条件）

発注承認など high-risk 操作で WebAuthn step_up が必要な場面、または親会社 IdP federation でパートナーアクセスを実装する時。

## 想定頻度 / 典型きっかけ

想定頻度: 四半期〜年次。典型きっかけ: 「発注承認フローに WebAuthn step_up が追加され、FIDO2 セキュリティキーを使った多段認証の UX を実装することになった」

## 主役 / 関与者

- **主役**: tier3 担当者（ジュニア級）
- **関与**: tier2 担当者（認可 API）/ infra 担当者（Keycloak 設定）/ dual reviewer

## 前提

- Keycloak が稼働中（step_up / token exchange / back-channel logout 設定済み）
- [BFF auth-edge](../../../04_詳細設計/03_クロスカッティング適合仕様/06_BFF_auth_edge.md) が short-lived token のリレーを担う設計になっている
- WebAuthn（FIDO2）認証器が業務担当者に配布済み
- （責務境界: [README 重要原則](./README.md#重要原則-tier3-が所有するもの--所有しないもの) を参照）

## 流れ

1. [v1_human_session + step_up](../../../04_詳細設計/01_適合仕様/04_認証適合仕様.md): 発注承認ボタン押下 → Keycloak step_up challenge → WebAuthn（FIDO2 authenticator）→ 承認 API 呼出し
2. 認可: step_up 後の short-lived token（スコープ限定）を BFF 経由で使用（tier3 は token を保持しない）
3. [v1_federated_exchange](../../../04_詳細設計/01_適合仕様/04_認証適合仕様.md): 親会社 IdP（SAML or OIDC）→ Keycloak token exchange → 取引先パートナーに限定スコープで tier2 API アクセス
4. logout: back-channel logout を単一受信器で受け取り、全 tier3 タブの session を同時終了
5. break-glass は tier3 UI には表示しない（業務管理 UI = Backstage プラグイン側の機能）
6. Playwright E2E: step_up フロー full walk / federation login フロー
7. dual reviewer sign-off

## 関連適合仕様 / 関連 OSS

- 認証適合仕様: [../../../04_詳細設計/01_適合仕様/04_認証適合仕様.md](../../../04_詳細設計/01_適合仕様/04_認証適合仕様.md)
- BFF_auth_edge: [../../../04_詳細設計/03_クロスカッティング適合仕様/06_BFF_auth_edge.md](../../../04_詳細設計/03_クロスカッティング適合仕様/06_BFF_auth_edge.md)
- 関連 OSS: Keycloak / Playwright / WebAuthn (FIDO2)

## 期待結果 / 観測指標

- step_up チャレンジなしに high-risk 操作 API が呼び出せない（E2E で確認）
- tier3 側に token が残存しない（BFF 経由のみ）
- back-channel logout で全タブが 5 秒以内に session 終了
- federation login で限定スコープのみ付与される
- Playwright E2E: step_up / federation フロー全 green
- dual reviewer 2 名 sign-off 完了

## 失敗時の挙動 / escalation

- tier3 が token を直接保持していることを lint が検出 → merge 阻止。**escalate 先**: security 担当者に Mattermost `#security-review` で即時報告（**SLA**: 2h 以内）。**runbook**: Backstage runbook `token-storage-violation` を参照
- break-glass UI が tier3 に混入 → merge 阻止。**escalate 先**: security 担当者に Mattermost `#security-review` で確認（**SLA**: 2h 以内）
- WebAuthn challenge 無限ループ → ユーザに「通常ログインに戻る」リンクを提示しつつ **escalate 先**: security 担当者に Mattermost `#auth-incident` で報告（**SLA**: 2h 以内）。**runbook**: Backstage runbook `webauthn-challenge-loop` を参照
- back-channel logout でタブ残存 → E2E で検出 → merge 阻止。**escalate 先**: infra 担当者に Mattermost `#infra-ops` で Keycloak back-channel logout 設定確認（**SLA**: 24h 以内）

## 関連参照

- [tier3 担当者シナリオ index](./README.md) — tier3 担当者シナリオ全体の構成と担当者プロフィール
- [05_レガシー_NetFx_Companion統合.md](./05_レガシー_NetFx_Companion統合.md) — .NET Framework 4.8 ERP での Keycloak OIDC token 連携が必要な場合の統合手順
- [認証適合仕様](../../../04_詳細設計/01_適合仕様/04_認証適合仕様.md) — v1_human_session + step_up / v1_federated_exchange の仕様定義。本シナリオの WebAuthn / federation 実装の SoT
- [BFF_auth_edge](../../../04_詳細設計/03_クロスカッティング適合仕様/06_BFF_auth_edge.md) — step_up 後の short-lived token リレーと back-channel logout 受信設計の SoT
