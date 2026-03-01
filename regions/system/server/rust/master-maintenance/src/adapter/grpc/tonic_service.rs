use std::collections::BTreeMap;

use tonic::{Request, Response, Status};

use crate::adapter::grpc::master_maintenance_grpc::MasterMaintenanceGrpcService;
use crate::proto::k1s0::system::mastermaintenance::v1::{
    master_maintenance_service_server::MasterMaintenanceService, CheckConsistencyRequest,
    CheckConsistencyResponse, ColumnDefinition as ProtoColumnDefinition, ConsistencyResult,
    CreateRecordRequest, DeleteRecordRequest, DeleteRecordResponse, GetRecordRequest,
    GetTableDefinitionRequest, GetTableSchemaRequest, ListRecordsRequest, ListRecordsResponse,
    ListTableDefinitionsRequest, ListTableDefinitionsResponse, RecordResponse,
    TableDefinitionResponse, TableRelationship as ProtoTableRelationship, TableSchemaResponse,
    UpdateRecordRequest,
};
use crate::proto::k1s0::system::common::v1::PaginationResult;

// --- serde_json::Value <-> prost_types 変換ヘルパー ---

fn json_value_to_prost(value: &serde_json::Value) -> prost_types::Value {
    let kind = match value {
        serde_json::Value::Null => {
            prost_types::value::Kind::NullValue(0)
        }
        serde_json::Value::Bool(b) => {
            prost_types::value::Kind::BoolValue(*b)
        }
        serde_json::Value::Number(n) => {
            prost_types::value::Kind::NumberValue(n.as_f64().unwrap_or(0.0))
        }
        serde_json::Value::String(s) => {
            prost_types::value::Kind::StringValue(s.clone())
        }
        serde_json::Value::Array(arr) => {
            let values = arr.iter().map(json_value_to_prost).collect();
            prost_types::value::Kind::ListValue(prost_types::ListValue { values })
        }
        serde_json::Value::Object(map) => {
            let fields = map
                .iter()
                .map(|(k, v)| (k.clone(), json_value_to_prost(v)))
                .collect();
            prost_types::value::Kind::StructValue(prost_types::Struct { fields })
        }
    };
    prost_types::Value { kind: Some(kind) }
}

fn json_to_struct(value: &serde_json::Value) -> Option<prost_types::Struct> {
    match value {
        serde_json::Value::Object(map) => {
            let fields: BTreeMap<String, prost_types::Value> = map
                .iter()
                .map(|(k, v)| (k.clone(), json_value_to_prost(v)))
                .collect();
            Some(prost_types::Struct { fields })
        }
        _ => None,
    }
}

fn struct_to_json(s: &prost_types::Struct) -> serde_json::Value {
    let map: serde_json::Map<String, serde_json::Value> = s
        .fields
        .iter()
        .map(|(k, v)| (k.clone(), prost_value_to_json(v)))
        .collect();
    serde_json::Value::Object(map)
}

fn prost_value_to_json(value: &prost_types::Value) -> serde_json::Value {
    match &value.kind {
        Some(prost_types::value::Kind::NullValue(_)) => serde_json::Value::Null,
        Some(prost_types::value::Kind::BoolValue(b)) => serde_json::Value::Bool(*b),
        Some(prost_types::value::Kind::NumberValue(n)) => {
            serde_json::json!(*n)
        }
        Some(prost_types::value::Kind::StringValue(s)) => {
            serde_json::Value::String(s.clone())
        }
        Some(prost_types::value::Kind::ListValue(list)) => {
            let arr: Vec<serde_json::Value> =
                list.values.iter().map(prost_value_to_json).collect();
            serde_json::Value::Array(arr)
        }
        Some(prost_types::value::Kind::StructValue(s)) => struct_to_json(s),
        None => serde_json::Value::Null,
    }
}

// --- ドメインエンティティ -> Proto 変換 ---

fn domain_column_to_proto(
    col: &crate::domain::entity::column_definition::ColumnDefinition,
) -> ProtoColumnDefinition {
    ProtoColumnDefinition {
        column_name: col.column_name.clone(),
        display_name: col.display_name.clone(),
        data_type: col.data_type.clone(),
        is_primary_key: col.is_primary_key,
        is_nullable: col.is_nullable,
        is_searchable: col.is_searchable,
        is_sortable: col.is_sortable,
        is_filterable: col.is_filterable,
        is_visible_in_list: col.is_visible_in_list,
        is_visible_in_form: col.is_visible_in_form,
        is_readonly: col.is_readonly,
        input_type: col.input_type.clone(),
        display_order: col.display_order,
    }
}

