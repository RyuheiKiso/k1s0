#![allow(clippy::unwrap_used)]
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use k1s0_api_registry_server::domain::entity::api_registration::{
    ApiSchema, ApiSchemaVersion, SchemaType,
};
use k1s0_api_registry_server::domain::repository::{
    ApiSchemaRepository, ApiSchemaVersionRepository,
};
use k1s0_api_registry_server::infrastructure::kafka::SchemaEventPublisher;

// ---------------------------------------------------------------------------
// In-memory stub: ApiSchemaRepository
// ---------------------------------------------------------------------------

struct StubSchemaRepository {
    schemas: RwLock<Vec<ApiSchema>>,
    should_fail: bool,
}

impl StubSchemaRepository {
    fn new() -> Self {
        Self {
            schemas: RwLock::new(Vec::new()),
            should_fail: false,
        }
    }

    fn with_schemas(schemas: Vec<ApiSchema>) -> Self {
        Self {
            schemas: RwLock::new(schemas),
            should_fail: false,
        }
    }

    fn failing() -> Self {
        Self {
            schemas: RwLock::new(Vec::new()),
            should_fail: true,
        }
    }
}

#[async_trait]
impl ApiSchemaRepository for StubSchemaRepository {
    // テナントスコープで検索するスタブ実装（テスト用のためテナント分離は省略）
    async fn find_by_name(
        &self,
        _tenant_id: &str,
        name: &str,
    ) -> anyhow::Result<Option<ApiSchema>> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        let schemas = self.schemas.read().await;
        Ok(schemas.iter().find(|s| s.name == name).cloned())
    }

    async fn find_all(
        &self,
        _tenant_id: &str,
        schema_type: Option<String>,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<ApiSchema>, u64)> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        let schemas = self.schemas.read().await;
        let filtered: Vec<ApiSchema> = schemas
            .iter()
            .filter(|s| {
                if let Some(ref st) = schema_type {
                    s.schema_type.to_string() == *st
                } else {
                    true
                }
            })
            .cloned()
            .collect();
        let total = filtered.len() as u64;
        let start = ((page - 1) * page_size) as usize;
        let end = (start + page_size as usize).min(filtered.len());
        let page_items = if start < filtered.len() {
            filtered[start..end].to_vec()
        } else {
            Vec::new()
        };
        Ok((page_items, total))
    }

    async fn create(&self, _tenant_id: &str, schema: &ApiSchema) -> anyhow::Result<()> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        self.schemas.write().await.push(schema.clone());
        Ok(())
    }

    async fn update(&self, _tenant_id: &str, schema: &ApiSchema) -> anyhow::Result<()> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        let mut schemas = self.schemas.write().await;
        if let Some(existing) = schemas.iter_mut().find(|s| s.name == schema.name) {
            *existing = schema.clone();
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// In-memory stub: ApiSchemaVersionRepository
// ---------------------------------------------------------------------------

struct StubVersionRepository {
    versions: RwLock<Vec<ApiSchemaVersion>>,
    should_fail: bool,
}

impl StubVersionRepository {
    fn new() -> Self {
        Self {
            versions: RwLock::new(Vec::new()),
            should_fail: false,
        }
    }

    fn with_versions(versions: Vec<ApiSchemaVersion>) -> Self {
        Self {
            versions: RwLock::new(versions),
            should_fail: false,
        }
    }

    fn failing() -> Self {
        Self {
            versions: RwLock::new(Vec::new()),
            should_fail: true,
        }
    }
}

#[async_trait]
impl ApiSchemaVersionRepository for StubVersionRepository {
    // テナントスコープで検索するスタブ実装（テスト用のためテナント分離は省略）
    async fn find_by_name_and_version(
        &self,
        _tenant_id: &str,
        name: &str,
        version: u32,
    ) -> anyhow::Result<Option<ApiSchemaVersion>> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        let versions = self.versions.read().await;
        Ok(versions
            .iter()
            .find(|v| v.name == name && v.version == version)
            .cloned())
    }

    async fn find_latest_by_name(
        &self,
        _tenant_id: &str,
        name: &str,
    ) -> anyhow::Result<Option<ApiSchemaVersion>> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        let versions = self.versions.read().await;
        Ok(versions
            .iter()
            .filter(|v| v.name == name)
            .max_by_key(|v| v.version)
            .cloned())
    }

    async fn find_all_by_name(
        &self,
        _tenant_id: &str,
        name: &str,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<ApiSchemaVersion>, u64)> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        let versions = self.versions.read().await;
        let filtered: Vec<ApiSchemaVersion> = versions
            .iter()
            .filter(|v| v.name == name)
            .cloned()
            .collect();
        let total = filtered.len() as u64;
        let start = ((page - 1) * page_size) as usize;
        let end = (start + page_size as usize).min(filtered.len());
        let page_items = if start < filtered.len() {
            filtered[start..end].to_vec()
        } else {
            Vec::new()
        };
        Ok((page_items, total))
    }

    async fn create(&self, _tenant_id: &str, version: &ApiSchemaVersion) -> anyhow::Result<()> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        self.versions.write().await.push(version.clone());
        Ok(())
    }

    async fn delete(&self, _tenant_id: &str, name: &str, version: u32) -> anyhow::Result<bool> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        let mut versions = self.versions.write().await;
        let len_before = versions.len();
        versions.retain(|v| !(v.name == name && v.version == version));
        Ok(versions.len() < len_before)
    }

    async fn count_by_name(&self, _tenant_id: &str, name: &str) -> anyhow::Result<u64> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        let versions = self.versions.read().await;
        Ok(versions.iter().filter(|v| v.name == name).count() as u64)
    }
}

// ---------------------------------------------------------------------------
// In-memory stub: SchemaEventPublisher
// ---------------------------------------------------------------------------

struct StubEventPublisher {
    should_fail: bool,
}

impl StubEventPublisher {
    #[allow(dead_code)]
    fn new() -> Self {
        Self { should_fail: false }
    }

    fn failing() -> Self {
        Self { should_fail: true }
    }
}

