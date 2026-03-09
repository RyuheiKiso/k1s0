use super::{router, AppState};
use crate::domain;
use crate::infrastructure;
use crate::usecase;
use crate::MIGRATOR;
use axum::body::{to_bytes, Body};
use axum::http::{Method, Request, StatusCode};
use axum::Router;
use serde_json::{json, Value};
use sqlx::PgPool;
use std::sync::Arc;
use tower::util::ServiceExt;

#[sqlx::test]
async fn metadata_and_record_lifecycle_round_trip(pool: PgPool) {
    MIGRATOR.run(&pool).await.unwrap();
    let app = build_test_app(pool);

    let table_response = send_json(
        &app,
        Method::POST,
        "/api/v1/tables",
        json!({
            "name": "departments",
            "schema_name": "business",
            "display_name": "Departments",
            "description": "Department catalog",
            "category": "organization",
            "allow_create": true,
            "allow_update": true,
            "allow_delete": true
        }),
    )
    .await;
    assert_eq!(table_response.0, StatusCode::CREATED);
    assert_eq!(table_response.1["name"], "departments");

    let columns_response = send_json(
        &app,
        Method::POST,
        "/api/v1/tables/departments/columns",
        json!([
            {
                "column_name": "id",
                "display_name": "ID",
                "data_type": "uuid",
                "is_primary_key": true,
                "is_nullable": false,
                "is_unique": true,
                "input_type": "text",
                "is_visible_in_list": true,
                "is_visible_in_form": false,
                "is_readonly": true
            },
            {
                "column_name": "name",
                "display_name": "Name",
                "data_type": "text",
                "is_nullable": false,
                "input_type": "text",
                "is_visible_in_list": true,
                "is_visible_in_form": true
            }
        ]),
    )
    .await;
    assert_eq!(columns_response.0, StatusCode::CREATED);
    assert_eq!(columns_response.1.as_array().unwrap().len(), 2);

    let record_id = uuid::Uuid::new_v4().to_string();
    let create_record_response = send_json(
        &app,
        Method::POST,
        "/api/v1/tables/departments/records",
        json!({
            "id": record_id,
            "name": "Platform"
        }),
    )
    .await;
    assert_eq!(create_record_response.0, StatusCode::CREATED);
    assert_eq!(create_record_response.1["data"]["name"], "Platform");
    assert_eq!(
        create_record_response.1["warnings"]
            .as_array()
            .unwrap()
            .len(),
        0
    );

    let list_records_response =
        send_empty(&app, Method::GET, "/api/v1/tables/departments/records").await;
    assert_eq!(list_records_response.0, StatusCode::OK);
    assert_eq!(list_records_response.1["total"], 1);
    assert_eq!(list_records_response.1["records"][0]["id"], record_id);
    assert_eq!(list_records_response.1["records"][0]["name"], "Platform");
    assert_eq!(list_records_response.1["metadata"]["allow_delete"], true);

    let update_record_response = send_json(
        &app,
        Method::PUT,
        &format!("/api/v1/tables/departments/records/{record_id}"),
        json!({
            "name": "Platform Core"
        }),
    )
    .await;
    assert_eq!(update_record_response.0, StatusCode::OK);
    assert_eq!(update_record_response.1["data"]["name"], "Platform Core");

    let audit_logs_response =
        send_empty(&app, Method::GET, "/api/v1/tables/departments/audit-logs").await;
    assert_eq!(audit_logs_response.0, StatusCode::OK);
    assert_eq!(audit_logs_response.1["total"], 2);
    assert_eq!(audit_logs_response.1["logs"][0]["operation"], "UPDATE");
    assert_eq!(audit_logs_response.1["logs"][1]["operation"], "INSERT");

    let delete_response = send_empty(
        &app,
        Method::DELETE,
        &format!("/api/v1/tables/departments/records/{record_id}"),
    )
    .await;
    assert_eq!(delete_response.0, StatusCode::NO_CONTENT);

    let audit_logs_after_delete =
        send_empty(&app, Method::GET, "/api/v1/tables/departments/audit-logs").await;
    assert_eq!(audit_logs_after_delete.0, StatusCode::OK);
    assert_eq!(audit_logs_after_delete.1["total"], 3);
    assert_eq!(audit_logs_after_delete.1["logs"][0]["operation"], "DELETE");
}

