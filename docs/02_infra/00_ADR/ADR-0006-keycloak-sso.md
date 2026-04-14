# ADR-0006: SSO 基盤として Keycloak を採用

- ステータス: Accepted
- 起票日: 2026-04-14
- 決定日: 2026-04-14
- 起票者: kiso ryuhei
- 関係者: インフラチーム / 起案者 / 決裁者

## コンテキスト

k1s0 では複数の構成要素が個別に認証を必要とする。
具体的には以下が挙げられる。

- tier1 / tier2 / tier3 サービス
- アプリ配信ポータル (エンドユーザー向け)
- Backstage (開発者ポータル / Phase 2)
- Argo CD (運用者向け / Phase 1b)
- Harbor (開発者・CI 向け / Phase 1c)
- Headlamp (運用者向け / Phase 1b)

これらが各自で認証を実装すると、認証系統が分裂し、ユーザー管理・監査・パスワードポリシーの一貫性が失われる。
**OIDC で統一された SSO 基盤** が必須である。

加えて、MVP-1a ではローカル DB 運用を許容しつつ、**Phase 2 以降で AD/LDAP 連携への移行** を見越した設計が必要となる。

## 決定

**Keycloak を SSO 基盤として採用** する。
具体的な構成は以下のとおり。

- **Realm**: `k1s0` 単一 Realm。
- **MVP-1a**: ローカル DB (CloudNativePG) ベースのユーザーストア。Keycloak HA 構成 (Primary + Replica 1)。
- **Phase 2**: AD / LDAP 連携を追加。Keycloak の Federation 機能で既存企業 AD と統合する。
- **Envoy Gateway 連携**: oauth2-proxy 経由で `ext_authz` として Keycloak と連携する。
- **バックエンド**: PostgreSQL (CloudNativePG 共有クラスタ上に Keycloak 用 DB を追加)。

## 検討した選択肢

### 選択肢 A: Zitadel

- 概要: モダンな OIDC プロバイダ。マルチテナント設計が標準。
- メリット: マルチテナント・API 設計がモダン。
- デメリット: 日本語情報が少ない。JTC 環境での採用例・運用ナレッジが限定的。

### 選択肢 B: Authentik

- 概要: モダンな IdP / SSO ソリューション。
- メリット: 管理 UI が洗練されている。
- デメリット: エンタープライズ実績が Keycloak に比べ少ない。長期運用での安定性に不確実性が残る。

### 選択肢 C: Dex

- 概要: OIDC フロントエンド。バックエンド IdP (LDAP / GitHub / Google 等) 必須。
- メリット: 軽量。GitOps 運用と相性が良い。
- デメリット: ユーザー DB を自前で持たない。MVP-1a のローカル運用 (AD 未連携状態) では別途ユーザーストアが必要となり、構成が複雑化する。

### 選択肢 D: 認証なし / 手動管理

- 概要: 各サービスごとに独自認証を実装する。
- メリット: 初期構築が容易。
- デメリット: スケーラビリティが皆無。Phase 2 以降の構成要素追加で破綻する。

### 選択肢 E: Keycloak (採用)

- 概要: CNCF 周辺で実績豊富な OIDC IdP。
- メリット: ユーザー DB を自前保有しているため MVP 単独で構築可能。AD 連携は後から設定追加のみで実現可能。OSS 版と商用版の機能差ゼロ。日本語情報が豊富。
- デメリット: ローカル DB 運用が必要なため、PostgreSQL HA とセットで運用する必要がある。

## 決定理由

- **MVP 単独構築可能**: ユーザー DB を自前保有するため、AD 連携が未構築の MVP-1a 段階で単独で立ち上げられる。Dex のような「外部 IdP 必須」の制約がない。
- **AD 連携への移行コストが小さい**: Phase 2 で AD 連携を追加する際、Keycloak の Federation 設定を追加するだけで実現できる。MVP のユーザー管理コードや UI を捨てる必要がない。
- **Envoy Gateway との統合経路が確立されている**: oauth2-proxy 経由で `ext_authz` 連携することで、tier1 / tier2 / tier3 の API 認証を統一的に処理できる。
- **OSS 版と商用版の機能差がゼロ**: ベンダーロックイン回避の原則と整合する。商用 SLA が必要になっても OSS 版から無停止で移行可能。
- **日本語情報・JTC 内採用例が豊富**: 運用トラブル発生時のナレッジアクセスが容易。バス係数 2 の実証フェーズで重要。

## 影響

### ポジティブな影響

- すべての構成要素 (アプリ配信ポータル / Backstage / Argo CD / Harbor / Headlamp 等) で SSO が統一される。
- AD 連携を Phase 2 で追加する際に MVP の構成を作り直す必要がない。
- ユーザー管理・監査ログ・パスワードポリシーが Keycloak で一元化される。

### ネガティブな影響 / リスク

- ローカル DB 運用には CloudNativePG (PostgreSQL HA) のセットアップが必須となる。
  - 緩和策: CloudNativePG は別途 Phase 1b で導入予定であり、Keycloak の DB は同じクラスタに同居させる。
- Keycloak 自体の高可用性 (HA) を確保する必要がある。
  - 緩和策: Phase 1b で Keycloak Pod を 2 レプリカ以上で構成し、PostgreSQL は CloudNativePG (Primary + Replica 1) で HA 化する。
- Realm / Client 設定の手作業構築は再現性が低い。
  - 緩和策: Realm エクスポート JSON を Git 管理し、Argo CD で適用する (Phase 1b 以降)。Phase 1c で SealedSecrets / OpenBao + ESO によりシークレット管理を整理する。
- ユーザー数が増えると DB 負荷が上がる可能性がある。
  - 緩和策: MVP-1a はパイロット業務のユーザー数に絞った運用とする。Phase 2 で AD 連携に切り替えれば、ユーザー DB 役割の大部分が AD に移譲される。

### 移行・対応事項

- Phase 1a: 単一インスタンス Keycloak + 単一 PostgreSQL (HA なし) を VM 1 台に同居構築する。デモ用途。
- Phase 1b: Keycloak HA + CloudNativePG (Primary + Replica 1) に拡張する。oauth2-proxy + Envoy Gateway `ext_authz` 連携を構築する。
- Phase 1c: SealedSecrets / OpenBao + ESO による Secret 管理に移行する。
- Phase 2: AD / LDAP 連携を追加する設計を別途ドキュメント化する。
- 認可ポリシー (RBAC / グループ → 権限マッピング) の設計は別途 ADR で起票する。

## 参考資料

- [`../../01_企画/04_技術選定/02_周辺OSS.md`](../../01_企画/04_技術選定/02_周辺OSS.md) — Keycloak 採用の根拠
- [`../../01_企画/02_アーキテクチャ/04_セキュリティモデル.md`](../../01_企画/02_アーキテクチャ/04_セキュリティモデル.md) — 認証・認可の全体像
- [`../../01_企画/07_ロードマップと体制/00_フェーズ計画.md`](../../01_企画/07_ロードマップと体制/00_フェーズ計画.md) — Keycloak の段階導入
- [ADR-0004](./ADR-0004-kubeadm-adoption.md) — kubeadm 採用 (Keycloak の稼働基盤)