fn domain_relationship_to_proto(
    rel: &crate::domain::entity::table_relationship::TableRelationship,
    target_table_name: &str,
) -> ProtoTableRelationship {
    ProtoTableRelationship {
        source_column: rel.source_column.clone(),
        target_table: target_table_name.to_string(),
        target_column: rel.target_column.clone(),
        relationship_type: rel.relationship_type.to_string(),
        display_name: rel.display_name.clone().unwrap_or_default(),
    }
}

fn domain_table_to_proto(
    table: &crate::domain::entity::table_definition::TableDefinition,
    columns: Vec<ProtoColumnDefinition>,
    relationships: Vec<ProtoTableRelationship>,
) -> TableDefinitionResponse {
    TableDefinitionResponse {
        id: table.id.to_string(),
        name: table.name.clone(),
        schema_name: table.schema_name.clone(),
        display_name: table.display_name.clone(),
        description: table.description.clone().unwrap_or_default(),
        allow_create: table.allow_create,
        allow_update: table.allow_update,
        allow_delete: table.allow_delete,
        columns,
        relationships,
    }
}

// --- MasterMaintenanceService 実装 ---

#[tonic::async_trait]
impl MasterMaintenanceService for MasterMaintenanceGrpcService {
    async fn get_table_definition(
        &self,
        request: Request<GetTableDefinitionRequest>,
    ) -> Result<Response<TableDefinitionResponse>, Status> {
        let req = request.into_inner();
        if req.table_name.is_empty() {
            return Err(Status::invalid_argument("table_name is required"));
        }

        let table = self
            .manage_tables_uc
            .get_table(&req.table_name)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| {
                Status::not_found(format!("Table '{}' not found", req.table_name))
            })?;

