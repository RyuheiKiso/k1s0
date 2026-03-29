# ADR-0056: マルチテナント設計の明確化（H-012 監査対応）

## ステータス

承認済み

## コンテキスト

外部監査 H-012「tenant_id 実装の不均等」の指摘により、以下のテーブルが tenant_id を持たないことが判明した。

| DB | テーブル | tenant_id | RLS |
|----|---------|-----------|-----|
| auth | users, roles, permissions | ✗ | ✗ |
| config | config_entries | ✗ | ✗ |
| vault | secrets | ✗ | ✗ |
| notification | channels | ✗ | ✗ |
| policy | policies | ✗ | ✗ |

各テーブルについて「意図的にグローバル設計にすべきか、テナントスコープにすべきか」の設計判断を明確化する。

## 決定

### グループA: 意図的にシステムグローバル設計（tenant_id 不要）

#### auth.users / auth.roles / auth.permissions

**設計判断: システムグローバル（tenant_id 不要）**

**理由:**
- `auth.users` は Keycloak の ID Provider と連携しており、`keycloak_sub` が全テナントにまたがるグローバル識別子として機能する
- 1人のユーザーが複数テナントに所属できるマルチテナントモデルでは、ユーザーレコード自体はグローバルに存在し、テナントへの所属は`user_roles`テーブルで管理される
- `auth.roles` の `tier` フィールド（system/business/service）はシステム全体で共有されるロール定義を表す。テナントごとに異なるロール定義は想定しない
- `auth.permissions` はリソース/アクションの組み合わせ定義。これらも全テナント共通の権限カタログである

**テナント分離の実現方式:**
- ユーザーのテナント所属は `auth.user_roles` + RBAC ポリシーで管理する
- テナントIDに基づくアクセス制御は API Gateway（Kong）レベルで実施する
- Keycloak の realm 設定でテナント分離を補完する

---

### グループB: テナントスコープ実装済み（notification.channels）

#### notification.channels（H-012 + H-010 対応実装済み）

**設計判断: テナントスコープ（tenant_id 実装済み）**

**理由:**
- 通知チャンネル（SMTP 設定、Webhook URL、API キー等）はテナントごとに異なる
- 機密認証情報を含むため、テナント間の設定漏洩は重大なセキュリティリスクになる
- 各テナントが独自のメール送信設定・Webhook を持つことが自然な設計

**実装内容:**
- `migration 012`: tenant_id カラム追加 + RLS ポリシー追加
- PostgreSQL RLS: `set_config('app.current_tenant_id', ?, true)` によるトランザクションスコープのテナント分離
- デフォルト `tenant_id = 'system'` でシステム共通チャンネルを表現

**JWT クレームからのテナント伝播（実装ロードマップ）:**
- 現状: `tenant_id = 'system'` をデフォルトとして使用
- 次フェーズ: JWT の `tenant_id` クレームを抽出して auth ミドルウェアからリクエストコンテキストに伝播する
- 実装箇所: `adapter/middleware/auth.rs` の Claims 抽出 → `adapter/handler/notification_handler.rs` への伝播

---

### グループC: namespace/key_path による論理分離（暗黙的テナント分離）

#### config.config_entries

**設計判断: namespace による論理分離（現状維持、将来的な tenant_id 追加を検討）**

**理由:**
- `namespace` フィールドが論理的なテナント分離の役割を担う（例: `tenant-abc/feature-flags`）
- アプリケーション層で `namespace` フィルタリングを強制することでテナント分離を実現する
- 明示的な `tenant_id` への移行は namespace 設計の整理後に実施する

**リスク:**
- アプリケーション層のバグによる namespace フィルタリング漏れが テナント間データ漏洩につながる可能性がある
- **将来対応**: namespace を `{tenant_id}/{category}` 形式に標準化し、RLS を追加する

---

#### vault.secrets

**設計判断: key_path による論理分離（現状維持、将来的な tenant_id 追加を検討）**

**理由:**
- `key_path` フィールドが論理的なテナント分離の役割を担う（例: `/tenants/abc123/db-password`）
- Vault の設計コンセプトがパスベースのアクセス制御であり、これと一致する
- アプリケーション層で key_path プレフィックス検証を強制することでテナント分離を実現する

**リスク:**
- アプリケーション層のバグによる key_path 検証漏れがテナント間シークレット漏洩につながる可能性がある
- **将来対応**: ADR-0022 の Vault per-service role 分離と組み合わせ、key_path ベース RLS を追加する

---

#### policy.policies

**設計判断: システムグローバル設計（tenant_id 不要）**

**理由:**
- OPA/Rego ポリシーは全テナントに適用されるアクセス制御ルールを定義する
- テナントごとに異なる認可ポリシーを持つことは想定しない（ポリシーが多様化するとセキュリティモデルが複雑化する）
- テナント固有の権限差異は RBAC のロール割り当てで対処する

---

## 実装ロードマップ

### フェーズ1（完了）
- [x] notification.channels: tenant_id + RLS 実装（migration 012）
- [x] notification.channels.config: AES-256-GCM 暗号化（migration 011、C-005 対応）

### フェーズ2（次スプリント）
- [ ] notification.channels: JWT クレームからのテナント ID 伝播（`adapter/middleware/auth.rs` 更新）
- [ ] config.config_entries: namespace フォーマット標準化（`{tenant_id}/{category}`）

### フェーズ3（中期）
- [ ] config.config_entries: tenant_id カラム追加 + RLS（namespace 標準化後）
- [ ] vault.secrets: key_path 検証の強化（prefix 必須化）

## 理由

| テーブル | 判断 | 主な根拠 |
|---------|------|---------|
| auth.users | グローバル | Keycloak identity はテナント横断型 |
| auth.roles | グローバル | システム共通ロール定義 |
| auth.permissions | グローバル | 全テナント共通の権限カタログ |
| notification.channels | テナントスコープ | テナントごとに異なる通知設定・機密認証情報 |
| config.config_entries | 論理分離（namespace） | 将来 tenant_id 追加を計画 |
| vault.secrets | 論理分離（key_path） | Vault パスベース設計に準拠 |
| policy.policies | グローバル | 全テナント共通のアクセス制御ルール |

## 影響

**ポジティブな影響:**
- H-012 指摘の設計不統一が明確な設計方針として文書化される
- notification.channels に DB 層でのテナント分離が実現される
- 将来の tenant_id 追加対象テーブルと優先度が明確になる

**リスク:**
- グループC（config/vault）はアプリケーション層のバグによるテナント漏洩リスクが残る
- このリスクは既存のコードレビュー規程と CI テストで軽減する

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| 全テーブル一括 tenant_id 追加 | notification/config/vault/policy 全てに tenant_id を追加 | auth.users/roles の グローバル設計を損なう。マイグレーションリスクが高い。段階的対応の方が安全 |
| スキーマ分離 | テナントごとに独立したスキーマを作成 | 既存アーキテクチャとの乖離が大きい。マイグレーション管理が複雑化する |

## 参考

- [ADR-0034: マルチテナント設計](0034-multi-tenant-design.md)
- [ADR-0054: RLS 段階的実装戦略](0054-rls-remaining-tenant-tables.md)
- [ADR-0052: JSONB カラム暗号化戦略](0052-jsonb-column-encryption.md)

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-03-29 | 初版作成。H-012 監査対応として各テーブルの設計判断を明確化 | @team |
