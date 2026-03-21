# マルチテナント設計

k1s0 におけるマルチテナント分離の方針と実装ガイドラインを定義する。

## テナント分離戦略

### 選定方針

k1s0 では **行レベルセキュリティ（RLS）+ tenant_id カラム** を基本戦略とする。

| 戦略 | 分離度 | 運用コスト | 採用 |
| --- | --- | --- | --- |
| 行レベルセキュリティ（RLS） | 中 | 低 | **採用** |
| スキーマ分離 | 高 | 中 | 将来検討 |
| データベース分離 | 最高 | 高 | 不採用 |

### 行レベルセキュリティを選定した理由

- サービス数が多いモノリポ構成では、DB インスタンスやスキーマの増殖がコスト・運用負荷を招く
- RLS であればマイグレーションが単一で済み、CI/CD パイプラインの複雑化を防げる
- テナント数の増加に対してスケーラブル（DDL 変更不要）

### RLS 適用パターン

```sql
-- テナント分離用のポリシーを定義する
ALTER TABLE {schema}.{table} ENABLE ROW LEVEL SECURITY;

-- テナントごとのデータアクセスを制限するポリシー
CREATE POLICY tenant_isolation ON {schema}.{table}
    USING (tenant_id = current_setting('app.current_tenant_id')::TEXT);
```

## tenant_id の伝搬方法

### JWT からデータベースまでのフロー

```
1. クライアント → Kong Gateway（JWT 検証）
2. Kong → バックエンドサービス（X-Tenant-Id ヘッダー付与）
3. サービス層 → コンテキストに tenant_id を設定
4. リポジトリ層 → DB セッション変数に tenant_id を設定
5. PostgreSQL → RLS ポリシーが tenant_id でフィルタリング
```

### サービス層での設定

各リクエストの先頭で DB セッション変数にテナント ID を設定する。

```sql
-- リクエストスコープでテナント ID を設定する
SET LOCAL app.current_tenant_id = '{tenant_id}';
```

### gRPC メタデータでの伝搬

サービス間通信では gRPC メタデータ `x-tenant-id` でテナント ID を伝搬する。
Kafka メッセージでは `tenant_id` ヘッダーを使用する。

## マイグレーション時のテナント考慮事項

### テナント対応テーブルの要件

1. **tenant_id カラムの必須化**: テナントデータを格納する全テーブルに `tenant_id VARCHAR(255) NOT NULL` を追加する
2. **複合インデックス**: 主要なクエリパスに `(tenant_id, ...)` の複合インデックスを作成する
3. **RLS ポリシー**: テーブル作成時に RLS ポリシーを同時に定義する
4. **外部キー**: テナントをまたぐ外部キー参照を禁止する

### テナント非依存テーブル

以下のテーブルはテナント横断で共有するため、tenant_id を持たない。

- `auth.users` / `auth.roles` / `auth.permissions`（認証基盤）
- `config.config_entries`（システム設定）
- `featureflag.feature_flags`（フィーチャーフラグ）— システム全体のグローバル設定として設計（ADR-0012 参照）
- `api-registry-db` 全テーブル — サービスディスカバリのグローバル情報として設計（ADR-0012 参照）
- マスタデータ系テーブル

### マイグレーション時の注意点

- 新規テーブル作成時は tenant_id カラムと RLS ポリシーをセットで定義する
- 既存テーブルへの tenant_id 追加時は、既存データのバックフィル計画を事前に策定する
- べき等性ガード（`IF NOT EXISTS`）を必ず含める

## 現在の実装状態

### 実装済み

- `tenant-db`: テナントマスタテーブル（`tenant.tenants`, `tenant.tenant_members`）
- `auth-db`: api_keys テーブルの `tenant_id` カラム（012 マイグレーションで追加）
- Kong Gateway での JWT ベースのテナント識別
- **Phase 1 実装済み** (M-25 対応):
  - `order-db`: `orders`, `order_items` テーブルへの `tenant_id` 追加と RLS ポリシー設定（マイグレーション 009）
  - `payment-db`: `payments` テーブルへの `tenant_id` 追加と RLS ポリシー設定（マイグレーション 007）
  - `inventory-db`: `inventory_items`, `inventory_reservations` テーブルへの `tenant_id` 追加と RLS ポリシー設定（マイグレーション 008）
- **system 層 RLS 追加** (ADR-0012 対応):
  - `saga-db`: `saga.saga_states`, `saga.saga_step_logs` テーブルへの `tenant_id` 追加と RLS ポリシー設定（マイグレーション 008）
  - `event-store-db`: `eventstore.event_streams`, `eventstore.events`, `eventstore.snapshots` テーブルへの `tenant_id` 追加と RLS ポリシー設定（マイグレーション 006）
  - `featureflag-db`: テナント非依存（グローバル設定）として扱う — tenant_id / RLS 対象外
  - `api-registry-db`: テナント非依存（サービスディスカバリ）として扱う — tenant_id / RLS 対象外

### RLS ポリシーの動作

全テナント対応テーブルは以下のポリシーパターンで保護される:

```sql
-- リクエスト処理開始時にテナント ID をセッション変数に設定する
SET LOCAL app.current_tenant_id = '{tenant_id}';

-- RLS ポリシーがセッション変数に基づいて行を自動フィルタリングする
-- CREATE POLICY tenant_isolation ON {table}
--     USING (tenant_id = current_setting('app.current_tenant_id', true)::TEXT);
```

### 今後の方針

1. ~~**Phase 1**: service 層の主要テーブル（orders, payments, inventory）に tenant_id を追加~~ **完了**
2. ~~**Phase 2**: RLS ポリシーを全テナント対応テーブルに適用~~ **完了（Phase 1 と同時実施）**
3. **Phase 3**: テナントごとのクォータ制御と monitoring の導入
4. **Phase 4**: テナント単位のデータエクスポート・削除機能の実装

### 注意事項

- `outbox_events` テーブルはシステム内部イベントバスのため、RLS 対象外（サービスロールが全件アクセスする）
- `app.current_tenant_id` が未設定のセッションは `current_setting(..., true)` が `NULL` を返すため、RLS によって全行が非表示になる
- サービスロール（アプリケーション DB ユーザー）は `SET LOCAL` でセッション変数を設定する責務を持つ