        let columns = self
            .column_repo
            .find_by_table_id(table.id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let relationships = self
            .relationship_repo
            .find_by_table_id(table.id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        // リレーションの target_table_id を名前に解決
        let mut proto_relationships = Vec::new();
        for rel in &relationships {
            let target_name = self
                .manage_tables_uc
                .get_table_by_id(rel.target_table_id)
                .await
                .ok()
                .flatten()
                .map(|t| t.name.clone())
                .unwrap_or_else(|| rel.target_table_id.to_string());
            proto_relationships.push(domain_relationship_to_proto(rel, &target_name));
        }

        let proto_columns: Vec<ProtoColumnDefinition> =
            columns.iter().map(domain_column_to_proto).collect();

        Ok(Response::new(domain_table_to_proto(
            &table,
            proto_columns,
            proto_relationships,
        )))
    }

    async fn list_table_definitions(
        &self,
        request: Request<ListTableDefinitionsRequest>,
    ) -> Result<Response<ListTableDefinitionsResponse>, Status> {
        let req = request.into_inner();
        let category = if req.category.is_empty() {
            None
        } else {
            Some(req.category.as_str())
        };

        let tables = self
            .manage_tables_uc
            .list_tables(category, req.active_only)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let pagination = req.pagination.unwrap_or(
            crate::proto::k1s0::system::common::v1::Pagination {
                page: 1,
                page_size: 100,
            },
        );

        let total_count = tables.len() as i32;
        let start = ((pagination.page - 1) * pagination.page_size) as usize;
        let page_tables: Vec<_> = tables
            .into_iter()
            .skip(start)
            .take(pagination.page_size as usize)
            .collect();

        let has_next = (start + page_tables.len()) < total_count as usize;

        let mut proto_tables = Vec::new();
        for table in &page_tables {
            let columns = self
                .column_repo
                .find_by_table_id(table.id)
                .await
                .map_err(|e| Status::internal(e.to_string()))?;
            let proto_columns: Vec<ProtoColumnDefinition> =
                columns.iter().map(domain_column_to_proto).collect();

            let relationships = self
                .relationship_repo
                .find_by_table_id(table.id)
                .await
                .map_err(|e| Status::internal(e.to_string()))?;

            let mut proto_relationships = Vec::new();
            for rel in &relationships {
                let target_name = self
                    .manage_tables_uc
                    .get_table_by_id(rel.target_table_id)
                    .await
                    .ok()
                    .flatten()
                    .map(|t| t.name.clone())
                    .unwrap_or_else(|| rel.target_table_id.to_string());
                proto_relationships.push(domain_relationship_to_proto(rel, &target_name));
            }

            proto_tables.push(domain_table_to_proto(
                table,
                proto_columns,
                proto_relationships,
            ));
        }

        Ok(Response::new(ListTableDefinitionsResponse {
            tables: proto_tables,
            pagination: Some(PaginationResult {
                total_count,
                page: pagination.page,
                page_size: pagination.page_size,
                has_next,
            }),
        }))
    }

    async fn get_record(
        &self,
        request: Request<GetRecordRequest>,
    ) -> Result<Response<RecordResponse>, Status> {
        let req = request.into_inner();
        if req.table_name.is_empty() {
            return Err(Status::invalid_argument("table_name is required"));
        }
        if req.record_id.is_empty() {
            return Err(Status::invalid_argument("record_id is required"));
        }

        let record = self
            .crud_records_uc
            .get_record(&req.table_name, &req.record_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| {
                Status::not_found(format!(
                    "Record '{}' not found in table '{}'",
                    req.record_id, req.table_name
                ))
            })?;

        Ok(Response::new(RecordResponse {
            data: json_to_struct(&record),
            warnings: vec![],
        }))
    }

    async fn list_records(
        &self,
        request: Request<ListRecordsRequest>,
    ) -> Result<Response<ListRecordsResponse>, Status> {
        let req = request.into_inner();
        if req.table_name.is_empty() {
            return Err(Status::invalid_argument("table_name is required"));
        }

        let pagination = req.pagination.unwrap_or(
            crate::proto::k1s0::system::common::v1::Pagination {
                page: 1,
                page_size: 50,
            },
        );

        let sort = if req.sort.is_empty() {
            None
        } else {
            Some(req.sort.as_str())
        };
        let filter = if req.filter.is_empty() {
            None
        } else {
            Some(req.filter.as_str())
        };
        let search = if req.search.is_empty() {
            None
        } else {
            Some(req.search.as_str())
        };

        let (records, total) = self
            .crud_records_uc
            .list_records(
                &req.table_name,
                pagination.page,
                pagination.page_size,
                sort,
                filter,
                search,
            )
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let proto_records: Vec<prost_types::Struct> = records
            .iter()
            .filter_map(json_to_struct)
            .collect();

        let total_count = total as i32;
        let has_next =
            (pagination.page * pagination.page_size) < total_count;

        Ok(Response::new(ListRecordsResponse {
            records: proto_records,
            pagination: Some(PaginationResult {
                total_count,
                page: pagination.page,
                page_size: pagination.page_size,
                has_next,
            }),
        }))
    }

    async fn create_record(
        &self,
        request: Request<CreateRecordRequest>,
    ) -> Result<Response<RecordResponse>, Status> {
        let req = request.into_inner();
        if req.table_name.is_empty() {
            return Err(Status::invalid_argument("table_name is required"));
        }
        let data_struct = req
            .data
            .as_ref()
            .ok_or_else(|| Status::invalid_argument("data is required"))?;

        let json_data = struct_to_json(data_struct);

        // gRPC リクエストでは created_by はメタデータから取得するのが理想だが、
        // ここでは data 内の created_by フィールドまたはデフォルト値を使用
        let created_by = json_data
            .get("created_by")
            .and_then(|v| v.as_str())
            .unwrap_or("grpc-user");

        let record = self
            .crud_records_uc
            .create_record(&req.table_name, &json_data, created_by)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(RecordResponse {
            data: json_to_struct(&record),
            warnings: vec![],
        }))
    }

