---
id: plan.overview.scenario_business_admin_partner_idp_federation
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

# partner連携設定_IdP_federation

## 一文方針

取引先 / 親会社 IdP との federation 設定（v1_federated_exchange）を Backstage プラグインで完結させ、claim mapping と audit emit を確認する。

> 四半期初め、業務管理者は新規取引先の IT 担当者から「自社 IdP（Azure AD）との federation 設定を依頼したい」という連絡を受ける。Backstage プラグインの partner 連携設定画面を開き、取引先の IdP メタデータ URL を入力して federation 設定を開始する。claim mapping を確認し（sub / email / groups の対応を設定）、test login で動作確認を行い、audit emit を確認して設定を有効化する。tier2 担当者と協働で OIDC / SAML の技術的詳細を確認しながら進める。

## ペルソナ要約

主役: 業務管理者（シニア級）、目的: 新規 partner IdP との federation を安全に設定し、claim mapping と audit 証跡を確立する

## 現状業務での痛み

- partner との連携設定に IT 部門の技術者が都度関与しなければならず、業務管理者が依頼してから設定完了まで数週間かかる
- claim mapping の設定ミスが partner ユーザのアクセス権過剰付与につながるリスクがあり、設定後の検証に工数がかかる
- federation 設定の変更履歴が IT 部門のメモにしか残らず、監査対応時に設定根拠を説明できない
- partner 離脱時の federation 無効化手順が定められておらず、アクセス権が残存するリスクがある

## k1s0 でこう変わる

- Backstage プラグインの partner 連携設定画面から業務管理者が自律的に federation 設定を開始でき、IT 部門への依頼工数が大幅に削減される
- claim mapping の設定ウィザードが権限付与の最小化原則を強制し、過剰付与が構造的に防止される
- 全 federation 設定変更が audit hash chain に記録されるため、誰がいつ何を設定したかを即座に証跡提出できる
- partner 離脱時の federation 無効化が Backstage から 1 操作で完結し、アクセス権の残存リスクが排除される

## Trigger

新規 partner との連携設定が必要になった時（partner IT 担当者からの依頼受信または社内稟議承認をトリガーとする）

## 想定頻度 / 典型きっかけ / 頻度根拠

- 想定頻度: 四半期〜年次
- 典型きっかけ: 「新規取引先との受注サブシステム連携で partner IdP federation が必要になった」「親会社の ID 基盤移行に伴い federation 設定を更新する必要が生じた」
- 頻度根拠: 製造業テナントでは年次〜四半期に新規 / 更新の partner 連携設定が発生する

## 主役 / 関与者

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|------|---|------------|----------------|----------------------|
| 業務管理者（主役） | シニア | 事務所 | Backstage プラグイン | federation 設定 / claim mapping 確認 / test login 実施 |
| tier2 担当者 | シニア | 開発拠点 | GitHub PR list | OIDC / SAML 技術詳細の支援 |
| partner IT 担当者 | — | partner 社内 | partner 社 IT システム | IdP メタデータ提供 / test user 提供 |
| security 担当者 | シニア | 開発拠点 | セキュリティダッシュボード | claim mapping の security review（必要時） |

## 個人 KPI / 達成感

- federation 設定開始から test login 成功（有効化完了）までの所要時間（目標: 1 営業日以内）
- claim mapping の最小権限原則遵守率（過剰付与グループなし）100%
- audit ログの発行確認率（設定操作ごと）100%
- partner 離脱時の federation 無効化を 1 営業日以内に完了した件数 / 年

## 工数 / 関与人数 / コスト感

- 平常（標準 OIDC federation）: 半日〜1 営業日（設定 + test login + 有効化）
- 複雑ケース（SAML / カスタム claim mapping）: 1〜2 営業日（tier2 担当者と協働）
- 失敗時（test login エラー / claim mapping 不整合）: +半日〜1 日（partner IT 担当者と協働調査）
- 関与人数: 3〜5 名（業務管理者 + tier2 担当者 + partner IT 担当者 + 必要に応じて security 担当者）

## 前提

- partner IdP のメタデータ URL / SAML メタデータ XML が partner IT 担当者から提供済みである
- 社内稟議で partner 連携が承認されており、承認番号が存在する
- tier2 担当者が技術支援に参加可能な状態である
- v1_federated_exchange が tier2 で稼働している

## 流れ

1. partner IT 担当者から IdP メタデータ（OIDC Discovery URL または SAML メタデータ XML）と test user 情報を受け取る
2. Backstage プラグインの partner 連携設定画面を開き、「新規 federation 追加」を選択する
3. partner 名 / 承認番号 / protocol（OIDC / SAML）を入力し、IdP メタデータ URL を貼り付ける
4. claim mapping ウィザードを開き、sub / email / groups の mapping を設定する（最小権限原則に基づき groups は必要最小限のみ許可）
5. tier2 担当者と mapping 内容を確認し、過剰付与のリスクがないことをチェックする
6. test login を実行し、partner の test user でログインを試みる
7. test login 成功・権限付与が mapping 通りであることを確認する
8. 「有効化」ボタンを押下し、audit ログ発行を確認する
9. Mattermost `#partner-integration` に設定完了を報告する

