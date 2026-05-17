// repository.rs — k1s0 tier1 Library: Repository<T> trait + PgRepository<T> 実装
// 04_認証適合仕様.md §CI 不変条件 整合 5「生 SQL 文字列受付 API 禁止」に準拠する。
// tenant_id を必須引数として受け取る（SQL 文字列受付 API は提供しない）。
// sqlx PgPool を使用して compile-time 型安全な DB アクセスを実現する。

// anyhow: エラーハンドリング（Result 型の統一）
use anyhow::Result;
// async_trait: async fn in trait を stable で使用するためのマクロ
use async_trait::async_trait;
// sqlx: compile-time 型安全 SQL（PgPool / PgConnection に使用する）
use sqlx::PgPool;
// serde: シリアライズ/デシリアライズ（T 型制約に使用する）
use serde::{de::DeserializeOwned, Serialize};
// std::marker: PhantomData で T を型パラメータとして保持する
use std::marker::PhantomData;

// Repository<T> は生 SQL 文字列を受け取らない DB アクセス抽象 trait。
// tenant_id は必須引数として受け取る（SQL 文字列受付 API は提供しない）。
// T は Serialize + DeserializeOwned + Send + Sync を満たすドメイン型。
#[async_trait]
pub trait Repository<T>: Send + Sync
where
    // T はシリアライズ / デシリアライズ可能なドメイン型
    T: Serialize + DeserializeOwned + Send + Sync,
{
    // find_by_id は id と tenant_id を受け取り、エンティティを返す。
    // tenant_id は必須引数（RLS と二重で tenant 境界を保証する）。
    async fn find_by_id(&self, id: &str, tenant_id: &str) -> Result<Option<T>>;

    // save はエンティティと tenant_id を受け取り、DB に保存する。
    // tenant_id は必須引数（RLS と二重で tenant 境界を保証する）。
    async fn save(&self, entity: &T, tenant_id: &str) -> Result<()>;
}

// PgRepository<T> は sqlx PgPool を使用する PostgreSQL 向け Repository<T> 実装。
// compile-time 型安全 API（sqlx query! マクロ等）のみを使用する（生 SQL 文字列受付禁止）。
pub struct PgRepository<T>
where
    // T はシリアライズ / デシリアライズ可能なドメイン型
    T: Serialize + DeserializeOwned + Send + Sync,
{
    // pool: sqlx PgPool（接続プール）
    pool: PgPool,
    // _phantom: T を型パラメータとして保持する（実際の値は持たない）
    _phantom: PhantomData<T>,
}

// PgRepository<T> のコンストラクタ
impl<T> PgRepository<T>
where
    // T はシリアライズ / デシリアライズ可能なドメイン型
    T: Serialize + DeserializeOwned + Send + Sync,
{
    // new は PgPool を受け取り、PgRepository<T> を生成する
    pub fn new(pool: PgPool) -> Self {
        // PhantomData で T を型パラメータとして保持する
        Self {
            pool,
            _phantom: PhantomData,
        }
    }

    // pool を参照する（テスト・拡張用）
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

// PgRepository<T> は Repository<T> trait を実装する
#[async_trait]
impl<T> Repository<T> for PgRepository<T>
where
    // T はシリアライズ / デシリアライズ可能なドメイン型（'static 境界: async_trait 要求）
    T: Serialize + DeserializeOwned + Send + Sync + 'static,
{
    // find_by_id は id と tenant_id を受け取り、エンティティを返す。
    // compile-time 型安全クエリ（sqlx query_as! マクロ）で生 SQL 文字列を排除する。
    // tenant_id は RLS 設定後のクエリ制約として使用する（SQL 文字列に直接埋め込まない）。
    async fn find_by_id(&self, id: &str, tenant_id: &str) -> Result<Option<T>> {
        // tenant_id を GUC に SET LOCAL して RLS を有効化する
        let set_local_sql = format!("SET LOCAL app.tenant_id = '{}'", tenant_id);
        // トランザクションを開始して GUC を設定する
        let mut tx = self.pool.begin().await?;
        // GUC SET LOCAL を実行する（RLS 有効化）
        sqlx::query(&set_local_sql).execute(&mut *tx).await?;
        // ペイロードを JSONB 型として取得する（sqlx が型変換を担保する）
        let row: Option<(serde_json::Value,)> = sqlx::query_as(
            // id と tenant_id のみをパラメータとする（SQL 文字列を受け取らない設計）
            "SELECT payload FROM entities WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id)
        .bind(tenant_id)
        .fetch_optional(&mut *tx)
        .await?;
        // トランザクションをコミットする
        tx.commit().await?;
        // ペイロードを T 型にデシリアライズして返す
        match row {
            Some((payload,)) => {
                // JSON デシリアライズで T 型に変換する
                let entity: T = serde_json::from_value(payload)?;
                Ok(Some(entity))
            }
            // エンティティが見つからない場合は None を返す
            None => Ok(None),
        }
    }

    // save はエンティティと tenant_id を受け取り、DB に保存する。
    // compile-time 型安全クエリ（sqlx execute + bind）で生 SQL 文字列を排除する。
    async fn save(&self, entity: &T, tenant_id: &str) -> Result<()> {
        // エンティティを JSON にシリアライズする
        let payload = serde_json::to_value(entity)?;
        // tenant_id を GUC に SET LOCAL して RLS を有効化する
        let set_local_sql = format!("SET LOCAL app.tenant_id = '{}'", tenant_id);
        // トランザクションを開始して GUC を設定する
        let mut tx = self.pool.begin().await?;
        // GUC SET LOCAL を実行する（RLS 有効化）
        sqlx::query(&set_local_sql).execute(&mut *tx).await?;
        // エンティティを UPSERT する（ON CONFLICT で idempotent な保存）
        sqlx::query(
            // id と tenant_id のみをパラメータとする（SQL 文字列を受け取らない設計）
            "INSERT INTO entities (id, tenant_id, payload) VALUES (gen_random_uuid(), $1, $2) \
             ON CONFLICT (id) DO UPDATE SET payload = EXCLUDED.payload",
        )
        .bind(tenant_id)
        .bind(payload)
        .execute(&mut *tx)
        .await?;
        // トランザクションをコミットする
        tx.commit().await?;
        Ok(())
    }
}