#[sqlx::test]
async fn display_config_lifecycle_round_trip(pool: PgPool) {
    MIGRATOR.run(&pool).await.unwrap();
    let app = build_test_app(pool);
    seed_departments_table(&app).await;

    let create_response = send_json(
        &app,
        Method::POST,
        "/api/v1/tables/departments/display-configs",
        json!({
            "config_type": "list_view",
            "config_json": {
                "columns": [
                    { "column_name": "name" }
                ]
            },
            "is_default": true
        }),
    )
    .await;
    assert_eq!(create_response.0, StatusCode::CREATED);
    let config_id = create_response.1["id"].as_str().unwrap().to_string();

    let list_response = send_empty(
        &app,
        Method::GET,
        "/api/v1/tables/departments/display-configs",
    )
    .await;
    assert_eq!(list_response.0, StatusCode::OK);
    assert_eq!(list_response.1.as_array().unwrap().len(), 1);
    assert_eq!(list_response.1[0]["config_type"], "list_view");

    let update_response = send_json(
        &app,
        Method::PUT,
        &format!("/api/v1/tables/departments/display-configs/{config_id}"),
        json!({
            "config_type": "detail_view",
            "config_json": {
                "sections": [
                    { "title": "Overview" }
                ]
            },
            "is_default": false
        }),
    )
    .await;
    assert_eq!(update_response.0, StatusCode::OK);
    assert_eq!(update_response.1["config_type"], "detail_view");
    assert_eq!(update_response.1["is_default"], false);

    let get_response = send_empty(
        &app,
        Method::GET,
        &format!("/api/v1/tables/departments/display-configs/{config_id}"),
    )
    .await;
    assert_eq!(get_response.0, StatusCode::OK);
    assert_eq!(
        get_response.1["config_json"]["sections"][0]["title"],
        "Overview"
    );

    let delete_response = send_empty(
        &app,
        Method::DELETE,
        &format!("/api/v1/tables/departments/display-configs/{config_id}"),
    )
    .await;
    assert_eq!(delete_response.0, StatusCode::NO_CONTENT);

    let list_after_delete = send_empty(
        &app,
        Method::GET,
        "/api/v1/tables/departments/display-configs",
    )
    .await;
    assert_eq!(list_after_delete.0, StatusCode::OK);
    assert_eq!(list_after_delete.1.as_array().unwrap().len(), 0);
}

#[sqlx::test]
async fn relationship_and_related_records_round_trip(pool: PgPool) {
    MIGRATOR.run(&pool).await.unwrap();
    let app = build_test_app(pool);
    seed_departments_table(&app).await;
    seed_employees_table(&app).await;

    let department_id = uuid::Uuid::new_v4().to_string();
    let employee_id = uuid::Uuid::new_v4().to_string();

    let create_department_response = send_json(
        &app,
        Method::POST,
        "/api/v1/tables/departments/records",
        json!({
            "id": department_id,
            "name": "Platform"
        }),
    )
    .await;
    assert_eq!(create_department_response.0, StatusCode::CREATED);

    let create_relationship_response = send_json(
        &app,
        Method::POST,
        "/api/v1/relationships",
        json!({
            "source_table": "employees",
            "source_column": "department_id",
            "target_table": "departments",
            "target_column": "id",
            "relationship_type": "many_to_one",
            "display_name": "Department",
            "is_cascade_delete": false
        }),
    )
    .await;
    assert_eq!(create_relationship_response.0, StatusCode::CREATED);
    let relationship_id = create_relationship_response.1["id"]
        .as_str()
        .unwrap()
        .to_string();

    let create_employee_response = send_json(
        &app,
        Method::POST,
        "/api/v1/tables/employees/records",
        json!({
            "id": employee_id,
            "name": "Alice",
            "department_id": department_id
        }),
    )
    .await;
    assert_eq!(create_employee_response.0, StatusCode::CREATED);

    let related_records_response = send_empty(
        &app,
        Method::GET,
        &format!("/api/v1/tables/employees/related-records/{employee_id}"),
    )
    .await;
    assert_eq!(related_records_response.0, StatusCode::OK);
    assert_eq!(
        related_records_response.1["departments"]["records"][0]["name"],
        "Platform"
    );
    assert_eq!(
        related_records_response.1["departments"]["relationship_id"],
        relationship_id
    );

    let delete_relationship_response = send_empty(
        &app,
        Method::DELETE,
        &format!("/api/v1/relationships/{relationship_id}"),
    )
    .await;
    assert_eq!(delete_relationship_response.0, StatusCode::NO_CONTENT);
}

