//! PostgreSQLリポジトリ統合テスト
//! 実行には PostgreSQL + saga スキーマが必要:
//!   DATABASE_URL="postgres://..." cargo test -- --ignored
//!
//! テスト対象: SagaPostgresRepository の CRUD 操作とトランザクション整合性。
//! saga スキーマ: infra/docker/init-db/04-saga-schema.sql

#[cfg(test)]
mod tests {
    #[tokio::test]
    #[ignore = "requires PostgreSQL with saga schema (infra/docker/init-db/04-saga-schema.sql)"]
    async fn test_create_and_find_saga() {
        // 1. DATABASE_URL から PgPool を作成
        // 2. SagaPostgresRepository::new(pool)
        // 3. SagaState::new(...) で saga を作成
        // 4. repo.create(&state) → repo.find_by_id(saga_id)
        // 5. フィールド (workflow_name, status, payload) を検証
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL with saga schema"]
    async fn test_update_with_step_log_atomicity() {
        // saga_states と saga_step_logs が原子的に更新されることを検証
        // 1. saga を作成
        // 2. update_with_step_log で状態更新 + ステップログ追加
        // 3. find_by_id + find_step_logs で両方反映されていることを確認
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL with saga schema"]
    async fn test_find_incomplete() {
        // Started/Running/Compensating 状態の saga のみ返されることを検証
        // 1. 各ステータスの saga を作成 (Started, Running, Completed, Failed, Compensating)
        // 2. find_incomplete() で Started, Running, Compensating のみ返される
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL with saga schema"]
    async fn test_list_with_filters() {
        // workflow_name, status, correlation_id フィルタとページネーションの検証
        // 1. 異なる workflow_name / status / correlation_id の saga を複数作成
        // 2. SagaListParams の各フィルタで正しく絞り込まれることを確認
        // 3. page / page_size によるページネーションを検証
    }
}