#[async_trait]
impl SchemaEventPublisher for StubEventPublisher {
    async fn publish_schema_updated(
        &self,
        _event: &k1s0_api_registry_server::infrastructure::kafka::SchemaUpdatedEvent,
    ) -> anyhow::Result<()> {
        if self.should_fail {
            anyhow::bail!("kafka unavailable");
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_schema(name: &str, schema_type: SchemaType) -> ApiSchema {
    ApiSchema::new(name.to_string(), format!("{} API", name), schema_type)
}

fn make_schema_with_versions(name: &str, latest: u32, count: u32) -> ApiSchema {
    let mut schema = make_schema(name, SchemaType::OpenApi);
    schema.latest_version = latest;
    schema.version_count = count;
    schema
}

fn make_version(
    name: &str,
    version: u32,
    schema_type: SchemaType,
    content: &str,
) -> ApiSchemaVersion {
    ApiSchemaVersion::new(
        name.to_string(),
        version,
        schema_type,
        content.to_string(),
        "test-user".to_string(),
    )
}

fn openapi_content_v1() -> String {
    "openapi: 3.0.3\npaths:\n  /api/v1/users:\n    get:\n      summary: Users\n  /api/v1/orders:\n    get:\n      summary: Orders\n".to_string()
}

fn openapi_content_v2_compatible() -> String {
    "openapi: 3.0.3\npaths:\n  /api/v1/users:\n    get:\n      summary: Users\n  /api/v1/orders:\n    get:\n      summary: Orders\n  /api/v1/products:\n    get:\n      summary: Products\n".to_string()
}

fn openapi_content_v2_breaking() -> String {
    "openapi: 3.0.3\npaths:\n  /api/v1/users:\n    get:\n      summary: Users\n".to_string()
}

fn protobuf_content_v1() -> String {
    "syntax = \"proto3\";\nmessage User {\n  string id = 1;\n  string name = 2;\n  string email = 3;\n}\n".to_string()
}

fn protobuf_content_v2_breaking() -> String {
    "syntax = \"proto3\";\nmessage User {\n  string id = 1;\n  string email = 3;\n}\n".to_string()
}

// ===========================================================================
// RegisterSchemaUseCase tests
// ===========================================================================

mod register_schema {
    use super::*;
    use k1s0_api_registry_server::usecase::register_schema::{
        RegisterSchemaError, RegisterSchemaInput, RegisterSchemaUseCase,
    };

    fn default_input() -> RegisterSchemaInput {
        RegisterSchemaInput {
            tenant_id: "tenant-a".to_string(),
            name: "tenant-api".to_string(),
            description: "Tenant management API".to_string(),
            schema_type: SchemaType::OpenApi,
            content: "openapi: 3.0.3".to_string(),
            registered_by: "user-001".to_string(),
        }
    }

    #[tokio::test]
    async fn success_creates_schema_and_version() {
        let schema_repo = Arc::new(StubSchemaRepository::new());
        let version_repo = Arc::new(StubVersionRepository::new());
        let uc = RegisterSchemaUseCase::new(schema_repo.clone(), version_repo.clone());

        let result = uc.execute(&default_input()).await;
        assert!(result.is_ok());

        let version = result.unwrap();
        assert_eq!(version.name, "tenant-api");
        assert_eq!(version.version, 1);
        assert_eq!(version.registered_by, "user-001");
        assert!(version.content_hash.starts_with("sha256:"));
        assert!(!version.breaking_changes);

        // Verify schema persisted
        let schemas = schema_repo.schemas.read().await;
        assert_eq!(schemas.len(), 1);
        assert_eq!(schemas[0].name, "tenant-api");
        assert_eq!(schemas[0].latest_version, 1);

        // Verify version persisted
        let versions = version_repo.versions.read().await;
        assert_eq!(versions.len(), 1);
    }

    #[tokio::test]
    async fn already_exists_returns_error() {
        let schema_repo = Arc::new(StubSchemaRepository::with_schemas(vec![make_schema(
            "tenant-api",
            SchemaType::OpenApi,
        )]));
        let version_repo = Arc::new(StubVersionRepository::new());
        let uc = RegisterSchemaUseCase::new(schema_repo, version_repo);

        let result = uc.execute(&default_input()).await;
        assert!(matches!(
            result,
            Err(RegisterSchemaError::AlreadyExists(ref name)) if name == "tenant-api"
        ));
    }

    #[tokio::test]
    async fn protobuf_schema_type_creates_correctly() {
        let schema_repo = Arc::new(StubSchemaRepository::new());
        let version_repo = Arc::new(StubVersionRepository::new());
        let uc = RegisterSchemaUseCase::new(schema_repo.clone(), version_repo);

        let input = RegisterSchemaInput {
            tenant_id: "tenant-a".to_string(),
            name: "proto-api".to_string(),
            description: "Protobuf API".to_string(),
            schema_type: SchemaType::Protobuf,
            content: protobuf_content_v1(),
            registered_by: "user-002".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let schemas = schema_repo.schemas.read().await;
        assert_eq!(schemas[0].schema_type, SchemaType::Protobuf);
    }

    #[tokio::test]
    async fn repository_error_returns_internal() {
        let schema_repo = Arc::new(StubSchemaRepository::failing());
        let version_repo = Arc::new(StubVersionRepository::new());
        let uc = RegisterSchemaUseCase::new(schema_repo, version_repo);

        let result = uc.execute(&default_input()).await;
        assert!(matches!(result, Err(RegisterSchemaError::Internal(_))));
    }

    #[tokio::test]
    async fn publisher_failure_does_not_fail_registration() {
        let schema_repo = Arc::new(StubSchemaRepository::new());
        let version_repo = Arc::new(StubVersionRepository::new());
        let publisher = Arc::new(StubEventPublisher::failing());
        let uc = RegisterSchemaUseCase::with_publisher(
            schema_repo.clone(),
            version_repo.clone(),
            publisher,
        );

        let result = uc.execute(&default_input()).await;
        assert!(
            result.is_ok(),
            "registration should succeed even if publisher fails"
        );

        let schemas = schema_repo.schemas.read().await;
        assert_eq!(schemas.len(), 1);
    }

    #[tokio::test]
    async fn content_hash_is_deterministic() {
        let schema_repo = Arc::new(StubSchemaRepository::new());
        let version_repo = Arc::new(StubVersionRepository::new());
        let uc = RegisterSchemaUseCase::new(schema_repo, version_repo.clone());

        let result = uc.execute(&default_input()).await.unwrap();
        let hash1 = result.content_hash.clone();

        // Same content should produce the same hash
        let expected_hash =
            k1s0_api_registry_server::domain::entity::api_registration::compute_content_hash(
                "openapi: 3.0.3",
            );
        assert_eq!(hash1, expected_hash);
    }
}

// ===========================================================================
// RegisterVersionUseCase tests
// ===========================================================================

mod register_version {
    use super::*;
    use k1s0_api_registry_server::usecase::register_version::{
        RegisterVersionError, RegisterVersionInput, RegisterVersionUseCase,
    };

    #[tokio::test]
    async fn success_increments_version() {
        let schema = make_schema("tenant-api", SchemaType::OpenApi);
        let v1 = make_version("tenant-api", 1, SchemaType::OpenApi, &openapi_content_v1());

        let schema_repo = Arc::new(StubSchemaRepository::with_schemas(vec![schema]));
        let version_repo = Arc::new(StubVersionRepository::with_versions(vec![v1]));
        let uc = RegisterVersionUseCase::new(schema_repo.clone(), version_repo.clone());

        let input = RegisterVersionInput {
            tenant_id: "tenant-a".to_string(),
            name: "tenant-api".to_string(),
            content: openapi_content_v2_compatible(),
            registered_by: "user-001".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let version = result.unwrap();
        assert_eq!(version.version, 2);
        assert_eq!(version.name, "tenant-api");

        // Verify schema metadata updated
        let schemas = schema_repo.schemas.read().await;
        assert_eq!(schemas[0].latest_version, 2);
        assert_eq!(schemas[0].version_count, 2);
    }

    #[tokio::test]
    async fn detects_breaking_changes_openapi() {
        let schema = make_schema("tenant-api", SchemaType::OpenApi);
        let v1 = make_version("tenant-api", 1, SchemaType::OpenApi, &openapi_content_v1());

        let schema_repo = Arc::new(StubSchemaRepository::with_schemas(vec![schema]));
        let version_repo = Arc::new(StubVersionRepository::with_versions(vec![v1]));
        let uc = RegisterVersionUseCase::new(schema_repo, version_repo);

        let input = RegisterVersionInput {
            tenant_id: "tenant-a".to_string(),
            name: "tenant-api".to_string(),
            content: openapi_content_v2_breaking(),
            registered_by: "user-001".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let version = result.unwrap();
        assert!(version.breaking_changes);
        assert!(!version.breaking_change_details.is_empty());
        assert_eq!(
            version.breaking_change_details[0].change_type,
            "path_removed"
        );
    }

    #[tokio::test]
    async fn detects_breaking_changes_protobuf() {
        let mut schema = make_schema("grpc-api", SchemaType::Protobuf);
        schema.schema_type = SchemaType::Protobuf;
        let v1 = make_version("grpc-api", 1, SchemaType::Protobuf, &protobuf_content_v1());

        let schema_repo = Arc::new(StubSchemaRepository::with_schemas(vec![schema]));
        let version_repo = Arc::new(StubVersionRepository::with_versions(vec![v1]));
        let uc = RegisterVersionUseCase::new(schema_repo, version_repo);

        let input = RegisterVersionInput {
            tenant_id: "tenant-a".to_string(),
            name: "grpc-api".to_string(),
            content: protobuf_content_v2_breaking(),
            registered_by: "user-001".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let version = result.unwrap();
        assert!(version.breaking_changes);
        assert!(version
            .breaking_change_details
            .iter()
            .any(|bc| bc.change_type == "field_removed"));
    }

    #[tokio::test]
    async fn compatible_changes_do_not_flag_breaking() {
        let schema = make_schema("tenant-api", SchemaType::OpenApi);
        let v1 = make_version("tenant-api", 1, SchemaType::OpenApi, &openapi_content_v1());

        let schema_repo = Arc::new(StubSchemaRepository::with_schemas(vec![schema]));
        let version_repo = Arc::new(StubVersionRepository::with_versions(vec![v1]));
        let uc = RegisterVersionUseCase::new(schema_repo, version_repo);

        let input = RegisterVersionInput {
            tenant_id: "tenant-a".to_string(),
            name: "tenant-api".to_string(),
            content: openapi_content_v2_compatible(),
            registered_by: "user-001".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let version = result.unwrap();
        assert!(!version.breaking_changes);
        assert!(version.breaking_change_details.is_empty());
    }

    #[tokio::test]
    async fn no_previous_version_means_no_breaking_changes() {
        let schema = make_schema("new-api", SchemaType::OpenApi);

        let schema_repo = Arc::new(StubSchemaRepository::with_schemas(vec![schema]));
        let version_repo = Arc::new(StubVersionRepository::new());
        let uc = RegisterVersionUseCase::new(schema_repo, version_repo);

        let input = RegisterVersionInput {
            tenant_id: "tenant-a".to_string(),
            name: "new-api".to_string(),
            content: openapi_content_v1(),
            registered_by: "user-001".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        assert!(!result.unwrap().breaking_changes);
    }

    #[tokio::test]
    async fn not_found_returns_error() {
        let schema_repo = Arc::new(StubSchemaRepository::new());
        let version_repo = Arc::new(StubVersionRepository::new());
        let uc = RegisterVersionUseCase::new(schema_repo, version_repo);

        let input = RegisterVersionInput {
            tenant_id: "tenant-a".to_string(),
            name: "nonexistent".to_string(),
            content: "openapi: 3.0.3".to_string(),
            registered_by: "user-001".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(matches!(
            result,
            Err(RegisterVersionError::NotFound(ref name)) if name == "nonexistent"
        ));
    }

    #[tokio::test]
    async fn repository_error_returns_internal() {
        let schema_repo = Arc::new(StubSchemaRepository::failing());
        let version_repo = Arc::new(StubVersionRepository::new());
        let uc = RegisterVersionUseCase::new(schema_repo, version_repo);

        let input = RegisterVersionInput {
            tenant_id: "tenant-a".to_string(),
            name: "tenant-api".to_string(),
            content: "openapi: 3.0.3".to_string(),
            registered_by: "user-001".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(matches!(result, Err(RegisterVersionError::Internal(_))));
    }

    #[tokio::test]
    async fn publisher_failure_does_not_fail_version_registration() {
        let schema = make_schema("tenant-api", SchemaType::OpenApi);
        let v1 = make_version("tenant-api", 1, SchemaType::OpenApi, "openapi: 3.0.3");

        let schema_repo = Arc::new(StubSchemaRepository::with_schemas(vec![schema]));
        let version_repo = Arc::new(StubVersionRepository::with_versions(vec![v1]));
        let publisher = Arc::new(StubEventPublisher::failing());
        let uc =
            RegisterVersionUseCase::with_publisher(schema_repo, version_repo.clone(), publisher);

        let input = RegisterVersionInput {
            tenant_id: "tenant-a".to_string(),
            name: "tenant-api".to_string(),
            content: "openapi: 3.0.3\nversion: 2".to_string(),
            registered_by: "user-001".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(
            result.is_ok(),
            "version registration should succeed even if publisher fails"
        );
    }
}

// ===========================================================================
// GetSchemaUseCase tests
// ===========================================================================

mod get_schema {
    use super::*;
    use k1s0_api_registry_server::usecase::get_schema::{GetSchemaError, GetSchemaUseCase};

    #[tokio::test]
    async fn found_returns_schema_with_latest_content() {
        let schema = make_schema("tenant-api", SchemaType::OpenApi);
        let v1 = make_version("tenant-api", 1, SchemaType::OpenApi, "openapi: 3.0.3");

        let schema_repo = Arc::new(StubSchemaRepository::with_schemas(vec![schema]));
        let version_repo = Arc::new(StubVersionRepository::with_versions(vec![v1]));
        let uc = GetSchemaUseCase::new(schema_repo, version_repo);

        let result = uc.execute("tenant-a", "tenant-api").await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.schema.name, "tenant-api");
        assert!(output.latest_content.is_some());
        assert_eq!(output.latest_content.unwrap().version, 1);
    }

    #[tokio::test]
    async fn returns_latest_version_not_earliest() {
        let schema = make_schema_with_versions("tenant-api", 3, 3);
        let v1 = make_version("tenant-api", 1, SchemaType::OpenApi, "v1");
        let v2 = make_version("tenant-api", 2, SchemaType::OpenApi, "v2");
        let v3 = make_version("tenant-api", 3, SchemaType::OpenApi, "v3");

        let schema_repo = Arc::new(StubSchemaRepository::with_schemas(vec![schema]));
        let version_repo = Arc::new(StubVersionRepository::with_versions(vec![v1, v2, v3]));
        let uc = GetSchemaUseCase::new(schema_repo, version_repo);

        let result = uc.execute("tenant-a", "tenant-api").await.unwrap();
        assert_eq!(result.latest_content.unwrap().version, 3);
    }

    #[tokio::test]
    async fn not_found_returns_error() {
        let schema_repo = Arc::new(StubSchemaRepository::new());
        let version_repo = Arc::new(StubVersionRepository::new());
        let uc = GetSchemaUseCase::new(schema_repo, version_repo);

        let result = uc.execute("tenant-a", "nonexistent").await;
        assert!(matches!(
            result,
            Err(GetSchemaError::NotFound(ref name)) if name == "nonexistent"
        ));
    }

    #[tokio::test]
    async fn repository_error_returns_internal() {
        let schema_repo = Arc::new(StubSchemaRepository::failing());
        let version_repo = Arc::new(StubVersionRepository::new());
        let uc = GetSchemaUseCase::new(schema_repo, version_repo);

        let result = uc.execute("tenant-a", "any").await;
        assert!(matches!(result, Err(GetSchemaError::Internal(_))));
    }
}

// ===========================================================================
// GetSchemaVersionUseCase tests
// ===========================================================================

mod get_schema_version {
    use super::*;
    use k1s0_api_registry_server::usecase::get_schema_version::{
        GetSchemaVersionError, GetSchemaVersionUseCase,
    };

    #[tokio::test]
    async fn found_returns_version() {
        let v2 = make_version("tenant-api", 2, SchemaType::OpenApi, "openapi: 3.0.3");
        let version_repo = Arc::new(StubVersionRepository::with_versions(vec![v2]));
        let uc = GetSchemaVersionUseCase::new(version_repo);

        let result = uc.execute("tenant-a", "tenant-api", 2).await;
        assert!(result.is_ok());
        let version = result.unwrap();
        assert_eq!(version.name, "tenant-api");
        assert_eq!(version.version, 2);
    }

    #[tokio::test]
    async fn not_found_returns_error() {
        let version_repo = Arc::new(StubVersionRepository::new());
        let uc = GetSchemaVersionUseCase::new(version_repo);

        let result = uc.execute("tenant-a", "tenant-api", 99).await;
        assert!(matches!(
            result,
            Err(GetSchemaVersionError::NotFound { ref name, version }) if name == "tenant-api" && version == 99
        ));
    }

    #[tokio::test]
    async fn repository_error_returns_internal() {
        let version_repo = Arc::new(StubVersionRepository::failing());
        let uc = GetSchemaVersionUseCase::new(version_repo);

        let result = uc.execute("tenant-a", "any", 1).await;
        assert!(matches!(result, Err(GetSchemaVersionError::Internal(_))));
    }
}

// ===========================================================================
// ListSchemasUseCase tests
// ===========================================================================

mod list_schemas {
    use super::*;
    use k1s0_api_registry_server::usecase::list_schemas::{
        ListSchemasError, ListSchemasInput, ListSchemasUseCase,
    };

    #[tokio::test]
    async fn returns_all_schemas() {
        let schemas = vec![
            make_schema("api-1", SchemaType::OpenApi),
            make_schema("api-2", SchemaType::Protobuf),
            make_schema("api-3", SchemaType::OpenApi),
        ];
        let schema_repo = Arc::new(StubSchemaRepository::with_schemas(schemas));
        let uc = ListSchemasUseCase::new(schema_repo);

        let input = ListSchemasInput {
            tenant_id: "tenant-a".to_string(),
            schema_type: None,
            page: 1,
            page_size: 20,
        };
        let output = uc.execute(&input).await.unwrap();
        assert_eq!(output.total_count, 3);
        assert_eq!(output.schemas.len(), 3);
        assert!(!output.has_next);
    }

    #[tokio::test]
    async fn filters_by_schema_type() {
        let schemas = vec![
            make_schema("api-1", SchemaType::OpenApi),
            make_schema("api-2", SchemaType::Protobuf),
            make_schema("api-3", SchemaType::OpenApi),
        ];
        let schema_repo = Arc::new(StubSchemaRepository::with_schemas(schemas));
        let uc = ListSchemasUseCase::new(schema_repo);

        let input = ListSchemasInput {
            tenant_id: "tenant-a".to_string(),
            schema_type: Some("openapi".to_string()),
            page: 1,
            page_size: 20,
        };
        let output = uc.execute(&input).await.unwrap();
        assert_eq!(output.total_count, 2);
        assert!(output
            .schemas
            .iter()
            .all(|s| s.schema_type == SchemaType::OpenApi));
    }

    #[tokio::test]
    async fn pagination_has_next() {
        let schemas: Vec<_> = (0..5)
            .map(|i| make_schema(&format!("api-{}", i), SchemaType::OpenApi))
            .collect();
        let schema_repo = Arc::new(StubSchemaRepository::with_schemas(schemas));
        let uc = ListSchemasUseCase::new(schema_repo);

        let input = ListSchemasInput {
            tenant_id: "tenant-a".to_string(),
            schema_type: None,
            page: 1,
            page_size: 2,
        };
        let output = uc.execute(&input).await.unwrap();
        assert_eq!(output.total_count, 5);
        assert_eq!(output.schemas.len(), 2);
        assert!(output.has_next);
        assert_eq!(output.page, 1);
        assert_eq!(output.page_size, 2);
    }

    #[tokio::test]
    async fn pagination_last_page() {
        let schemas: Vec<_> = (0..5)
            .map(|i| make_schema(&format!("api-{}", i), SchemaType::OpenApi))
            .collect();
        let schema_repo = Arc::new(StubSchemaRepository::with_schemas(schemas));
        let uc = ListSchemasUseCase::new(schema_repo);

        let input = ListSchemasInput {
            tenant_id: "tenant-a".to_string(),
            schema_type: None,
            page: 3,
            page_size: 2,
        };
        let output = uc.execute(&input).await.unwrap();
        assert_eq!(output.total_count, 5);
        assert_eq!(output.schemas.len(), 1);
        assert!(!output.has_next);
    }

    #[tokio::test]
    async fn empty_result() {
        let schema_repo = Arc::new(StubSchemaRepository::new());
        let uc = ListSchemasUseCase::new(schema_repo);

        let input = ListSchemasInput {
            tenant_id: "tenant-a".to_string(),
            schema_type: None,
            page: 1,
            page_size: 20,
        };
        let output = uc.execute(&input).await.unwrap();
        assert_eq!(output.total_count, 0);
        assert!(output.schemas.is_empty());
        assert!(!output.has_next);
    }

    #[tokio::test]
    async fn repository_error_propagates() {
        let schema_repo = Arc::new(StubSchemaRepository::failing());
        let uc = ListSchemasUseCase::new(schema_repo);

        let input = ListSchemasInput {
            tenant_id: "tenant-a".to_string(),
            schema_type: None,
            page: 1,
            page_size: 20,
        };
        let result = uc.execute(&input).await;
        assert!(matches!(result, Err(ListSchemasError::Internal(_))));
    }
}

// ===========================================================================
// ListVersionsUseCase tests
// ===========================================================================

mod list_versions {
    use super::*;
    use k1s0_api_registry_server::usecase::list_versions::{
        ListVersionsError, ListVersionsInput, ListVersionsUseCase,
    };

    #[tokio::test]
    async fn success_returns_versions_for_schema() {
        let schema = make_schema_with_versions("tenant-api", 2, 2);
        let v1 = make_version("tenant-api", 1, SchemaType::OpenApi, "v1");
        let v2 = make_version("tenant-api", 2, SchemaType::OpenApi, "v2");

        let schema_repo = Arc::new(StubSchemaRepository::with_schemas(vec![schema]));
        let version_repo = Arc::new(StubVersionRepository::with_versions(vec![v1, v2]));
        let uc = ListVersionsUseCase::new(schema_repo, version_repo);

        let input = ListVersionsInput {
            tenant_id: "tenant-a".to_string(),
            name: "tenant-api".to_string(),
            page: 1,
            page_size: 20,
        };
        let output = uc.execute(&input).await.unwrap();
        assert_eq!(output.name, "tenant-api");
        assert_eq!(output.versions.len(), 2);
        assert_eq!(output.total_count, 2);
    }

    #[tokio::test]
    async fn only_returns_versions_for_requested_schema() {
        let schema_a = make_schema("api-a", SchemaType::OpenApi);
        let v_a = make_version("api-a", 1, SchemaType::OpenApi, "a");
        let v_b = make_version("api-b", 1, SchemaType::OpenApi, "b");

        let schema_repo = Arc::new(StubSchemaRepository::with_schemas(vec![schema_a]));
        let version_repo = Arc::new(StubVersionRepository::with_versions(vec![v_a, v_b]));
        let uc = ListVersionsUseCase::new(schema_repo, version_repo);

        let input = ListVersionsInput {
            tenant_id: "tenant-a".to_string(),
            name: "api-a".to_string(),
            page: 1,
            page_size: 20,
        };
        let output = uc.execute(&input).await.unwrap();
        assert_eq!(output.versions.len(), 1);
        assert!(output.versions.iter().all(|v| v.name == "api-a"));
    }

    #[tokio::test]
    async fn not_found_returns_error() {
        let schema_repo = Arc::new(StubSchemaRepository::new());
        let version_repo = Arc::new(StubVersionRepository::new());
        let uc = ListVersionsUseCase::new(schema_repo, version_repo);

        let input = ListVersionsInput {
            tenant_id: "tenant-a".to_string(),
            name: "nonexistent".to_string(),
            page: 1,
            page_size: 20,
        };
        let result = uc.execute(&input).await;
        assert!(matches!(
            result,
            Err(ListVersionsError::NotFound(ref name)) if name == "nonexistent"
        ));
    }

    #[tokio::test]
    async fn repository_error_returns_internal() {
        let schema_repo = Arc::new(StubSchemaRepository::failing());
        let version_repo = Arc::new(StubVersionRepository::new());
        let uc = ListVersionsUseCase::new(schema_repo, version_repo);

        let input = ListVersionsInput {
            tenant_id: "tenant-a".to_string(),
            name: "any".to_string(),
            page: 1,
            page_size: 20,
        };
        let result = uc.execute(&input).await;
        assert!(matches!(result, Err(ListVersionsError::Internal(_))));
    }
}

// ===========================================================================
// CheckCompatibilityUseCase tests
// ===========================================================================

mod check_compatibility {
    use super::*;
    use k1s0_api_registry_server::usecase::check_compatibility::{
        CheckCompatibilityError, CheckCompatibilityInput, CheckCompatibilityUseCase,
    };

    #[tokio::test]
    async fn compatible_change_returns_true() {
        let schema = make_schema("tenant-api", SchemaType::OpenApi);
        let v1 = make_version("tenant-api", 1, SchemaType::OpenApi, &openapi_content_v1());

        let schema_repo = Arc::new(StubSchemaRepository::with_schemas(vec![schema]));
        let version_repo = Arc::new(StubVersionRepository::with_versions(vec![v1]));
        let uc = CheckCompatibilityUseCase::new(schema_repo, version_repo);

        let input = CheckCompatibilityInput {
            tenant_id: "tenant-a".to_string(),
            name: "tenant-api".to_string(),
            content: openapi_content_v2_compatible(),
            base_version: None,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.result.compatible);
        assert!(output.result.breaking_changes.is_empty());
        assert!(!output.result.non_breaking_changes.is_empty());
        assert_eq!(output.base_version, 1);
    }

    #[tokio::test]
    async fn breaking_change_returns_false() {
        let schema = make_schema("tenant-api", SchemaType::OpenApi);
        let v1 = make_version("tenant-api", 1, SchemaType::OpenApi, &openapi_content_v1());

        let schema_repo = Arc::new(StubSchemaRepository::with_schemas(vec![schema]));
        let version_repo = Arc::new(StubVersionRepository::with_versions(vec![v1]));
        let uc = CheckCompatibilityUseCase::new(schema_repo, version_repo);

        let input = CheckCompatibilityInput {
            tenant_id: "tenant-a".to_string(),
            name: "tenant-api".to_string(),
            content: openapi_content_v2_breaking(),
            base_version: None,
        };
        let result = uc.execute(&input).await.unwrap();
        assert!(!result.result.compatible);
        assert!(!result.result.breaking_changes.is_empty());
    }

    #[tokio::test]
    async fn explicit_base_version() {
        let schema = make_schema_with_versions("tenant-api", 3, 3);
        let v1 = make_version("tenant-api", 1, SchemaType::OpenApi, &openapi_content_v1());
        let v2 = make_version(
            "tenant-api",
            2,
            SchemaType::OpenApi,
            &openapi_content_v2_compatible(),
        );

        let schema_repo = Arc::new(StubSchemaRepository::with_schemas(vec![schema]));
        let version_repo = Arc::new(StubVersionRepository::with_versions(vec![v1, v2]));
        let uc = CheckCompatibilityUseCase::new(schema_repo, version_repo);

        let input = CheckCompatibilityInput {
            tenant_id: "tenant-a".to_string(),
            name: "tenant-api".to_string(),
            content: openapi_content_v2_breaking(),
            base_version: Some(2),
        };
        let result = uc.execute(&input).await.unwrap();
        assert_eq!(result.base_version, 2);
    }

    #[tokio::test]
    async fn protobuf_compatibility_check() {
        let mut schema = make_schema("grpc-api", SchemaType::Protobuf);
        schema.schema_type = SchemaType::Protobuf;
        let v1 = make_version("grpc-api", 1, SchemaType::Protobuf, &protobuf_content_v1());

        let schema_repo = Arc::new(StubSchemaRepository::with_schemas(vec![schema]));
        let version_repo = Arc::new(StubVersionRepository::with_versions(vec![v1]));
        let uc = CheckCompatibilityUseCase::new(schema_repo, version_repo);

        let input = CheckCompatibilityInput {
            tenant_id: "tenant-a".to_string(),
            name: "grpc-api".to_string(),
            content: protobuf_content_v2_breaking(),
            base_version: None,
        };
        let result = uc.execute(&input).await.unwrap();
        assert!(!result.result.compatible);
    }

    #[tokio::test]
    async fn schema_not_found_returns_error() {
        let schema_repo = Arc::new(StubSchemaRepository::new());
        let version_repo = Arc::new(StubVersionRepository::new());
        let uc = CheckCompatibilityUseCase::new(schema_repo, version_repo);

        let input = CheckCompatibilityInput {
            tenant_id: "tenant-a".to_string(),
            name: "nonexistent".to_string(),
            content: "openapi: 3.0.3".to_string(),
            base_version: None,
        };
        let result = uc.execute(&input).await;
        assert!(matches!(
            result,
            Err(CheckCompatibilityError::SchemaNotFound(ref name)) if name == "nonexistent"
        ));
    }

    #[tokio::test]
    async fn version_not_found_returns_error() {
        let schema = make_schema_with_versions("tenant-api", 5, 5);
        let schema_repo = Arc::new(StubSchemaRepository::with_schemas(vec![schema]));
        let version_repo = Arc::new(StubVersionRepository::new());
        let uc = CheckCompatibilityUseCase::new(schema_repo, version_repo);

        let input = CheckCompatibilityInput {
            tenant_id: "tenant-a".to_string(),
            name: "tenant-api".to_string(),
            content: "openapi: 3.0.3".to_string(),
            base_version: None,
        };
        let result = uc.execute(&input).await;
        assert!(matches!(
            result,
            Err(CheckCompatibilityError::VersionNotFound { .. })
        ));
    }

    #[tokio::test]
    async fn repository_error_returns_internal() {
        let schema_repo = Arc::new(StubSchemaRepository::failing());
        let version_repo = Arc::new(StubVersionRepository::new());
        let uc = CheckCompatibilityUseCase::new(schema_repo, version_repo);

        let input = CheckCompatibilityInput {
            tenant_id: "tenant-a".to_string(),
            name: "any".to_string(),
            content: "openapi: 3.0.3".to_string(),
            base_version: None,
        };
        let result = uc.execute(&input).await;
        assert!(matches!(result, Err(CheckCompatibilityError::Internal(_))));
    }
}

// ===========================================================================
// GetDiffUseCase tests
// ===========================================================================

mod get_diff {
    use super::*;
    use k1s0_api_registry_server::usecase::get_diff::{GetDiffError, GetDiffInput, GetDiffUseCase};

    #[tokio::test]
    async fn success_returns_diff_between_versions() {
        let schema = make_schema_with_versions("tenant-api", 2, 2);
        let v1 = make_version("tenant-api", 1, SchemaType::OpenApi, &openapi_content_v1());
        let v2 = make_version(
            "tenant-api",
            2,
            SchemaType::OpenApi,
            &openapi_content_v2_compatible(),
        );

        let schema_repo = Arc::new(StubSchemaRepository::with_schemas(vec![schema]));
        let version_repo = Arc::new(StubVersionRepository::with_versions(vec![v1, v2]));
        let uc = GetDiffUseCase::new(schema_repo, version_repo);

        let input = GetDiffInput {
            tenant_id: "tenant-a".to_string(),
            name: "tenant-api".to_string(),
            from_version: Some(1),
            to_version: Some(2),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.name, "tenant-api");
        assert_eq!(output.from_version, 1);
        assert_eq!(output.to_version, 2);
        assert!(!output.breaking_changes);
        // New path /api/v1/products added
        assert!(!output.diff.added.is_empty());
    }

    #[tokio::test]
    async fn diff_with_breaking_changes() {
        let schema = make_schema_with_versions("tenant-api", 2, 2);
        let v1 = make_version("tenant-api", 1, SchemaType::OpenApi, &openapi_content_v1());
        let v2 = make_version(
            "tenant-api",
            2,
            SchemaType::OpenApi,
            &openapi_content_v2_breaking(),
        );

        let schema_repo = Arc::new(StubSchemaRepository::with_schemas(vec![schema]));
        let version_repo = Arc::new(StubVersionRepository::with_versions(vec![v1, v2]));
        let uc = GetDiffUseCase::new(schema_repo, version_repo);

        let input = GetDiffInput {
            tenant_id: "tenant-a".to_string(),
            name: "tenant-api".to_string(),
            from_version: Some(1),
            to_version: Some(2),
        };
        let result = uc.execute(&input).await.unwrap();
        assert!(result.breaking_changes);
    }

    #[tokio::test]
    async fn validation_error_from_gte_to() {
        let schema = make_schema_with_versions("tenant-api", 3, 3);
        let schema_repo = Arc::new(StubSchemaRepository::with_schemas(vec![schema]));
        let version_repo = Arc::new(StubVersionRepository::new());
        let uc = GetDiffUseCase::new(schema_repo, version_repo);

        let input = GetDiffInput {
            tenant_id: "tenant-a".to_string(),
            name: "tenant-api".to_string(),
            from_version: Some(3),
            to_version: Some(2),
        };
        let result = uc.execute(&input).await;
        assert!(matches!(result, Err(GetDiffError::ValidationError(_))));
    }

    #[tokio::test]
    async fn schema_not_found_returns_error() {
        let schema_repo = Arc::new(StubSchemaRepository::new());
        let version_repo = Arc::new(StubVersionRepository::new());
        let uc = GetDiffUseCase::new(schema_repo, version_repo);

        let input = GetDiffInput {
            tenant_id: "tenant-a".to_string(),
            name: "nonexistent".to_string(),
            from_version: Some(1),
            to_version: Some(2),
        };
        let result = uc.execute(&input).await;
        assert!(matches!(
            result,
            Err(GetDiffError::SchemaNotFound(ref name)) if name == "nonexistent"
        ));
    }

    #[tokio::test]
    async fn version_not_found_returns_error() {
        let schema = make_schema_with_versions("tenant-api", 3, 3);
        let schema_repo = Arc::new(StubSchemaRepository::with_schemas(vec![schema]));
        let version_repo = Arc::new(StubVersionRepository::new());
        let uc = GetDiffUseCase::new(schema_repo, version_repo);

        let input = GetDiffInput {
            tenant_id: "tenant-a".to_string(),
            name: "tenant-api".to_string(),
            from_version: Some(1),
            to_version: Some(2),
        };
        let result = uc.execute(&input).await;
        assert!(matches!(result, Err(GetDiffError::VersionNotFound { .. })));
    }

    #[tokio::test]
    async fn repository_error_returns_internal() {
        let schema_repo = Arc::new(StubSchemaRepository::failing());
        let version_repo = Arc::new(StubVersionRepository::new());
        let uc = GetDiffUseCase::new(schema_repo, version_repo);

        let input = GetDiffInput {
            tenant_id: "tenant-a".to_string(),
            name: "any".to_string(),
            from_version: Some(1),
            to_version: Some(2),
        };
        let result = uc.execute(&input).await;
        assert!(matches!(result, Err(GetDiffError::Internal(_))));
    }

    #[tokio::test]
    async fn protobuf_diff() {
        let mut schema = make_schema_with_versions("grpc-api", 2, 2);
        schema.schema_type = SchemaType::Protobuf;
        let v1 = make_version("grpc-api", 1, SchemaType::Protobuf, &protobuf_content_v1());
        let v2_content = "syntax = \"proto3\";\nmessage User {\n  string id = 1;\n  string name = 2;\n  string email = 3;\n  string phone = 4;\n}\n";
        let v2 = make_version("grpc-api", 2, SchemaType::Protobuf, v2_content);

        let schema_repo = Arc::new(StubSchemaRepository::with_schemas(vec![schema]));
        let version_repo = Arc::new(StubVersionRepository::with_versions(vec![v1, v2]));
        let uc = GetDiffUseCase::new(schema_repo, version_repo);

        let input = GetDiffInput {
            tenant_id: "tenant-a".to_string(),
            name: "grpc-api".to_string(),
            from_version: Some(1),
            to_version: Some(2),
        };
        let result = uc.execute(&input).await.unwrap();
        assert!(!result.breaking_changes);
        assert!(!result.diff.added.is_empty());
    }
}

// ===========================================================================
// DeleteVersionUseCase tests
// ===========================================================================

mod delete_version {
    use super::*;
    use k1s0_api_registry_server::usecase::delete_version::{
        DeleteVersionError, DeleteVersionUseCase,
    };

    #[tokio::test]
    async fn success_deletes_version_and_updates_schema() {
        let schema = make_schema_with_versions("tenant-api", 3, 3);
        let v1 = make_version("tenant-api", 1, SchemaType::OpenApi, "v1");
        let v2 = make_version("tenant-api", 2, SchemaType::OpenApi, "v2");
        let v3 = make_version("tenant-api", 3, SchemaType::OpenApi, "v3");

        let schema_repo = Arc::new(StubSchemaRepository::with_schemas(vec![schema]));
        let version_repo = Arc::new(StubVersionRepository::with_versions(vec![v1, v2, v3]));
        let uc = DeleteVersionUseCase::new(schema_repo.clone(), version_repo.clone());

        let result = uc
            .execute("tenant-a", "tenant-api", 1, Some("admin".to_string()))
            .await;
        assert!(result.is_ok());

        // Verify version deleted
        let versions = version_repo.versions.read().await;
        assert_eq!(versions.len(), 2);
        assert!(!versions.iter().any(|v| v.version == 1));

        // Verify schema metadata updated
        let schemas = schema_repo.schemas.read().await;
        assert_eq!(schemas[0].version_count, 2);
        assert_eq!(schemas[0].latest_version, 3);
    }

    #[tokio::test]
    async fn cannot_delete_only_remaining_version() {
        let schema = make_schema("tenant-api", SchemaType::OpenApi);
        let v1 = make_version("tenant-api", 1, SchemaType::OpenApi, "v1");

        let schema_repo = Arc::new(StubSchemaRepository::with_schemas(vec![schema]));
        let version_repo = Arc::new(StubVersionRepository::with_versions(vec![v1]));
        let uc = DeleteVersionUseCase::new(schema_repo, version_repo);

        let result = uc.execute("tenant-a", "tenant-api", 1, None).await;
        assert!(matches!(
            result,
            Err(DeleteVersionError::CannotDeleteLatest(ref name)) if name == "tenant-api"
        ));
    }

    #[tokio::test]
    async fn schema_not_found_returns_error() {
        let schema_repo = Arc::new(StubSchemaRepository::new());
        let version_repo = Arc::new(StubVersionRepository::new());
        let uc = DeleteVersionUseCase::new(schema_repo, version_repo);

        let result = uc.execute("tenant-a", "nonexistent", 1, None).await;
        assert!(matches!(
            result,
            Err(DeleteVersionError::SchemaNotFound(ref name)) if name == "nonexistent"
        ));
    }

    #[tokio::test]
    async fn version_not_found_returns_error() {
        let schema = make_schema_with_versions("tenant-api", 3, 3);
        let v1 = make_version("tenant-api", 1, SchemaType::OpenApi, "v1");
        let v2 = make_version("tenant-api", 2, SchemaType::OpenApi, "v2");
        let v3 = make_version("tenant-api", 3, SchemaType::OpenApi, "v3");

        let schema_repo = Arc::new(StubSchemaRepository::with_schemas(vec![schema]));
        let version_repo = Arc::new(StubVersionRepository::with_versions(vec![v1, v2, v3]));
        let uc = DeleteVersionUseCase::new(schema_repo, version_repo);

        let result = uc.execute("tenant-a", "tenant-api", 99, None).await;
        assert!(matches!(
            result,
            Err(DeleteVersionError::VersionNotFound { ref name, version }) if name == "tenant-api" && version == 99
        ));
    }

    #[tokio::test]
    async fn repository_error_returns_internal() {
        let schema_repo = Arc::new(StubSchemaRepository::failing());
        let version_repo = Arc::new(StubVersionRepository::new());
        let uc = DeleteVersionUseCase::new(schema_repo, version_repo);

        let result = uc.execute("tenant-a", "any", 1, None).await;
        assert!(matches!(result, Err(DeleteVersionError::Internal(_))));
    }

    #[tokio::test]
    async fn publisher_failure_does_not_fail_delete() {
        let schema = make_schema_with_versions("tenant-api", 2, 2);
        let v1 = make_version("tenant-api", 1, SchemaType::OpenApi, "v1");
        let v2 = make_version("tenant-api", 2, SchemaType::OpenApi, "v2");

        let schema_repo = Arc::new(StubSchemaRepository::with_schemas(vec![schema]));
        let version_repo = Arc::new(StubVersionRepository::with_versions(vec![v1, v2]));
        let publisher = Arc::new(StubEventPublisher::failing());
        let uc = DeleteVersionUseCase::with_publisher(schema_repo, version_repo.clone(), publisher);

        let result = uc.execute("tenant-a", "tenant-api", 1, None).await;
        assert!(
            result.is_ok(),
            "delete should succeed even if publisher fails"
        );
    }
}

// ===========================================================================
// Full lifecycle test
// ===========================================================================

mod lifecycle {
    use super::*;
    use k1s0_api_registry_server::usecase::check_compatibility::{
        CheckCompatibilityInput, CheckCompatibilityUseCase,
    };
    use k1s0_api_registry_server::usecase::delete_version::DeleteVersionUseCase;
    use k1s0_api_registry_server::usecase::get_diff::{GetDiffInput, GetDiffUseCase};
    use k1s0_api_registry_server::usecase::get_schema::GetSchemaUseCase;
    use k1s0_api_registry_server::usecase::get_schema_version::GetSchemaVersionUseCase;
    use k1s0_api_registry_server::usecase::list_schemas::{ListSchemasInput, ListSchemasUseCase};
    use k1s0_api_registry_server::usecase::list_versions::{
        ListVersionsInput, ListVersionsUseCase,
    };
    use k1s0_api_registry_server::usecase::register_schema::{
        RegisterSchemaInput, RegisterSchemaUseCase,
    };
    use k1s0_api_registry_server::usecase::register_version::{
        RegisterVersionInput, RegisterVersionUseCase,
    };

    #[tokio::test]
    async fn full_schema_lifecycle() {
        let schema_repo: Arc<StubSchemaRepository> = Arc::new(StubSchemaRepository::new());
        let version_repo: Arc<StubVersionRepository> = Arc::new(StubVersionRepository::new());

        // 1. Register a new schema
        let register_uc = RegisterSchemaUseCase::new(schema_repo.clone(), version_repo.clone());
        let reg_input = RegisterSchemaInput {
            tenant_id: "tenant-a".to_string(),
            name: "lifecycle-api".to_string(),
            description: "Lifecycle test API".to_string(),
            schema_type: SchemaType::OpenApi,
            content: openapi_content_v1(),
            registered_by: "admin".to_string(),
        };
        let v1 = register_uc.execute(&reg_input).await.unwrap();
        assert_eq!(v1.version, 1);
        assert_eq!(v1.name, "lifecycle-api");

        // 2. Get the schema
        let get_uc = GetSchemaUseCase::new(schema_repo.clone(), version_repo.clone());
        // テナント分離のため tenant_id を渡す
        let schema_output = get_uc.execute("tenant-a", "lifecycle-api").await.unwrap();
        assert_eq!(schema_output.schema.name, "lifecycle-api");
        assert_eq!(schema_output.schema.latest_version, 1);

        // 3. Check compatibility before registering v2
        let compat_uc = CheckCompatibilityUseCase::new(schema_repo.clone(), version_repo.clone());
        let compat_input = CheckCompatibilityInput {
            // テナント分離のため tenant_id を設定
            tenant_id: "tenant-a".to_string(),
            name: "lifecycle-api".to_string(),
            content: openapi_content_v2_compatible(),
            base_version: None,
        };
        let compat = compat_uc.execute(&compat_input).await.unwrap();
        assert!(compat.result.compatible);

        // 4. Register v2 (compatible change)
        let version_uc = RegisterVersionUseCase::new(schema_repo.clone(), version_repo.clone());
        let ver_input = RegisterVersionInput {
            // テナント分離のため tenant_id を設定
            tenant_id: "tenant-a".to_string(),
            name: "lifecycle-api".to_string(),
            content: openapi_content_v2_compatible(),
            registered_by: "admin".to_string(),
        };
        let v2 = version_uc.execute(&ver_input).await.unwrap();
        assert_eq!(v2.version, 2);
        assert!(!v2.breaking_changes);

        // 5. List schemas
        let list_uc = ListSchemasUseCase::new(schema_repo.clone());
        let list_output = list_uc
            .execute(&ListSchemasInput {
                // テナント分離のため tenant_id を設定
                tenant_id: "tenant-a".to_string(),
                schema_type: None,
                page: 1,
                page_size: 20,
            })
            .await
            .unwrap();
        assert_eq!(list_output.total_count, 1);

        // 6. List versions
        let list_ver_uc = ListVersionsUseCase::new(schema_repo.clone(), version_repo.clone());
        let list_ver_output = list_ver_uc
            .execute(&ListVersionsInput {
                // テナント分離のため tenant_id を設定
                tenant_id: "tenant-a".to_string(),
                name: "lifecycle-api".to_string(),
                page: 1,
                page_size: 20,
            })
            .await
            .unwrap();
        assert_eq!(list_ver_output.versions.len(), 2);

        // 7. Get specific version
        let get_ver_uc = GetSchemaVersionUseCase::new(version_repo.clone());
        // テナント分離のため tenant_id を渡す
        let ver = get_ver_uc
            .execute("tenant-a", "lifecycle-api", 2)
            .await
            .unwrap();
        assert_eq!(ver.version, 2);

        // 8. Get diff between v1 and v2
        let diff_uc = GetDiffUseCase::new(schema_repo.clone(), version_repo.clone());
        let diff_output = diff_uc
            .execute(&GetDiffInput {
                // テナント分離のため tenant_id を設定
                tenant_id: "tenant-a".to_string(),
                name: "lifecycle-api".to_string(),
                from_version: Some(1),
                to_version: Some(2),
            })
            .await
            .unwrap();
        assert!(!diff_output.breaking_changes);
        assert!(!diff_output.diff.added.is_empty());

        // 9. Register v3 with breaking change
        let ver_input_v3 = RegisterVersionInput {
            // テナント分離のため tenant_id を設定
            tenant_id: "tenant-a".to_string(),
            name: "lifecycle-api".to_string(),
            content: openapi_content_v2_breaking(),
            registered_by: "admin".to_string(),
        };
        let v3 = version_uc.execute(&ver_input_v3).await.unwrap();
        assert_eq!(v3.version, 3);
        assert!(v3.breaking_changes);

        // 10. Delete v1
        let delete_uc = DeleteVersionUseCase::new(schema_repo.clone(), version_repo.clone());
        // テナント分離のため tenant_id を渡す
        let delete_result = delete_uc
            .execute("tenant-a", "lifecycle-api", 1, None)
            .await;
        assert!(delete_result.is_ok());

        // 11. Verify deletion
        // テナント分離のため tenant_id を渡す
        let get_deleted = get_ver_uc.execute("tenant-a", "lifecycle-api", 1).await;
        assert!(get_deleted.is_err());

        // 12. Verify remaining versions
        let remaining = list_ver_uc
            .execute(&ListVersionsInput {
                // テナント分離のため tenant_id を設定
                tenant_id: "tenant-a".to_string(),
                name: "lifecycle-api".to_string(),
                page: 1,
                page_size: 20,
            })
            .await
            .unwrap();
        assert_eq!(remaining.versions.len(), 2);
    }
}
