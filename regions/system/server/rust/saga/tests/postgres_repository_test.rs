//! PostgreSQLリポジトリ統合テスト
//! 実行には PostgreSQL + saga スキーマが必要:
//!   DATABASE_URL="postgres://..." cargo test -- --ignored
//!
//! テスト対象: SagaPostgresRepository の CRUD 操作とトランザクション整合性。
//! test-utils フィーチャーが有効な場合のみコンパイルする（L-07 監査対応）
#![cfg(feature = "test-utils")]
//! saga スキーマ: infra/docker/init-db/04-saga-schema.sql
//!
//! このファイルでは InMemorySagaRepository を使ったユニットテストで
//! リポジトリの振る舞いを検証し、PostgreSQL固有の統合テストは #[ignore] で残す。
#![allow(clippy::unwrap_used)]

#[cfg(test)]
mod tests {
    use k1s0_saga_server::domain::entity::saga_state::{SagaState, SagaStatus};
    use k1s0_saga_server::domain::entity::saga_step_log::SagaStepLog;
    use k1s0_saga_server::domain::repository::saga_repository::{SagaListParams, SagaRepository};
    use k1s0_saga_server::test_support::InMemorySagaRepository;

    /// テスト用のSagaStateを作成するヘルパー
    fn make_saga(workflow_name: &str) -> SagaState {
        SagaState::new(
            workflow_name.to_string(),
            serde_json::json!({"task_id": "TASK-001"}),
            Some("corr-001".to_string()),
            Some("user-1".to_string()),
        )
    }

    /// テスト用のSagaStateを指定ステータスで作成するヘルパー
    fn make_saga_with_status(workflow_name: &str, status: SagaStatus) -> SagaState {
        let mut saga = make_saga(workflow_name);
        saga.status = status;
        saga
    }

    // =========================================================================
    // ユニットテスト（InMemorySagaRepository使用、外部サービス不要）
    // =========================================================================

    /// create → find_by_id でSagaの作成と取得を検証する
    #[tokio::test]
    async fn test_create_and_find_saga() {
        let repo = InMemorySagaRepository::new();
        let saga = make_saga("task-assignment");
        let saga_id = saga.saga_id;

        // Saga を作成する
        repo.create(&saga).await.unwrap();

        // 作成したSagaをIDで取得する
        let found = repo.find_by_id(saga_id).await.unwrap();
        assert!(found.is_some(), "作成したSagaが見つかること");

        let found = found.unwrap();
        assert_eq!(found.saga_id, saga_id);
        assert_eq!(found.workflow_name, "task-assignment");
        assert_eq!(found.status, SagaStatus::Started);
        assert_eq!(found.current_step, 0);
        assert_eq!(found.correlation_id.as_deref(), Some("corr-001"));
        assert_eq!(found.initiated_by.as_deref(), Some("user-1"));
        assert!(found.error_message.is_none());
    }

    /// 存在しないIDの検索がNoneを返すことを検証する
    #[tokio::test]
    async fn test_find_by_id_not_found() {
        let repo = InMemorySagaRepository::new();
        let result = repo.find_by_id(uuid::Uuid::new_v4()).await.unwrap();
        assert!(result.is_none(), "存在しないIDはNoneを返す");
    }

    /// update_status でステータスとエラーメッセージの更新を検証する
    #[tokio::test]
    async fn test_update_status() {
        let repo = InMemorySagaRepository::new();
        let saga = make_saga("test-workflow");
        let saga_id = saga.saga_id;
        repo.create(&saga).await.unwrap();

        // ステータスを RUNNING に更新する
        repo.update_status(saga_id, &SagaStatus::Running, None)
            .await
            .unwrap();

        let updated = repo.find_by_id(saga_id).await.unwrap().unwrap();
        assert_eq!(updated.status, SagaStatus::Running);
        assert!(updated.error_message.is_none());

        // ステータスを FAILED に更新し、エラーメッセージを設定する
        repo.update_status(
            saga_id,
            &SagaStatus::Failed,
            Some("step failed: connection refused".to_string()),
        )
        .await
        .unwrap();

        let failed = repo.find_by_id(saga_id).await.unwrap().unwrap();
        assert_eq!(failed.status, SagaStatus::Failed);
        assert_eq!(
            failed.error_message.as_deref(),
            Some("step failed: connection refused")
        );
    }