#[sqlx::test]
async fn import_and_export_csv_round_trip(pool: PgPool) {
    MIGRATOR.run(&pool).await.unwrap();
    let app = build_test_app(pool);
    seed_departments_table(&app).await;

    let import_response = send_json(
        &app,
        Method::POST,
        "/api/v1/tables/departments/import",
        json!({
            "file_name": "departments.csv",
            "format": "csv",
            "content": "id,name\n11111111-1111-1111-1111-111111111111,Platform\n22222222-2222-2222-2222-222222222222,Security\n"
        }),
    )
    .await;
    assert_eq!(import_response.0, StatusCode::CREATED);
    let job_id = import_response.1["id"].as_str().unwrap().to_string();
    assert_eq!(import_response.1["status"], "completed");
    assert_eq!(import_response.1["processed_rows"], 2);
    assert_eq!(import_response.1["error_rows"], 0);

    let import_job_response =
        send_empty(&app, Method::GET, &format!("/api/v1/import-jobs/{job_id}")).await;
    assert_eq!(import_job_response.0, StatusCode::OK);
    assert_eq!(import_job_response.1["file_name"], "departments.csv");

    let export_response = send_raw(
        &app,
        Request::builder()
            .method(Method::GET)
            .uri("/api/v1/tables/departments/export?format=csv")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(export_response.0, StatusCode::OK);
    assert!(export_response.1.contains("id,name"));
    assert!(export_response
        .1
        .contains("11111111-1111-1111-1111-111111111111,Platform"));
    assert!(export_response
        .1
        .contains("22222222-2222-2222-2222-222222222222,Security"));
}

#[sqlx::test]
async fn import_file_multipart_round_trip(pool: PgPool) {
    MIGRATOR.run(&pool).await.unwrap();
    let app = build_test_app(pool);
    seed_departments_table(&app).await;

    let boundary = "master-maintenance-boundary";
    let multipart_body = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"departments.csv\"\r\nContent-Type: text/csv\r\n\r\nid,name\r\n33333333-3333-3333-3333-333333333333,People Ops\r\n44444444-4444-4444-4444-444444444444,Finance\r\n--{boundary}--\r\n"
    );

    let import_response = send(
        &app,
        Request::builder()
            .method(Method::POST)
            .uri("/api/v1/tables/departments/import-file")
            .header(
                "content-type",
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(Body::from(multipart_body))
            .unwrap(),
    )
    .await;
    assert_eq!(import_response.0, StatusCode::CREATED);
    assert_eq!(import_response.1["file_name"], "departments.csv");
    assert_eq!(import_response.1["status"], "completed");
    assert_eq!(import_response.1["processed_rows"], 2);

    let list_records_response =
        send_empty(&app, Method::GET, "/api/v1/tables/departments/records").await;
    assert_eq!(list_records_response.0, StatusCode::OK);
    assert_eq!(list_records_response.1["total"], 2);
    assert_eq!(list_records_response.1["records"][0]["name"], "People Ops");
}

#[sqlx::test]
async fn rule_create_check_and_execute_round_trip(pool: PgPool) {
    MIGRATOR.run(&pool).await.unwrap();
    let app = build_test_app(pool);
    seed_departments_table_with_headcount(&app).await;

    let invalid_department_id = uuid::Uuid::new_v4().to_string();
    let valid_department_id = uuid::Uuid::new_v4().to_string();

    let invalid_record_response = send_json(
        &app,
        Method::POST,
        "/api/v1/tables/departments/records",
        json!({
            "id": invalid_department_id,
            "name": "Platform",
            "headcount": -1
        }),
    )
    .await;
    assert_eq!(invalid_record_response.0, StatusCode::CREATED);

    let valid_record_response = send_json(
        &app,
        Method::POST,
        "/api/v1/tables/departments/records",
        json!({
            "id": valid_department_id,
            "name": "Security",
            "headcount": 12
        }),
    )
    .await;
    assert_eq!(valid_record_response.0, StatusCode::CREATED);

    let create_rule_response = send_json(
        &app,
        Method::POST,
        "/api/v1/rules",
        json!({
            "name": "department_headcount_non_negative",
            "description": "Headcount must stay non-negative",
            "rule_type": "range",
            "severity": "error",
            "source_table": "departments",
            "evaluation_timing": "on_demand",
            "error_message_template": "Headcount must be >= 0",
            "conditions": [
                {
                    "condition_order": 1,
                    "left_column": "headcount",
                    "operator": "gte",
                    "right_value": "0",
                    "logical_connector": "AND"
                }
            ]
        }),
    )
    .await;
    assert_eq!(create_rule_response.0, StatusCode::CREATED);
    let rule_id = create_rule_response.1["id"].as_str().unwrap().to_string();

    let check_rules_response = send_json(
        &app,
        Method::POST,
        "/api/v1/rules/check",
        json!({
            "table_name": "departments"
        }),
    )
    .await;
    assert_eq!(check_rules_response.0, StatusCode::OK);
    assert_eq!(check_rules_response.1.as_array().unwrap().len(), 1);
    assert_eq!(
        check_rules_response.1[0]["rule_name"],
        "department_headcount_non_negative"
    );
    assert_eq!(check_rules_response.1[0]["severity"], "error");

    let execute_rule_response = send_empty(
        &app,
        Method::POST,
        &format!("/api/v1/rules/{rule_id}/execute"),
    )
    .await;
    assert_eq!(execute_rule_response.0, StatusCode::OK);
    assert_eq!(execute_rule_response.1.as_array().unwrap().len(), 1);
    assert_eq!(
        execute_rule_response.1[0]["message"],
        "Headcount must be >= 0"
    );

    let update_rule_response = send_json(
        &app,
        Method::PUT,
        &format!("/api/v1/rules/{rule_id}"),
        json!({
            "severity": "warning",
            "error_message_template": "Headcount should stay >= 0",
            "conditions": [
                {
                    "condition_order": 1,
                    "left_column": "headcount",
                    "operator": "gte",
                    "right_value": "5",
                    "logical_connector": "AND"
                }
            ]
        }),
    )
    .await;
    assert_eq!(update_rule_response.0, StatusCode::OK);
    assert_eq!(update_rule_response.1["severity"], "warning");
    assert_eq!(
        update_rule_response.1["error_message_template"],
        "Headcount should stay >= 0"
    );

    let execute_updated_rule_response = send_empty(
        &app,
        Method::POST,
        &format!("/api/v1/rules/{rule_id}/execute"),
    )
    .await;
    assert_eq!(execute_updated_rule_response.0, StatusCode::OK);
    assert_eq!(execute_updated_rule_response.1.as_array().unwrap().len(), 1);
    assert_eq!(execute_updated_rule_response.1[0]["severity"], "warning");
    assert_eq!(
        execute_updated_rule_response.1[0]["message"],
        "Headcount should stay >= 0"
    );

    let delete_rule_response =
        send_empty(&app, Method::DELETE, &format!("/api/v1/rules/{rule_id}")).await;
    assert_eq!(delete_rule_response.0, StatusCode::NO_CONTENT);

    let get_deleted_rule_response =
        send_empty(&app, Method::GET, &format!("/api/v1/rules/{rule_id}")).await;
    assert_eq!(get_deleted_rule_response.0, StatusCode::NOT_FOUND);
}

async fn seed_departments_table(app: &Router) {
    let table_response = send_json(
        app,
        Method::POST,
        "/api/v1/tables",
        json!({
            "name": "departments",
            "schema_name": "business",
            "display_name": "Departments",
            "description": "Department catalog",
            "category": "organization",
            "allow_create": true,
            "allow_update": true,
            "allow_delete": true
        }),
    )
    .await;
    assert_eq!(table_response.0, StatusCode::CREATED);

    let columns_response = send_json(
        app,
        Method::POST,
        "/api/v1/tables/departments/columns",
        json!([
            {
                "column_name": "id",
                "display_name": "ID",
                "data_type": "uuid",
                "is_primary_key": true,
                "is_nullable": false,
                "is_unique": true,
                "input_type": "text",
                "is_visible_in_list": true,
                "is_visible_in_form": false,
                "is_readonly": true
            },
            {
                "column_name": "name",
                "display_name": "Name",
                "data_type": "text",
                "is_nullable": false,
                "input_type": "text",
                "is_visible_in_list": true,
                "is_visible_in_form": true
            }
        ]),
    )
    .await;
    assert_eq!(columns_response.0, StatusCode::CREATED);
}

async fn seed_employees_table(app: &Router) {
    let table_response = send_json(
        app,
        Method::POST,
        "/api/v1/tables",
        json!({
            "name": "employees",
            "schema_name": "business",
            "display_name": "Employees",
            "description": "Employee catalog",
            "category": "organization",
            "allow_create": true,
            "allow_update": true,
            "allow_delete": true
        }),
    )
    .await;
    assert_eq!(table_response.0, StatusCode::CREATED);

    let columns_response = send_json(
        app,
        Method::POST,
        "/api/v1/tables/employees/columns",
        json!([
            {
                "column_name": "id",
                "display_name": "ID",
                "data_type": "uuid",
                "is_primary_key": true,
                "is_nullable": false,
                "is_unique": true,
                "input_type": "text",
                "is_visible_in_list": true,
                "is_visible_in_form": false,
                "is_readonly": true
            },
            {
                "column_name": "name",
                "display_name": "Name",
                "data_type": "text",
                "is_nullable": false,
                "input_type": "text",
                "is_visible_in_list": true,
                "is_visible_in_form": true
            },
            {
                "column_name": "department_id",
                "display_name": "Department ID",
                "data_type": "uuid",
                "is_nullable": false,
                "input_type": "text",
                "is_visible_in_list": true,
                "is_visible_in_form": true
            }
        ]),
    )
    .await;
    assert_eq!(columns_response.0, StatusCode::CREATED);
}

async fn seed_departments_table_with_headcount(app: &Router) {
    let table_response = send_json(
        app,
        Method::POST,
        "/api/v1/tables",
        json!({
            "name": "departments",
            "schema_name": "business",
            "display_name": "Departments",
            "description": "Department catalog",
            "category": "organization",
            "allow_create": true,
            "allow_update": true,
            "allow_delete": true
        }),
    )
    .await;
    assert_eq!(table_response.0, StatusCode::CREATED);

    let columns_response = send_json(
        app,
        Method::POST,
        "/api/v1/tables/departments/columns",
        json!([
            {
                "column_name": "id",
                "display_name": "ID",
                "data_type": "uuid",
                "is_primary_key": true,
                "is_nullable": false,
                "is_unique": true,
                "input_type": "text",
                "is_visible_in_list": true,
                "is_visible_in_form": false,
                "is_readonly": true
            },
            {
                "column_name": "name",
                "display_name": "Name",
                "data_type": "text",
                "is_nullable": false,
                "input_type": "text",
                "is_visible_in_list": true,
                "is_visible_in_form": true
            },
            {
                "column_name": "headcount",
                "display_name": "Headcount",
                "data_type": "integer",
                "is_nullable": false,
                "input_type": "number",
                "is_visible_in_list": true,
                "is_visible_in_form": true
            }
        ]),
    )
    .await;
    assert_eq!(columns_response.0, StatusCode::CREATED);
}

fn build_test_app(pool: PgPool) -> Router {
    let table_repo: Arc<
        dyn domain::repository::table_definition_repository::TableDefinitionRepository,
    > = Arc::new(
        infrastructure::persistence::table_definition_repo_impl::TableDefinitionPostgresRepository::new(
            pool.clone(),
        ),
    );
    let column_repo: Arc<
        dyn domain::repository::column_definition_repository::ColumnDefinitionRepository,
    > = Arc::new(
        infrastructure::persistence::column_definition_repo_impl::ColumnDefinitionPostgresRepository::new(
            pool.clone(),
        ),
    );
    let rule_repo: Arc<
        dyn domain::repository::consistency_rule_repository::ConsistencyRuleRepository,
    > = Arc::new(
        infrastructure::persistence::consistency_rule_repo_impl::ConsistencyRulePostgresRepository::new(
            pool.clone(),
        ),
    );
    let record_repo: Arc<
        dyn domain::repository::dynamic_record_repository::DynamicRecordRepository,
    > = Arc::new(
        infrastructure::persistence::dynamic_record_repo_impl::DynamicRecordPostgresRepository::new(
            pool.clone(),
        ),
    );
    let change_log_repo: Arc<dyn domain::repository::change_log_repository::ChangeLogRepository> =
        Arc::new(
            infrastructure::persistence::change_log_repo_impl::ChangeLogPostgresRepository::new(
                pool.clone(),
            ),
        );
    let relationship_repo: Arc<
        dyn domain::repository::table_relationship_repository::TableRelationshipRepository,
    > = Arc::new(
        infrastructure::persistence::table_relationship_repo_impl::TableRelationshipPostgresRepository::new(
            pool.clone(),
        ),
    );
    let display_config_repo: Arc<
        dyn domain::repository::display_config_repository::DisplayConfigRepository,
    > = Arc::new(
        infrastructure::persistence::display_config_repo_impl::DisplayConfigPostgresRepository::new(
            pool.clone(),
        ),
    );
    let import_job_repo: Arc<dyn domain::repository::import_job_repository::ImportJobRepository> =
        Arc::new(
            infrastructure::persistence::import_job_repo_impl::ImportJobPostgresRepository::new(
                pool.clone(),
            ),
        );

    let rule_engine =
        Arc::new(infrastructure::rule_engine::zen_engine_adapter::ZenEngineAdapter::new());
    let schema_manager = Arc::new(infrastructure::schema::PhysicalSchemaManager::new(pool));

    let crud_records_uc = Arc::new(usecase::crud_records::CrudRecordsUseCase::new(
        table_repo.clone(),
        column_repo.clone(),
        rule_repo.clone(),
        record_repo.clone(),
        change_log_repo.clone(),
        rule_engine.clone(),
    ));

    let state = AppState {
        manage_tables_uc: Arc::new(
            usecase::manage_table_definitions::ManageTableDefinitionsUseCase::new(
                table_repo.clone(),
                column_repo.clone(),
                schema_manager.clone(),
            ),
        ),
        manage_columns_uc: Arc::new(
            usecase::manage_column_definitions::ManageColumnDefinitionsUseCase::new(
                table_repo.clone(),
                column_repo.clone(),
                schema_manager.clone(),
            ),
        ),
        crud_records_uc: crud_records_uc.clone(),
        manage_rules_uc: Arc::new(usecase::manage_rules::ManageRulesUseCase::new(
            table_repo.clone(),
            rule_repo.clone(),
        )),
        check_consistency_uc: Arc::new(usecase::check_consistency::CheckConsistencyUseCase::new(
            table_repo.clone(),
            column_repo.clone(),
            rule_repo.clone(),
            record_repo.clone(),
            rule_engine,
        )),
        get_audit_logs_uc: Arc::new(usecase::get_audit_logs::GetAuditLogsUseCase::new(
            change_log_repo,
        )),
        manage_relationships_uc: Arc::new(
            usecase::manage_relationships::ManageRelationshipsUseCase::new(
                table_repo.clone(),
                relationship_repo,
                record_repo.clone(),
                column_repo.clone(),
                schema_manager,
            ),
        ),
        manage_display_configs_uc: Arc::new(
            usecase::manage_display_configs::ManageDisplayConfigsUseCase::new(
                table_repo.clone(),
                display_config_repo,
            ),
        ),
        import_export_uc: Arc::new(usecase::import_export::ImportExportUseCase::new(
            table_repo,
            column_repo,
            import_job_repo,
            crud_records_uc,
        )),
        metrics: Arc::new(k1s0_telemetry::metrics::Metrics::new(
            "master_maintenance_test",
        )),
        kafka_producer: None,
        auth_state: None,
    };

    router(state)
}

async fn send_json(app: &Router, method: Method, uri: &str, payload: Value) -> (StatusCode, Value) {
    let request = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();
    send(app, request).await
}

async fn send_empty(app: &Router, method: Method, uri: &str) -> (StatusCode, Value) {
    let request = Request::builder()
        .method(method)
        .uri(uri)
        .body(Body::empty())
        .unwrap();
    send(app, request).await
}

async fn send(app: &Router, request: Request<Body>) -> (StatusCode, Value) {
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();

    if status == StatusCode::NO_CONTENT {
        return (status, Value::Null);
    }

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let payload = serde_json::from_slice(&body).unwrap();
    (status, payload)
}

async fn send_raw(app: &Router, request: Request<Body>) -> (StatusCode, String) {
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (status, String::from_utf8(body.to_vec()).unwrap())
}
