// k1s0 tier2 Repository abstraction
// 生 SQL 文字列を受け取る public API を持たない設計で tier2 の DB アクセスを封鎖する
// sqlx の query! マクロ等の compile-time 型安全 API のみを library crate 内部から呼ぶ

// テーブルクラスの再エクスポート（atomic_triple_write との整合に使用する）
pub use crate::atomic_triple_write::TableClass;
// テナントコンテキスト
use crate::tenant_context::TenantContext;
// エラーハンドリング
use anyhow::Result;
// シリアライズ
use serde::{Deserialize, Serialize};
// UUID
use uuid::Uuid;

// Repository を実行するコンテキスト（TenantContext をラップする）
// 生 SQL 文字列は受け取らない設計を型で表現する
#[derive(Debug)]
pub struct RepositoryContext {
    // テナントコンテキスト（GUC 注入に使用する）
    tenant_context: TenantContext,
}

impl RepositoryContext {
    // RepositoryContext を生成する
    // tenant_id は API 引数として渡せない（TenantContext 経由のみ）
    pub fn new(tenant_context: TenantContext) -> Self {
        Self { tenant_context }
    }

    // テナントコンテキストを参照する
    pub fn tenant_context(&self) -> &TenantContext {
        &self.tenant_context
    }

    // SET LOCAL SQL を取得する（Repository 実装が DB に注入するために使用する）
    pub fn set_local_sql(&self) -> String {
        // 4 GUC の SET LOCAL SQL を返す
        self.tenant_context.to_set_local_sql()
    }

    // 公開 API に生 SQL 文字列受付メソッドを持たないことを型で保証する
    // このコメントは: 「生 SQL を渡す API は存在しないため、呼び出し側は提供できない」という設計上の宣言
}

// select 操作の結果型（テナントスコープ内のエンティティを表す）
// PII 列は含まない（pii_segregated table は別の型で管理する）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantScopedEntity {
    // エンティティの主キー
    pub id: Uuid,
    // テナント ID（read-only、RLS が保証する）
    pub tenant_id: Uuid,
    // エンティティバージョン（楽観的ロックに使用する）
    pub version: i64,
    // エンティティペイロード（業界中立的な汎用 JSON）
    pub payload: serde_json::Value,
}

// SELECT 結果の tenant_id が TenantContext と一致することを検証するヘルパー
// RLS が物理的に保証するが、アプリ層でも二重検証する
pub fn verify_select_result(
    entity: &TenantScopedEntity,
    ctx: &RepositoryContext,
) -> Result<()> {
    // RLS が保証するはずだが、アプリ層でも tenant_id を照合する
    let expected = ctx.tenant_context.tenant_id();
    if entity.tenant_id != expected {
        // RLS bypass が発生した場合は即座にエラーで止める
        anyhow::bail!(
            "RLS bypass detected: entity.tenant_id={} != context.tenant_id={}",
            entity.tenant_id,
            expected
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    // テストモジュール内部でのみ使用するインポート
    use super::*;
    use crate::tenant_context::SessionPurpose;

    #[test]
    // RepositoryContext の set_local_sql が GUC を含むことを確認する
    fn test_repository_context_set_local_sql() {
        // TenantContext を生成する
        let tenant_id = Uuid::new_v4();
        let ctx = TenantContext::from_auth(
            tenant_id,
            "repo-actor".to_string(),
            SessionPurpose::Export,
        );
        // RepositoryContext を生成する
        let repo_ctx = RepositoryContext::new(ctx);
        // SET LOCAL SQL を取得する
        let sql = repo_ctx.set_local_sql();
        // 全 GUC が含まれることを確認する
        assert!(sql.contains("app.tenant_id"));
        assert!(sql.contains("app.actor_id"));
        assert!(sql.contains("export"));
    }

    #[test]
    // RLS bypass 検証が正常な場合は OK を返すことを確認する
    fn test_verify_select_result_ok() {
        // 同一の tenant_id でエンティティとコンテキストを生成する
        let tenant_id = Uuid::new_v4();
        let ctx_inner = TenantContext::from_auth(
            tenant_id,
            "actor".to_string(),
            SessionPurpose::BusinessOp,
        );
        let ctx = RepositoryContext::new(ctx_inner);
        // 同一 tenant_id のエンティティを生成する
        let entity = TenantScopedEntity {
            id: Uuid::new_v4(),
            tenant_id,
            version: 1,
            payload: serde_json::json!({}),
        };
        // 検証が OK を返すことを確認する
        assert!(verify_select_result(&entity, &ctx).is_ok());
    }

    #[test]
    // RLS bypass 検証が不一致の場合はエラーを返すことを確認する
    fn test_verify_select_result_bypass_error() {
        // 異なる tenant_id でエンティティとコンテキストを生成する
        let ctx_inner = TenantContext::from_auth(
            Uuid::new_v4(),
            "actor".to_string(),
            SessionPurpose::BusinessOp,
        );
        let ctx = RepositoryContext::new(ctx_inner);
        // 別 tenant の entity（RLS bypass を模擬する）
        let entity = TenantScopedEntity {
            id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            version: 1,
            payload: serde_json::json!({}),
        };
        // RLS bypass 検出でエラーを返すことを確認する
        assert!(verify_select_result(&entity, &ctx).is_err());
    }
}