    /// update_with_step_log でSaga状態とステップログが原子的に更新されることを検証する
    #[tokio::test]
    async fn test_update_with_step_log_atomicity() {
        let repo = InMemorySagaRepository::new();
        let mut saga = make_saga("task-assignment");
        let saga_id = saga.saga_id;
        repo.create(&saga).await.unwrap();

        // ステップ0を実行完了としてステップログを作成する
        let mut step_log = SagaStepLog::new_execute(
            saga_id,
            0,
            "create-task".to_string(),
            Some(serde_json::json!({"task_id": "TASK-001"})),
        );
        step_log.mark_success(Some(serde_json::json!({"reserved": true})));

        // SagaState を次のステップに進める
        saga.advance_step();

        // 状態とログを同時に更新する
        repo.update_with_step_log(&saga, &step_log).await.unwrap();

        // Saga状態が更新されていることを検証する
        let updated = repo.find_by_id(saga_id).await.unwrap().unwrap();
        assert_eq!(updated.current_step, 1);
        assert_eq!(updated.status, SagaStatus::Running);

        // ステップログが記録されていることを検証する
        let logs = repo.find_step_logs(saga_id).await.unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].step_name, "create-task");
        assert_eq!(
            logs[0].status,
            k1s0_saga_server::domain::entity::saga_step_log::StepStatus::Success
        );
    }

    /// 複数ステップのログが正しく記録されることを検証する
    #[tokio::test]
    async fn test_multiple_step_logs() {
        let repo = InMemorySagaRepository::new();
        let mut saga = make_saga("task-assignment");
        let saga_id = saga.saga_id;
        repo.create(&saga).await.unwrap();

        // ステップ0: 成功
        let mut log0 = SagaStepLog::new_execute(saga_id, 0, "create-task".to_string(), None);
        log0.mark_success(Some(serde_json::json!({"ok": true})));
        saga.advance_step();
        repo.update_with_step_log(&saga, &log0).await.unwrap();

        // ステップ1: 失敗
        let mut log1 =
            SagaStepLog::new_execute(saga_id, 1, "increment-board-column".to_string(), None);
        log1.mark_failed("board column increment failed".to_string());
        saga.start_compensation("step 'increment-board-column' failed".to_string());
        repo.update_with_step_log(&saga, &log1).await.unwrap();

        // 全ステップログが記録されていることを検証する
        let logs = repo.find_step_logs(saga_id).await.unwrap();
        assert_eq!(logs.len(), 2, "2件のステップログが記録される");
        assert_eq!(logs[0].step_name, "create-task");
        assert_eq!(logs[1].step_name, "increment-board-column");
        assert_eq!(
            logs[1].status,
            k1s0_saga_server::domain::entity::saga_step_log::StepStatus::Failed
        );
        assert_eq!(
            logs[1].error_message.as_deref(),
            Some("board column increment failed")
        );
    }

    /// find_incomplete が Started/Running/Compensating のみ返すことを検証する
    #[tokio::test]
    async fn test_find_incomplete() {
        let repo = InMemorySagaRepository::new();

        // 各ステータスのSagaを作成する
        let saga_started = make_saga_with_status("wf-a", SagaStatus::Started);
        let saga_running = make_saga_with_status("wf-b", SagaStatus::Running);
        let saga_completed = make_saga_with_status("wf-c", SagaStatus::Completed);
        let saga_failed = make_saga_with_status("wf-d", SagaStatus::Failed);
        let saga_compensating = make_saga_with_status("wf-e", SagaStatus::Compensating);
        let saga_cancelled = make_saga_with_status("wf-f", SagaStatus::Cancelled);

        // 全て作成する
        for saga in [
            &saga_started,
            &saga_running,
            &saga_completed,
            &saga_failed,
            &saga_compensating,
            &saga_cancelled,
        ] {
            repo.create(saga).await.unwrap();
        }

        // 未完了Sagaのみ返されることを検証する
        let incomplete = repo.find_incomplete().await.unwrap();
        assert_eq!(incomplete.len(), 3, "Started, Running, Compensating の3件");

        // 返されたSagaのステータスを検証する
        let statuses: Vec<SagaStatus> = incomplete.iter().map(|s| s.status.clone()).collect();
        assert!(statuses.contains(&SagaStatus::Started));
        assert!(statuses.contains(&SagaStatus::Running));
        assert!(statuses.contains(&SagaStatus::Compensating));
        assert!(!statuses.contains(&SagaStatus::Completed));
        assert!(!statuses.contains(&SagaStatus::Failed));
        assert!(!statuses.contains(&SagaStatus::Cancelled));
    }

    /// list でフィルタなしの一覧取得を検証する
    #[tokio::test]
    async fn test_list_no_filter() {
        let repo = InMemorySagaRepository::new();

        // 3件のSagaを作成する
        for i in 0..3 {
            let saga = make_saga(&format!("workflow-{}", i));
            repo.create(&saga).await.unwrap();
        }

        // フィルタなし、ページサイズ10で取得する
        let params = SagaListParams {
            page: 1,
            page_size: 10,
            ..Default::default()
        };
        let (sagas, total) = repo.list(&params).await.unwrap();
        assert_eq!(total, 3);
        assert_eq!(sagas.len(), 3);
    }

    /// list の workflow_name フィルタを検証する
    #[tokio::test]
    async fn test_list_filter_by_workflow_name() {
        let repo = InMemorySagaRepository::new();

        // 異なるワークフロー名のSagaを作成する
        repo.create(&make_saga("task-assignment")).await.unwrap();
        repo.create(&make_saga("task-assignment")).await.unwrap();
        repo.create(&make_saga("board-update")).await.unwrap();

        // workflow_name フィルタで絞り込む
        let params = SagaListParams {
            workflow_name: Some("task-assignment".to_string()),
            page: 1,
            page_size: 10,
            ..Default::default()
        };
        let (sagas, total) = repo.list(&params).await.unwrap();
        assert_eq!(total, 2, "task-assignment は2件");
        assert_eq!(sagas.len(), 2);
        assert!(sagas.iter().all(|s| s.workflow_name == "task-assignment"));
    }

    /// list の status フィルタを検証する
    #[tokio::test]
    async fn test_list_filter_by_status() {
        let repo = InMemorySagaRepository::new();

        repo.create(&make_saga_with_status("wf-a", SagaStatus::Started))
            .await
            .unwrap();
        repo.create(&make_saga_with_status("wf-b", SagaStatus::Completed))
            .await
            .unwrap();
        repo.create(&make_saga_with_status("wf-c", SagaStatus::Completed))
            .await
            .unwrap();

        // COMPLETED でフィルタする
        let params = SagaListParams {
            status: Some(SagaStatus::Completed),
            page: 1,
            page_size: 10,
            ..Default::default()
        };
        let (sagas, total) = repo.list(&params).await.unwrap();
        assert_eq!(total, 2, "COMPLETED は2件");
        assert!(sagas.iter().all(|s| s.status == SagaStatus::Completed));
    }

    /// list の correlation_id フィルタを検証する
    #[tokio::test]
    async fn test_list_filter_by_correlation_id() {
        let repo = InMemorySagaRepository::new();

        // corr-001 のSagaを2件、corr-002 のSagaを1件作成する
        let saga1 = make_saga("wf-a"); // corr-001
        let saga2 = make_saga("wf-b"); // corr-001
        let mut saga3 = make_saga("wf-c");
        saga3.correlation_id = Some("corr-002".to_string());

        repo.create(&saga1).await.unwrap();
        repo.create(&saga2).await.unwrap();
        repo.create(&saga3).await.unwrap();

        // correlation_id でフィルタする
        let params = SagaListParams {
            correlation_id: Some("corr-002".to_string()),
            page: 1,
            page_size: 10,
            ..Default::default()
        };
        let (sagas, total) = repo.list(&params).await.unwrap();
        assert_eq!(total, 1, "corr-002 は1件");
        assert_eq!(sagas[0].correlation_id.as_deref(), Some("corr-002"));
    }

    /// list のページネーションを検証する
    #[tokio::test]
    async fn test_list_pagination() {
        let repo = InMemorySagaRepository::new();

        // 5件のSagaを作成する
        for i in 0..5 {
            let saga = make_saga(&format!("workflow-{}", i));
            repo.create(&saga).await.unwrap();
        }

        // ページ1: 2件取得する
        let params = SagaListParams {
            page: 1,
            page_size: 2,
            ..Default::default()
        };
        let (page1, total) = repo.list(&params).await.unwrap();
        assert_eq!(total, 5, "合計は5件");
        assert_eq!(page1.len(), 2, "ページ1は2件");

        // ページ2: 2件取得する
        let params = SagaListParams {
            page: 2,
            page_size: 2,
            ..Default::default()
        };
        let (page2, _) = repo.list(&params).await.unwrap();
        assert_eq!(page2.len(), 2, "ページ2は2件");

        // ページ3: 残り1件を取得する
        let params = SagaListParams {
            page: 3,
            page_size: 2,
            ..Default::default()
        };
        let (page3, _) = repo.list(&params).await.unwrap();
        assert_eq!(page3.len(), 1, "ページ3は1件");

        // ページ1とページ2で重複がないことを検証する
        let page1_ids: Vec<_> = page1.iter().map(|s| s.saga_id).collect();
        let page2_ids: Vec<_> = page2.iter().map(|s| s.saga_id).collect();
        for id in &page1_ids {
            assert!(!page2_ids.contains(id), "ページ間でIDが重複しない");
        }
    }

    /// find_step_logs が他のSagaのログを返さないことを検証する
    #[tokio::test]
    async fn test_find_step_logs_isolation() {
        let repo = InMemorySagaRepository::new();

        // 2つのSagaを作成する
        let mut saga_a = make_saga("wf-a");
        let mut saga_b = make_saga("wf-b");
        let id_a = saga_a.saga_id;
        let id_b = saga_b.saga_id;
        repo.create(&saga_a).await.unwrap();
        repo.create(&saga_b).await.unwrap();

        // Saga Aにステップログを追加する
        let mut log_a = SagaStepLog::new_execute(id_a, 0, "step-a".to_string(), None);
        log_a.mark_success(None);
        saga_a.advance_step();
        repo.update_with_step_log(&saga_a, &log_a).await.unwrap();

        // Saga Bにステップログを追加する
        let mut log_b = SagaStepLog::new_execute(id_b, 0, "step-b".to_string(), None);
        log_b.mark_success(None);
        saga_b.advance_step();
        repo.update_with_step_log(&saga_b, &log_b).await.unwrap();

        // Saga Aのログのみ取得されることを検証する
        let logs_a = repo.find_step_logs(id_a).await.unwrap();
        assert_eq!(logs_a.len(), 1);
        assert_eq!(logs_a[0].step_name, "step-a");

        // Saga Bのログのみ取得されることを検証する
        let logs_b = repo.find_step_logs(id_b).await.unwrap();
        assert_eq!(logs_b.len(), 1);
        assert_eq!(logs_b[0].step_name, "step-b");
    }

    // =========================================================================
    // keyset ページネーションのテスト（InMemorySagaRepository使用）
    // =========================================================================

    /// cursor ページネーションで連続したページが重複なく取得できることを検証する
    #[tokio::test]
    async fn test_list_cursor_first_page() {
        let repo = InMemorySagaRepository::new();

        // 5件のSagaを時刻が異なるように作成する
        let mut all_sagas = Vec::new();
        for i in 0..5 {
            let saga = make_saga(&format!("workflow-{}", i));
            repo.create(&saga).await.unwrap();
            all_sagas.push(saga);
            // created_at を確実に異なる値にする
            tokio::time::sleep(std::time::Duration::from_millis(2)).await;
        }

        // keyset ページネーションは DESC 順で取得するため、
        // 最新（後で作成）のものが先に返る。
        // 全5件を created_at DESC, id DESC でソートして期待値を作成する
        all_sagas.sort_by(|a, b| {
            b.created_at
                .cmp(&a.created_at)
                .then(b.saga_id.cmp(&a.saga_id))
        });

        // 最新レコードの cursor を使って最初の keyset ページを取得する（全件より1件新しい仮の cursor）
        // 最初のページは cursor なしで取得するのが自然な使い方
        let params_page1 = SagaListParams {
            page: 1,
            page_size: 2,
            ..Default::default()
        };
        let (page1, total) = repo.list(&params_page1).await.unwrap();
        assert_eq!(total, 5, "合計は5件");
        assert_eq!(page1.len(), 2, "最初のページは2件");

        // 2ページ目: OFFSET で page=2 として取得する
        let params_page2 = SagaListParams {
            page: 2,
            page_size: 2,
            ..Default::default()
        };
        let (page2, _) = repo.list(&params_page2).await.unwrap();
        assert_eq!(page2.len(), 2, "2ページ目は2件");

        // 3ページ目: OFFSET で page=3 として取得する
        let params_page3 = SagaListParams {
            page: 3,
            page_size: 2,
            ..Default::default()
        };
        let (page3, _) = repo.list(&params_page3).await.unwrap();
        assert_eq!(page3.len(), 1, "3ページ目は1件");

        // 全ページ間でIDが重複しないことを検証する
        let page1_ids: Vec<_> = page1.iter().map(|s| s.saga_id).collect();
        let page2_ids: Vec<_> = page2.iter().map(|s| s.saga_id).collect();
        let page3_ids: Vec<_> = page3.iter().map(|s| s.saga_id).collect();

        for id in &page1_ids {
            assert!(!page2_ids.contains(id), "ページ1とページ2でIDが重複しない");
            assert!(!page3_ids.contains(id), "ページ1とページ3でIDが重複しない");
        }
        for id in &page2_ids {
            assert!(!page3_ids.contains(id), "ページ2とページ3でIDが重複しない");
        }

        // keyset ページネーション専用テスト:
        // all_sagas の最新レコードから cursor を作成し、keyset で取得する
        let newest = &all_sagas[0]; // DESC ソート済みの先頭
        // newest より1ms新しい仮の cursor を作成（実際の keyset 最初のページ相当）
        let fake_ts_ms = newest.created_at.timestamp_millis() + 1;
        let fake_cursor = format!("{}_{}", fake_ts_ms, uuid::Uuid::max());

        let params_keyset = SagaListParams {
            page: 1,
            page_size: 3,
            cursor: Some(fake_cursor),
            ..Default::default()
        };
        let (keyset_page, _) = repo.list(&params_keyset).await.unwrap();
        // newest より古いレコードが全件返る（page_size=3 なので最大3件）
        assert_eq!(keyset_page.len(), 3, "keyset: newest 以前のレコードが3件返る");

        // 次の cursor でさらに取得する
        let last = keyset_page.last().unwrap();
        let next_cursor = format!("{}_{}", last.created_at.timestamp_millis(), last.saga_id);

        let params_keyset2 = SagaListParams {
            page: 1,
            page_size: 3,
            cursor: Some(next_cursor),
            ..Default::default()
        };
        let (keyset_page2, _) = repo.list(&params_keyset2).await.unwrap();
        assert_eq!(keyset_page2.len(), 2, "keyset 2ページ目: 残り2件");

        // 2ページ間でIDが重複しないことを検証する
        let keyset1_ids: Vec<_> = keyset_page.iter().map(|s| s.saga_id).collect();
        let keyset2_ids: Vec<_> = keyset_page2.iter().map(|s| s.saga_id).collect();
        for id in &keyset1_ids {
            assert!(
                !keyset2_ids.contains(id),
                "keyset ページ間でIDが重複しない"
            );
        }
    }

    /// cursor ページネーションで最終ページが正しく取得できることを検証する
    #[tokio::test]
    async fn test_list_cursor_last_page() {
        let repo = InMemorySagaRepository::new();

        // 3件のSagaを作成する
        for i in 0..3 {
            let saga = make_saga(&format!("workflow-{}", i));
            repo.create(&saga).await.unwrap();
            tokio::time::sleep(std::time::Duration::from_millis(2)).await;
        }

        // 最初の2件を OFFSET で取得する
        let params = SagaListParams {
            page: 1,
            page_size: 2,
            ..Default::default()
        };
        let (first_page, _) = repo.list(&params).await.unwrap();
        assert_eq!(first_page.len(), 2);

        // cursor を使って残り1件を取得する
        let last = first_page.last().unwrap();
        let cursor = format!("{}_{}", last.created_at.timestamp_millis(), last.saga_id);

        let params_cursor = SagaListParams {
            page: 1,
            page_size: 2,
            cursor: Some(cursor),
            ..Default::default()
        };
        let (last_page, _) = repo.list(&params_cursor).await.unwrap();
        assert_eq!(last_page.len(), 1, "最後のページは1件のみ");
    }

    /// cursor ページネーションで無効な cursor を渡した場合に空リストが返ることを検証する
    #[tokio::test]
    async fn test_list_cursor_invalid_cursor_returns_empty() {
        let repo = InMemorySagaRepository::new();

        // Saga を1件作成する
        let saga = make_saga("workflow-a");
        repo.create(&saga).await.unwrap();

        // 不正な cursor 形式を渡す
        let params = SagaListParams {
            page: 1,
            page_size: 10,
            cursor: Some("invalid_cursor_format".to_string()),
            ..Default::default()
        };
        let (sagas, total) = repo.list(&params).await.unwrap();
        // total はフィルタ前の全件数を返す
        assert_eq!(total, 1);
        // 不正な cursor では空リストが返る
        assert_eq!(sagas.len(), 0, "不正な cursor では空リストが返る");
    }

    /// cursor ページネーションとフィルタの組み合わせを検証する
    #[tokio::test]
    async fn test_list_cursor_with_workflow_name_filter() {
        let repo = InMemorySagaRepository::new();

        // 異なるワークフロー名のSagaを作成する
        for i in 0..3 {
            let saga = make_saga("wf-target");
            repo.create(&saga).await.unwrap();
            // 別ワークフローも挟む
            let other = make_saga(&format!("wf-other-{}", i));
            repo.create(&other).await.unwrap();
            tokio::time::sleep(std::time::Duration::from_millis(2)).await;
        }

        // wf-target でフィルタして最初の2件を取得する
        let params = SagaListParams {
            workflow_name: Some("wf-target".to_string()),
            page: 1,
            page_size: 2,
            ..Default::default()
        };
        let (first_page, total) = repo.list(&params).await.unwrap();
        assert_eq!(total, 3, "wf-target は3件");
        assert_eq!(first_page.len(), 2);

        // cursor を使って wf-target の残り1件を取得する
        let last = first_page.last().unwrap();
        let cursor = format!("{}_{}", last.created_at.timestamp_millis(), last.saga_id);

        let params_cursor = SagaListParams {
            workflow_name: Some("wf-target".to_string()),
            page: 1,
            page_size: 2,
            cursor: Some(cursor),
            ..Default::default()
        };
        let (second_page, _) = repo.list(&params_cursor).await.unwrap();
        assert_eq!(second_page.len(), 1, "フィルタ適用後の残り1件");
        assert_eq!(
            second_page[0].workflow_name, "wf-target",
            "フィルタが正しく適用される"
        );
    }

    // =========================================================================
    // 統合テスト（PostgreSQL必要）
    // =========================================================================

    #[tokio::test]
    #[cfg(feature = "integration-tests")]
    #[ignore = "requires PostgreSQL with saga schema (infra/docker/init-db/04-saga-schema.sql)"]
    async fn test_postgres_create_and_find_saga() {
        // 1. DATABASE_URL から PgPool を作成
        // 2. SagaPostgresRepository::new(pool)
        // 3. SagaState::new(...) で saga を作成
        // 4. repo.create(&state) → repo.find_by_id(saga_id)
        // 5. フィールド (workflow_name, status, payload) を検証
    }

    #[tokio::test]
    #[cfg(feature = "integration-tests")]
    #[ignore = "requires PostgreSQL with saga schema"]
    async fn test_postgres_update_with_step_log_atomicity() {
        // saga_states と saga_step_logs が原子的に更新されることを検証
        // 1. saga を作成
        // 2. update_with_step_log で状態更新 + ステップログ追加
        // 3. find_by_id + find_step_logs で両方反映されていることを確認
    }

    #[tokio::test]
    #[cfg(feature = "integration-tests")]
    #[ignore = "requires PostgreSQL with saga schema"]
    async fn test_postgres_find_incomplete() {
        // Started/Running/Compensating 状態の saga のみ返されることを検証
        // 1. 各ステータスの saga を作成 (Started, Running, Completed, Failed, Compensating)
        // 2. find_incomplete() で Started, Running, Compensating のみ返される
    }

    #[tokio::test]
    #[cfg(feature = "integration-tests")]
    #[ignore = "requires PostgreSQL with saga schema"]
    async fn test_postgres_list_with_filters() {
        // workflow_name, status, correlation_id フィルタとページネーションの検証
        // 1. 異なる workflow_name / status / correlation_id の saga を複数作成
        // 2. SagaListParams の各フィルタで正しく絞り込まれることを確認
        // 3. page / page_size によるページネーションを検証
    }
}