## Timeline

| T+ | actor | action | 通知例 |
|----|-------|--------|--------|
| T+0m | 業務管理者 | IdP メタデータ受領・Backstage 開く | メール: 「OIDC Discovery URL を送付しました」 |
| T+30m | 業務管理者 | federation 新規追加・メタデータ入力 | — |
| T+45m | 業務管理者 + tier2 | claim mapping ウィザード確認 | — |
| T+90m | 業務管理者 | test login 実行 | Backstage: 「test login 成功: test_user@partner.example」 |
| T+100m | 業務管理者 | 権限付与の確認（mapping 通り） | — |
| T+105m | 業務管理者 | 「有効化」ボタン押下 | — |
| T+106m | tier2 | audit emit | Backstage: 「federation 有効化完了。audit ログ発行済み」 |
| T+110m | 業務管理者 | Mattermost 完了報告 | Mattermost `#partner-integration`: 「partner社 federation 有効化完了（承認 #PA-2024-012）」 |

## 業界 9 業務との紐付け

| 業務名 | 影響度 | 紐付き内容 |
|--------|--------|----------|
| 受注 sub | 高 | partner 企業の受注サブシステム連携に federation が必要なケースが多い |
| 図面 collaborative review | 中 | partner 設計担当者が図面 review に参加する際に federation 認証が使用される |
| 在庫最新値 | 中 | 取引先への在庫情報共有で partner federated identity が使用される |

## Backstage プラグイン操作 UI

- **partner 連携設定画面**: 既存 federation 一覧 + 新規追加ボタン。partner 名 / protocol / 有効 / 無効ステータスを表示
- **claim mapping ウィザード**: sub / email / groups の mapping を step-by-step で設定。最小権限チェックが各ステップに組み込まれている
- **test login パネル**: test user の ID / パスワードを入力して federation ログインを試験。成功 / 失敗と付与された権限一覧を表示
- **有効化 / 無効化トグル**: federation の有効 / 無効を 1 タップで切替。変更ごとに audit ログが emit される

## 業務管理者の決定権限境界

- **実施可（単独 + tier2 担当者協働）**: OIDC / SAML federation 設定 / claim mapping / test login / 有効化 / 無効化
- **escalation 必要（security 担当者）**: claim mapping に特権グループが含まれる場合の security review / federation 設定の外部セキュリティ監査
- **escalation 必要（tier2 担当者）**: IdP メタデータの解析 / SAML カスタム属性のパース実装 / federation 設定の技術的エラー解消

## 関連適合仕様

- [認証適合仕様](../../../04_詳細設計/01_適合仕様/04_認証適合仕様.md)
- BFF_auth_edge

## 期待結果 / 観測指標 / 受入条件

- test login が partner の test user で成功し、付与された権限が mapping 通りである
- 全設定操作が audit hash chain に記録されている
- federation 有効化後、partner ユーザが対象テナントの業務 UI にアクセス可能になっている
- claim mapping で意図しない特権グループへのアクセスが付与されていない
- 受入条件: 上記 4 点が E2E テストで全て pass

## 失敗時の挙動 / escalation

- **test login 失敗（IdP 設定不一致）**: Backstage が具体的なエラーコード（例: invalid_issuer）を表示。partner IT 担当者に IdP 設定の確認を依頼する
- **claim mapping で特権グループが検出**: Backstage が「特権グループが含まれています。security 担当者の review が必要です」と表示し、有効化をブロックする。escalation 先: security 担当者 Mattermost `#security-review`（SLA: 1 営業日以内に review）
- **SAML メタデータ解析エラー**: tier2 担当者に SAML XML の技術的確認を依頼する。escalation 先: Mattermost `#tier2-support`（SLA: 4 時間以内に対応確認）

## 失敗パターン（3 例）

1. **claim mapping の groups 設定で全グループを許可してしまう**: partner ユーザに意図しない広範な権限が付与される。mapping ウィザードの「必要最小限のグループのみ選択」ガイダンスに従い、tier2 担当者と一緒に確認する
2. **test login なしで直接有効化する**: claim mapping の誤りが本番で発覚し、partner ユーザがアクセスできない / 過剰アクセスができる状態になる。test login 成功を有効化の必須前提として手順書に明記する
3. **partner 離脱後に federation を無効化しないままにする**: 契約終了後も partner ユーザがアクセスできる状態が継続するセキュリティリスクが発生する。partner 契約終了通知を受信したら 1 営業日以内に無効化する SOP を確立する

## 関連参照

- [業務管理者シナリオ INDEX](README.md)
- [07_テナント_onboarding_offboarding](07_テナント_onboarding_offboarding.md)
- `arch.security.security_index`
- `req.team.tier_engineer_requirement`