    async fn update_record(
        &self,
        request: Request<UpdateRecordRequest>,
    ) -> Result<Response<RecordResponse>, Status> {
        let req = request.into_inner();
        if req.table_name.is_empty() {
            return Err(Status::invalid_argument("table_name is required"));
        }
        if req.record_id.is_empty() {
            return Err(Status::invalid_argument("record_id is required"));
        }
        let data_struct = req
            .data
            .as_ref()
            .ok_or_else(|| Status::invalid_argument("data is required"))?;

        let json_data = struct_to_json(data_struct);

        let updated_by = json_data
            .get("updated_by")
            .and_then(|v| v.as_str())
            .unwrap_or("grpc-user");

        let record = self
            .crud_records_uc
            .update_record(&req.table_name, &req.record_id, &json_data, updated_by)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(RecordResponse {
            data: json_to_struct(&record),
            warnings: vec![],
        }))
    }

    async fn delete_record(
        &self,
        request: Request<DeleteRecordRequest>,
    ) -> Result<Response<DeleteRecordResponse>, Status> {
        let req = request.into_inner();
        if req.table_name.is_empty() {
            return Err(Status::invalid_argument("table_name is required"));
        }
        if req.record_id.is_empty() {
            return Err(Status::invalid_argument("record_id is required"));
        }

        self.crud_records_uc
            .delete_record(&req.table_name, &req.record_id, "grpc-user")
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(DeleteRecordResponse { success: true }))
    }

    async fn check_consistency(
        &self,
        request: Request<CheckConsistencyRequest>,
    ) -> Result<Response<CheckConsistencyResponse>, Status> {
        let req = request.into_inner();
        if req.table_name.is_empty() {
            return Err(Status::invalid_argument("table_name is required"));
        }

        let results = self
            .check_consistency_uc
            .check_rules(&req.table_name, &req.rule_ids)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let total_checked = results.len() as i32;
        let error_count = results
            .iter()
            .filter(|r| !r.passed && r.severity == "error")
            .count() as i32;
        let warning_count = results
            .iter()
            .filter(|r| r.severity == "warning")
            .count() as i32;

        let proto_results: Vec<ConsistencyResult> = results
            .iter()
            .map(|r| ConsistencyResult {
                rule_id: r.rule_id.clone(),
                rule_name: r.rule_name.clone(),
                severity: r.severity.clone(),
                passed: r.passed,
                message: r.message.clone().unwrap_or_default(),
                affected_record_ids: r.affected_record_ids.clone(),
            })
            .collect();

        Ok(Response::new(CheckConsistencyResponse {
            results: proto_results,
            total_checked,
            error_count,
            warning_count,
        }))
    }

    async fn get_table_schema(
        &self,
        request: Request<GetTableSchemaRequest>,
    ) -> Result<Response<TableSchemaResponse>, Status> {
        let req = request.into_inner();
        if req.table_name.is_empty() {
            return Err(Status::invalid_argument("table_name is required"));
        }

        let schema = self
            .manage_tables_uc
            .get_table_schema(&req.table_name)
            .await
            .map_err(|e| {
                let msg = e.to_string();
                if msg.contains("not found") {
                    Status::not_found(msg)
                } else {
                    Status::internal(msg)
                }
            })?;

        let json_schema =
            serde_json::to_string(&schema).map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(TableSchemaResponse { json_schema }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_null_to_prost() {
        let val = serde_json::Value::Null;
        let prost_val = json_value_to_prost(&val);
        assert!(matches!(
            prost_val.kind,
            Some(prost_types::value::Kind::NullValue(0))
        ));
    }

    #[test]
    fn test_json_bool_to_prost() {
        let val = serde_json::Value::Bool(true);
        let prost_val = json_value_to_prost(&val);
        assert!(matches!(
            prost_val.kind,
            Some(prost_types::value::Kind::BoolValue(true))
        ));
    }

    #[test]
    fn test_json_number_to_prost() {
        let val = serde_json::json!(42.0);
        let prost_val = json_value_to_prost(&val);
        match prost_val.kind {
            Some(prost_types::value::Kind::NumberValue(n)) => assert!((n - 42.0).abs() < f64::EPSILON),
            _ => panic!("expected NumberValue"),
        }
    }

    #[test]
    fn test_json_string_to_prost() {
        let val = serde_json::json!("hello");
        let prost_val = json_value_to_prost(&val);
        assert!(matches!(
            prost_val.kind,
            Some(prost_types::value::Kind::StringValue(ref s)) if s == "hello"
        ));
    }

    #[test]
    fn test_json_object_to_struct_roundtrip() {
        let original = serde_json::json!({
            "name": "test",
            "count": 42,
            "active": true,
            "tags": ["a", "b"],
            "nested": { "key": "value" }
        });

        let prost_struct = json_to_struct(&original).expect("should convert to struct");
        let roundtrip = struct_to_json(&prost_struct);

        assert_eq!(
            original.get("name").and_then(|v| v.as_str()),
            roundtrip.get("name").and_then(|v| v.as_str())
        );
        assert_eq!(
            original.get("active").and_then(|v| v.as_bool()),
            roundtrip.get("active").and_then(|v| v.as_bool())
        );
        assert_eq!(
            original.get("count").and_then(|v| v.as_f64()),
            roundtrip.get("count").and_then(|v| v.as_f64())
        );
    }

    #[test]
    fn test_json_to_struct_non_object_returns_none() {
        assert!(json_to_struct(&serde_json::json!("string")).is_none());
        assert!(json_to_struct(&serde_json::json!(123)).is_none());
        assert!(json_to_struct(&serde_json::json!(null)).is_none());
    }
}
