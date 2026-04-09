// B-MEDIUM-02 監査対応: usecase レイヤーの infrastructure 依存を解消
// このファイルは元々 usecase/postgres_support.rs に存在していたが、
// sqlx::Postgres / sqlx::Transaction などの infrastructure 具体型を直接参照しており
// クリーンアーキテクチャ原則（usecase は infrastructure に依存してはならない）に違反していた。
// adapter/repository レイヤーへ移動することで依存方向を正しく修正する。

// RUST-CRIT-001 対応: テナント分離のため各 TX 操作に tenant_id を追加する

// TODO(future-work): RLS テナント分離の統合テストが不足している。
// 優先度: MEDIUM。異なる tenant_id でのクロステナントアクセス拒否を検証するテストを追加すること。
// テストケース: tenant_A でINSERT → tenant_B でSELECT → 0件を確認
// 実装には testcontainers か sqlx::test を使用し、マイグレーション適用後に RLS ポリシーを有効化する。
// 対応時は integration-test.yaml CI ワークフローに組み込み、ADR を作成すること。

use crate::domain::entity::workflow_instance::WorkflowInstance;
use crate::domain::entity::workflow_task::WorkflowTask;
use sqlx::{Postgres, Transaction};

// ワークフローインスタンスをトランザクション内に新規挿入する
// tenant_id を SET LOCAL で設定してから INSERT する
pub async fn insert_instance_tx(
    tx: &mut Transaction<'_, Postgres>,
    instance: &WorkflowInstance,
    tenant_id: &str,
) -> anyhow::Result<()> {
    // テナント分離: RLS のために現在のテナントIDをセッション変数に設定する
    // HIGH-006 監査対応: SET LOCAL は $1 パラメータバインドをサポートしないため set_config() を使用する
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id)
        .execute(&mut **tx)
        .await?;

    // current_step_id が None の場合は空文字列として挿入する
    let current_step = instance
        .current_step_id
        .as_deref()
        .unwrap_or("")
        .to_string();

    sqlx::query(
        "INSERT INTO workflow.workflow_instances \
         (id, definition_id, workflow_name, title, initiator_id, current_step_id, \
          status, context, started_at, completed_at, created_at, tenant_id) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
    )
    .bind(&instance.id)
    .bind(&instance.workflow_id)
    .bind(&instance.workflow_name)
    .bind(&instance.title)
    .bind(&instance.initiator_id)
    .bind(&current_step)
    .bind(&instance.status)
    .bind(&instance.context)
    .bind(instance.started_at)
    .bind(instance.completed_at)
    .bind(instance.created_at)
    .bind(tenant_id)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

// ワークフローインスタンスをトランザクション内で更新する
pub async fn update_instance_tx(
    tx: &mut Transaction<'_, Postgres>,
    instance: &WorkflowInstance,
    tenant_id: &str,
) -> anyhow::Result<()> {
    // テナント分離: RLS のために現在のテナントIDをセッション変数に設定する
    // HIGH-006 監査対応: SET LOCAL は $1 パラメータバインドをサポートしないため set_config() を使用する
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id)
        .execute(&mut **tx)
        .await?;

    // current_step_id が None の場合は空文字列として更新する
    let current_step = instance
        .current_step_id
        .as_deref()
        .unwrap_or("")
        .to_string();

    sqlx::query(
        "UPDATE workflow.workflow_instances \
         SET current_step_id = $2, status = $3, context = $4, completed_at = $5 \
         WHERE id = $1",
    )
    .bind(&instance.id)
    .bind(&current_step)
    .bind(&instance.status)
    .bind(&instance.context)
    .bind(instance.completed_at)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

// ワークフロータスクをトランザクション内に新規挿入する
// tenant_id を SET LOCAL で設定してから INSERT する
pub async fn insert_task_tx(
    tx: &mut Transaction<'_, Postgres>,
    task: &WorkflowTask,
    tenant_id: &str,
) -> anyhow::Result<()> {
    // テナント分離: RLS のために現在のテナントIDをセッション変数に設定する
    // HIGH-006 監査対応: SET LOCAL は $1 パラメータバインドをサポートしないため set_config() を使用する
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id)
        .execute(&mut **tx)
        .await?;

    // assignee_id が None の場合は空文字列として挿入する
    let assignee = task.assignee_id.as_deref().unwrap_or("").to_string();

    sqlx::query(
        "INSERT INTO workflow.workflow_tasks \
         (id, instance_id, step_id, step_name, assignee_id, status, \
          comment, actor_id, due_at, decided_at, created_at, updated_at, tenant_id) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)",
    )
    .bind(&task.id)
    .bind(&task.instance_id)
    .bind(&task.step_id)
    .bind(&task.step_name)
    .bind(&assignee)
    .bind(&task.status)
    .bind(&task.comment)
    .bind(&task.actor_id)
    .bind(task.due_at)
    .bind(task.decided_at)
    .bind(task.created_at)
    .bind(task.updated_at)
    .bind(tenant_id)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

// ワークフロータスクをトランザクション内で更新する
pub async fn update_task_tx(
    tx: &mut Transaction<'_, Postgres>,
    task: &WorkflowTask,
    tenant_id: &str,
) -> anyhow::Result<()> {
    // テナント分離: RLS のために現在のテナントIDをセッション変数に設定する
    // HIGH-006 監査対応: SET LOCAL は $1 パラメータバインドをサポートしないため set_config() を使用する
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id)
        .execute(&mut **tx)
        .await?;

    // assignee_id が None の場合は空文字列として更新する
    let assignee = task.assignee_id.as_deref().unwrap_or("").to_string();

    sqlx::query(
        "UPDATE workflow.workflow_tasks \
         SET assignee_id = $2, status = $3, comment = $4, actor_id = $5, \
             decided_at = $6, updated_at = $7 \
         WHERE id = $1",
    )
    .bind(&task.id)
    .bind(&assignee)
    .bind(&task.status)
    .bind(&task.comment)
    .bind(&task.actor_id)
    .bind(task.decided_at)
    .bind(task.updated_at)
    .execute(&mut **tx)
    .await?;

    Ok(())
}
