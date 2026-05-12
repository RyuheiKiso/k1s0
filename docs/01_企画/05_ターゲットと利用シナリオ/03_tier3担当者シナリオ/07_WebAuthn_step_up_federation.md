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
  defense_in_depth_layers: [A, B, C]
  proof_classes: []
---

# WebAuthn step_up + IdP federation

## 一文方針

発注承認など high-risk 操作では WebAuthn step_up チャレンジを強制し、親会社 IdP federation では Keycloak token exchange で限定スコープを付与することで、tier3 が token を保持せずに高強度の認証・認可を実現する。

> 午後 2 時、本社 IT 室の tier3 担当者（ジュニア級）が Playwright E2E のデバッグ中に「WebAuthn step_up チャレンジ後の承認 API 呼び出しが 401 を返す」問題に気付く。手元には BFF_auth_edge の仕様と Keycloak 管理コンソール、Mattermost 越しに tier2 担当者（認可 API）と infra 担当者（Keycloak 設定）がいる。

## ペルソナ要約

主役: tier3 担当者（ジュニア級）、目的: WebAuthn step-up 認証を BFF 経由で実装し、認証ロジックを UI に持ち込まない

## 現状業務での痛み

- 認証ロジックを UI コンポーネントに実装し、token 検証や step-up 判定が tier3 に混入してセキュリティホールが生まれる
- WebAuthn の credential 管理を tier3 が直接行い、FIDO2 仕様の更新のたびに UI を改修する必要がある
- step-up 要求の条件判定が UI に散在し、認証バイパスの脆弱性が生じやすい

## k1s0 でこう変わる

- BFF が step-up 判定と token 発行を一元管理し、tier3 は BFF から受け取った認証結果を表示するだけになる
- WebAuthn credential 管理が BFF / Keycloak に集約され、FIDO2 仕様更新の影響が UI に及ばない
- step-up 条件判定が tier2 の認可ルールに基づき BFF で実行され、UI 側のバイパスが構造的に不可能になる

## Trigger（発火条件）

発注承認など high-risk 操作で WebAuthn step_up が必要な場面、または親会社 IdP federation でパートナーアクセスを実装する時。

## 想定頻度 / 典型きっかけ

想定頻度: 四半期〜年次。典型きっかけ: 「発注承認フローに WebAuthn step_up が追加され、FIDO2 セキュリティキーを使った多段認証の UX を実装することになった」

## 主役 / 関与者

- **主役**: tier3 担当者（ジュニア級）
- **関与**: tier2 担当者（認可 API）/ infra 担当者（Keycloak 設定）/ dual reviewer

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（tier3）| ジュニア | 本社 IT 室 / リモート | GitHub PR / Playwright CI | step_up UI 実装 / federation login フロー / back-channel logout / E2E テスト |
| 関与（tier2）| 中堅 | 本社 IT 室 | Backstage Catalog | 認可 API 仕様確認 / limited scope 設計 / sign-off |
| 関与（infra）| 中堅 | 本社 IT 室 | Backstage Catalog | Keycloak step_up / token exchange / back-channel logout 設定確認 |
| 承認（dual reviewer）| 中堅〜シニア | 本社 IT 室 / リモート | GitHub PR | token 非保持 / break-glass 混入禁止 / back-channel logout 確認 / sign-off |

## 個人 KPI / 達成感

- tier3 のコードに token 検証ロジックが 0 件であることを static analysis で確認できる
- WebAuthn E2E 全シナリオ green で step-up 実装完了の達成感を得られる

## 工数 / 関与人数 / コスト感

- 工数: 1〜2 日（BFF 連携設定 2h + step-up UI 実装 4h + E2E 4h）
- 関与人数: 3〜4 名（tier3・BFF 担当者・security 担当者・dual reviewer）
- コスト感: 中。BFF が認証処理を集約するため UI 側の実装量は最小化される

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

## Timeline

| T+ | actor | action | 通知例 |
|---|---|---|---|
| 0 | tier3 担当者 | BFF の step-up endpoint 仕様を確認し UI 連携設計を作成 | `BFF 仕様確認完了 / step-up flow 設計` |
| 4h | tier3 担当者 | step-up UI（WebAuthn 認証プロンプト + 完了画面）を実装 | `step-up UI 実装完了` |
| 1d | tier3 担当者 | E2E で step-up 要求 / 認証完了 / バイパス拒否を green にし PR 提出 | `WebAuthn E2E green / PR #NNN 提出` |
| 1d+4h | dual reviewer + security 担当者 | token ロジック混入なし + step-up 動作を確認し sign-off | `sign-off 完了` |

## 業界 9 業務との紐付け

- **受注**: 発注承認フローが high-risk 操作の典型例であり、WebAuthn step_up チャレンジにより受注承認の操作権限を多段認証で保護する
- **FA 生産指示・設備操作**: 設備リモート操作など安全上 high-risk な操作においても step_up 認証を適用し、不正操作リスクを排除する

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

## 失敗パターン (anti-pattern)

- token を tier3 で保持: localStorage に token を保存する実装は security CI が検知し merge 阻止される
- step-up 条件を UI で判定: 認可ロジックを UI に書くと BFF bypass テストが fail する

## 関連参照

- [tier3 担当者シナリオ index](./README.md) — tier3 担当者シナリオ全体の構成と担当者プロフィール
- [05_レガシー_NetFx_Companion統合.md](./05_レガシー_NetFx_Companion統合.md) — .NET Framework 4.8 ERP での Keycloak OIDC token 連携が必要な場合の統合手順
- [認証適合仕様](../../../04_詳細設計/01_適合仕様/04_認証適合仕様.md) — v1_human_session + step_up / v1_federated_exchange の仕様定義。本シナリオの WebAuthn / federation 実装の SoT
- [BFF_auth_edge](../../../04_詳細設計/03_クロスカッティング適合仕様/06_BFF_auth_edge.md) — step_up 後の short-lived token リレーと back-channel logout 受信設計の SoT
