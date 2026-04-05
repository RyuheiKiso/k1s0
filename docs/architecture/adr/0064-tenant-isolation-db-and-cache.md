# ADR-0064: マルチテナント分離の DB・キャッシュ実装戦略

## ステータス

承認済み

## コンテキスト

外部技術監査（2026-04-01）の STATIC-CRITICAL-001 指摘: config / featureflag / ratelimit サービスにおいて、テナント間のデータ分離が DB・キャッシュ・Redis の各レベルで未実装だった。

具体的には以下の問題が存在していた:

- `config_entries` / `feature_flags` テーブルに `tenant_id` カラムが存在しない
- リポジトリトレイトのメソッドシグネチャにテナントID引数がない
- キャッシュキーが `{namespace}:{key}` 形式のため、異なるテナントが同一キャッシュエントリを参照できる
- ratelimit の Redis キーが `ratelimit:{scope}:{identifier}` 形式のため、テナント間でレートリミット状態を共有してしまう
- RBAC ミドルウェアで `tenant_id: String::new()` がハードコードされており、JWT クレームからのテナントID抽出が未実装

## 決定

### 1. DB レベル: `tenant_id UUID NOT NULL` カラムの追加

- `config_entries`、`config_change_logs`、`feature_flags`、`flag_audit_logs` テーブルに `tenant_id UUID NOT NULL` を追加する
- `(tenant_id, flag_key)` や `(tenant_id, namespace, key)` の複合 UNIQUE 制約に変更する
- 全クエリに `WHERE tenant_id = $X` を必須とする

### 2. リポジトリトレイト: `tenant_id: Uuid` を第1引数に

全リポジトリトレイトメソッドの第1引数を `tenant_id: Uuid` とする。

```rust
async fn find_by_key(&self, tenant_id: Uuid, flag_key: &str) -> anyhow::Result<FeatureFlag>;
async fn find_all(&self, tenant_id: Uuid) -> anyhow::Result<Vec<FeatureFlag>>;
```

### 3. キャッシュキーのテナントスコープ化

キャッシュキー形式を `{tenant_id}:{namespace}:{key}` / `{tenant_id}:{flag_key}` に変更し、テナント間のキャッシュ汚染を防ぐ。

### 4. ratelimit Redis キーのテナントスコープ化

Redis キー形式を `ratelimit:{tenant_id}:{scope}:{identifier}` に変更する。ルール定義（PostgreSQL）はシステムレベルでテナント横断のまま維持する。

### 5. JWT クレームからのテナントID抽出

ハンドラー層で `Option<Extension<k1s0_auth::Claims>>` を受け取り、`Claims.tenant_id` を UUID としてパースする。パース失敗・クレームなしの場合はシステムテナント UUID `00000000-0000-0000-0000-000000000001` をフォールバックとして使用する。

```rust
fn extract_tenant_id(claims: &Option<Extension<k1s0_auth::Claims>>) -> Uuid {
    claims
        .as_ref()
        .and_then(|ext| Uuid::parse_str(&ext.0.tenant_id).ok())
        .unwrap_or_else(|| Uuid::parse_str(SYSTEM_TENANT_ID).expect("valid"))
}
```

## 理由

- **Row Level Security (RLS) より先にアプリ層で実施**: RLS は将来的な追加防衛層として有効だが、まずアプリ層で確実に保証する
- **リポジトリトレイトへの `tenant_id` 追加**: コンパイル時にテナントID漏れを検出できる
- **キャッシュキーのプレフィックス化**: moka・Redis の両方で同一パターンを適用し、一貫性を保つ
- **フォールバックの明示化**: システムコンポーネント（内部呼び出し）は JWT なしでシステムテナントとして動作するため、`Option<Extension>` で受け取り明示的にフォールバックする

## 影響

**ポジティブな影響**:

- テナント間のデータ漏洩リスクを排除
- キャッシュ・Redis のテナント汚染を排除
- コンパイル時にテナントスコープ漏れを検出できる

**ネガティブな影響・トレードオフ**:

- 既存データのマイグレーション時にシステムテナント UUID を `tenant_id` として埋める必要がある
- ratelimit の Redis キー形式変更により、既存の Redis 状態が新キーとマッチしなくなる（キーは TTL で自然消滅するため許容範囲）

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| PostgreSQL RLS のみ | RLS でテナント境界を強制 | アプリ層の `tenant_id` なしでも動作してしまうため多層防御として不十分 |
| マルチテナント用スキーマ分離 | テナントごとに別スキーマを作成 | テナント数が動的に増加する場合のスキーマ管理コストが高い |
| Redis キーを変更しない | ratelimit のみキースコープ化しない | テナントAがテナントBのレートを消費できるため脆弱性リスクが残る |

## 参考

- [featureflag database.md](../../servers/system/featureflag/database.md)
- [featureflag implementation.md](../../servers/system/featureflag/implementation.md)
- [ratelimit implementation.md](../../servers/system/ratelimit/implementation.md)
- [config implementation.md](../../servers/system/config/implementation.md)
- 外部技術監査報告書 2026-04-01: STATIC-CRITICAL-001

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-04-01 | 初版作成（外部監査 STATIC-CRITICAL-001 対応） | @kiso ryuhei |
