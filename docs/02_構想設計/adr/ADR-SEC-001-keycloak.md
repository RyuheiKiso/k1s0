# ADR-SEC-001: 認証認可基盤に Keycloak を採用

- ステータス: Accepted
- 起票日: 2026-04-19
- 決定日: 2026-04-19
- 起票者: kiso ryuhei
- 関係者: システム基盤チーム / セキュリティチーム / 情シス運用 / 法務部

## コンテキスト

k1s0 は JTC 社内の多数のシステムから利用されるプラットフォームであり、認証認可基盤の選定は以下の要件を同時に満たす必要がある。

- **SSO**: JTC 既存の Active Directory/LDAP 連携で業務システムと共通の ID で SSO（NFR-E-AC、NFR-D-OBJ-003）
- **OpenID Connect / OAuth 2.0 標準準拠**
- **テナント分離**: Realm ごとのテナント境界、テナント管理者によるユーザー管理
- **認可**: RBAC / ABAC の両方、細粒度アクセス制御
- **監査ログ完整性**: 認証・認可イベントの監査ログ
- **オンプレミス完結**
- **外部 IdP 連携**: SAML/OIDC で他社製 IdP（Okta、OneLogin、Azure AD 等）と連携可能性を残す

候補は Keycloak、Zitadel、Authelia、Authentik、Ory Hydra/Kratos など。

## 決定

**認証認可基盤は Keycloak（Apache 2.0、Red Hat）を採用する。**

- Keycloak 26+（2024〜2025 のメジャーリリース）
- Realm ごとにテナント分離
- 永続化は PostgreSQL（ADR-DATA-001）
- SSO は OpenID Connect、Kerberos/LDAP 連携で JTC 既存 AD と統合
- JWT 署名鍵は OpenBao（ADR-SEC-002）で管理、定期ローテーション
- ワークロード ID（サービス間）は SPIFFE（ADR-SEC-003）と併用、ユーザー認証のみ Keycloak
- Backstage / Grafana / Argo CD / k1s0 ポータルすべて Keycloak SSO に統合

## 検討した選択肢

### 選択肢 A: Keycloak（採用）

- 概要: Red Hat 発、Apache 2.0、IAM の OSS デファクト
- メリット:
  - 業界での採用実績圧倒的（Red Hat SSO として商用サポートも取得可能）
  - OpenID Connect / OAuth 2.0 / SAML 2.0 / WS-Federation などプロトコル完備
  - Realm によるマルチテナント機能が標準
  - RBAC / ABAC / Authorization Services（UMA 2.0）対応
  - LDAP/AD 連携、Kerberos SPNEGO 対応
  - カスタム SPI（Service Provider Interface）で拡張可能
  - 監査ログ・イベントリスナー標準搭載
- デメリット:
  - JVM ベースで起動時間・メモリ使用量が他選択肢より多い（Quarkus 化で改善）
  - 設定項目が多く、学習曲線が急
  - Red Hat 方針変更リスク（ただし Apache 2.0 ライセンスは維持されている）

### 選択肢 B: Zitadel

- 概要: Go 製、Apache 2.0、モダンなクラウドネイティブ設計
- メリット: 起動軽量、マルチテナント設計が Keycloak より洗練
- デメリット:
  - コミュニティ規模が Keycloak より小さい
  - JTC 既存 AD 連携の実績が薄い（Keycloak の LDAP/Kerberos 対応が強力）

### 選択肢 C: Authelia + Authentik

- 概要: 軽量 IdP、個人利用・小規模向け
- メリット: 軽量、設定容易
- デメリット:
  - エンタープライズ機能（UMA、SPI 拡張）が薄い
  - JTC 規模の運用実績が乏しい

### 選択肢 D: Ory Hydra + Kratos

- 概要: 機能分離型、マイクロサービス志向
- メリット: Cloud Native、スケーラビリティ高い
- デメリット:
  - IdP 全機能を揃えるには Hydra + Kratos + Keto + Oathkeeper の統合が必要
  - 運用コンポーネント数が増える、2 名チームで破綻

### 選択肢 E: 商用 IdP（Okta、Auth0、PingIdentity 等）

- 概要: SaaS または商用ソフトウェア
- メリット: 機能成熟、サポート充実
- デメリット:
  - オンプレ制約（NFR-F-SYS-001）で SaaS は選択肢外
  - 商用ライセンス費用（年間数千万円規模）がコスト削減目標と矛盾

## 帰結

### ポジティブな帰結

- JTC 既存 AD との SSO 実現、従業員は単一 ID で業務システム + k1s0 プラットフォーム利用
- テナント分離が Realm で実現、BC-ONB-003 の自動プロビジョニングと統合可能
- OIDC/SAML で将来の外部 IdP 連携が容易
- SPI 拡張で JTC 固有要件（例: マイナンバー連携）に対応可能
- Red Hat SSO として商用サポート取得可能（保険）

### ネガティブな帰結

- JVM メモリ使用量（初期 2GB/Pod 想定）、Pod スケール時のコスト増
- Keycloak 管理コンソールの複雑さで運用工数が増える（Runbook 必須）
- バージョンアップ時の DB スキーマ migration で停止時間発生、保守ウィンドウ設計必要
- JWT 署名鍵ローテーション時の既発行トークン失効戦略が運用設計として必要

## 実装タスク

- Keycloak 26+ の Helm Chart バージョン固定、Argo CD 管理
- PostgreSQL バックエンドの設定（ADR-DATA-001 と統合）
- Realm テンプレート（テナント用、管理者用）を Backstage Software Template 化
- LDAP/AD 連携設定を Vault/OpenBao 経由でシークレット管理
- JWT 署名鍵のローテーション手順を Runbook 化、四半期訓練
- 監査ログを Loki に集約、異常検知アラート設定

## 参考文献

- Keycloak 公式: keycloak.org
- Red Hat SSO（商用サポート）
- OpenID Connect 仕様: openid.net/connect
- OAuth 2.1 ドラフト: IETF
- JTC 既存 AD Kerberos 運用規程
