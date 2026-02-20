use async_trait::async_trait;
#[cfg(feature = "mock")]
use mockall::automock;
use std::time::Duration;
use tracing::{debug, error};

use crate::{
    config::SchemaRegistryConfig,
    error::SchemaRegistryError,
    schema::{
        parse_schema_type, RegisterSchemaRequest, RegisterSchemaResponse, RegisteredSchema,
        SchemaByIdResponse, SchemaType, SchemaVersionResponse,
    },
};

/// Schema Registry クライアントのインターフェース。
///
/// スキーマの登録・取得・バージョン管理・互換性チェックを提供する。
/// `mock` feature を有効にすると `MockSchemaRegistryClient` が生成される。
#[async_trait]
#[cfg_attr(feature = "mock", automock)]
pub trait SchemaRegistryClient: Send + Sync {
    /// スキーマをサブジェクトに登録し、スキーマ ID を返す。
    ///
    /// 同一スキーマが既に存在する場合は既存の ID を返す。
    async fn register_schema(
        &self,
        subject: &str,
        schema: &str,
        schema_type: SchemaType,
    ) -> Result<i32, SchemaRegistryError>;

    /// グローバルスキーマ ID でスキーマを取得する。
    async fn get_schema_by_id(
        &self,
        schema_id: i32,
    ) -> Result<RegisteredSchema, SchemaRegistryError>;

    /// サブジェクトの最新バージョンのスキーマを取得する。
    async fn get_latest_schema(
        &self,
        subject: &str,
    ) -> Result<RegisteredSchema, SchemaRegistryError>;

    /// サブジェクトの指定バージョンのスキーマを取得する。
    async fn get_schema_version(
        &self,
        subject: &str,
        version: i32,
    ) -> Result<RegisteredSchema, SchemaRegistryError>;

    /// 登録されているすべてのサブジェクト名を返す。
    async fn list_subjects(&self) -> Result<Vec<String>, SchemaRegistryError>;

    /// サブジェクトに登録されているすべてのバージョン番号を返す。
    async fn list_versions(&self, subject: &str) -> Result<Vec<i32>, SchemaRegistryError>;

    /// 指定スキーマがサブジェクトの現在の互換性設定を満たすか確認する。
    ///
    /// `true` の場合は互換性あり、`false` の場合は互換性なし。
    async fn check_compatibility(
        &self,
        subject: &str,
        schema: &str,
        schema_type: SchemaType,
    ) -> Result<bool, SchemaRegistryError>;

    /// サブジェクトを削除し、削除されたバージョン番号のリストを返す。
    async fn delete_subject(&self, subject: &str) -> Result<Vec<i32>, SchemaRegistryError>;

    /// Schema Registry サービスへの接続を確認する。
    async fn health_check(&self) -> Result<(), SchemaRegistryError>;
}

/// HTTP 経由で Confluent Schema Registry と通信する実装。
pub struct HttpSchemaRegistryClient {
    config: SchemaRegistryConfig,
    http_client: reqwest::Client,
}

impl HttpSchemaRegistryClient {
    /// 設定から HTTP クライアントを構築する。
    ///
    /// タイムアウトは `config.timeout_secs` の値を使用する。
    pub fn new(config: SchemaRegistryConfig) -> Result<Self, SchemaRegistryError> {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(|e| SchemaRegistryError::Unavailable(e.to_string()))?;

        Ok(Self {
            config,
            http_client,
        })
    }

    /// Schema Registry のベース URL を返す（末尾スラッシュなし）。
    fn base_url(&self) -> &str {
        self.config.url.trim_end_matches('/')
    }

    /// HTTP レスポンスのステータスコードを検査してエラーに変換する。
    ///
    /// - 404 → `SchemaNotFound`
    /// - その他の 4xx/5xx → `Unavailable`
    async fn check_response(
        response: reqwest::Response,
        subject: Option<&str>,
        version: Option<i32>,
    ) -> Result<reqwest::Response, SchemaRegistryError> {
        let status = response.status();
        if status.is_success() {
            return Ok(response);
        }

        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(SchemaRegistryError::SchemaNotFound {
                subject: subject.unwrap_or("unknown").to_string(),
                version,
            });
        }

        let body = response
            .text()
            .await
            .unwrap_or_else(|_| status.to_string());
        error!("Schema Registry returned error {}: {}", status, body);
        Err(SchemaRegistryError::Unavailable(format!(
            "status={}, body={}",
            status, body
        )))
    }
}

#[async_trait]
impl SchemaRegistryClient for HttpSchemaRegistryClient {
    async fn register_schema(
        &self,
        subject: &str,
        schema: &str,
        schema_type: SchemaType,
    ) -> Result<i32, SchemaRegistryError> {
        let url = format!("{}/subjects/{}/versions", self.base_url(), subject);
        let body = RegisterSchemaRequest {
            schema,
            schema_type: schema_type.as_str(),
        };

        debug!(
            "Registering schema: subject={}, type={}",
            subject, schema_type
        );

        let response = self
            .http_client
            .post(&url)
            .json(&body)
            .send()
            .await?;

        let response = Self::check_response(response, Some(subject), None).await?;
        let reg: RegisterSchemaResponse = response.json().await?;

        debug!("Schema registered: id={}", reg.id);
        Ok(reg.id)
    }

    async fn get_schema_by_id(
        &self,
        schema_id: i32,
    ) -> Result<RegisteredSchema, SchemaRegistryError> {
        let url = format!("{}/schemas/ids/{}", self.base_url(), schema_id);

        debug!("Fetching schema by id={}", schema_id);

        let response = self.http_client.get(&url).send().await?;
        let response = Self::check_response(response, None, None).await?;
        let data: SchemaByIdResponse = response.json().await?;

        let schema_type = parse_schema_type(&data.schema_type);
        Ok(RegisteredSchema {
            id: data.id.unwrap_or(schema_id),
            subject: data.subject.unwrap_or_default(),
            version: data.version.unwrap_or(0),
            schema: data.schema,
            schema_type,
        })
    }

    async fn get_latest_schema(
        &self,
        subject: &str,
    ) -> Result<RegisteredSchema, SchemaRegistryError> {
        let url = format!("{}/subjects/{}/versions/latest", self.base_url(), subject);

        debug!("Fetching latest schema: subject={}", subject);

        let response = self.http_client.get(&url).send().await?;
        let response = Self::check_response(response, Some(subject), None).await?;
        let data: SchemaVersionResponse = response.json().await?;

        let schema_type = parse_schema_type(&data.schema_type);
        Ok(RegisteredSchema {
            id: data.id,
            subject: data.subject,
            version: data.version,
            schema: data.schema,
            schema_type,
        })
    }

    async fn get_schema_version(
        &self,
        subject: &str,
        version: i32,
    ) -> Result<RegisteredSchema, SchemaRegistryError> {
        let url = format!(
            "{}/subjects/{}/versions/{}",
            self.base_url(),
            subject,
            version
        );

        debug!("Fetching schema: subject={}, version={}", subject, version);

        let response = self.http_client.get(&url).send().await?;
        let response = Self::check_response(response, Some(subject), Some(version)).await?;
        let data: SchemaVersionResponse = response.json().await?;

        let schema_type = parse_schema_type(&data.schema_type);
        Ok(RegisteredSchema {
            id: data.id,
            subject: data.subject,
            version: data.version,
            schema: data.schema,
            schema_type,
        })
    }

    async fn list_subjects(&self) -> Result<Vec<String>, SchemaRegistryError> {
        let url = format!("{}/subjects", self.base_url());

        debug!("Listing all subjects");

        let response = self.http_client.get(&url).send().await?;
        let response = Self::check_response(response, None, None).await?;
        let subjects: Vec<String> = response.json().await?;

        debug!("Found {} subjects", subjects.len());
        Ok(subjects)
    }

    async fn list_versions(&self, subject: &str) -> Result<Vec<i32>, SchemaRegistryError> {
        let url = format!("{}/subjects/{}/versions", self.base_url(), subject);

        debug!("Listing versions: subject={}", subject);

        let response = self.http_client.get(&url).send().await?;
        let response = Self::check_response(response, Some(subject), None).await?;
        let versions: Vec<i32> = response.json().await?;

        debug!("Subject {} has {} versions", subject, versions.len());
        Ok(versions)
    }

    async fn check_compatibility(
        &self,
        subject: &str,
        schema: &str,
        schema_type: SchemaType,
    ) -> Result<bool, SchemaRegistryError> {
        let url = format!(
            "{}/compatibility/subjects/{}/versions/latest",
            self.base_url(),
            subject
        );
        let body = RegisterSchemaRequest {
            schema,
            schema_type: schema_type.as_str(),
        };

        debug!(
            "Checking compatibility: subject={}, type={}",
            subject, schema_type
        );

        let response = self
            .http_client
            .post(&url)
            .json(&body)
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            // サブジェクトが未登録の場合は互換性チェック不要（登録可能）。
            debug!(
                "Subject {} not found, treating as compatible (new subject)",
                subject
            );
            return Ok(true);
        }

        let response = Self::check_response(response, Some(subject), None).await?;
        let result: serde_json::Value = response.json().await?;

        let is_compatible = result
            .get("is_compatible")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        debug!(
            "Compatibility check result: subject={}, is_compatible={}",
            subject, is_compatible
        );
        Ok(is_compatible)
    }

    async fn delete_subject(&self, subject: &str) -> Result<Vec<i32>, SchemaRegistryError> {
        let url = format!("{}/subjects/{}", self.base_url(), subject);

        debug!("Deleting subject={}", subject);

        let response = self.http_client.delete(&url).send().await?;
        let response = Self::check_response(response, Some(subject), None).await?;
        let versions: Vec<i32> = response.json().await?;

        debug!("Deleted subject {}: {} versions removed", subject, versions.len());
        Ok(versions)
    }

    async fn health_check(&self) -> Result<(), SchemaRegistryError> {
        let url = format!("{}/", self.base_url());

        debug!("Health check: url={}", url);

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| {
                error!("Schema Registry health check failed: {}", e);
                SchemaRegistryError::Unavailable(e.to_string())
            })?;

        if !response.status().is_success() {
            let status = response.status();
            error!("Schema Registry health check returned status {}", status);
            return Err(SchemaRegistryError::Unavailable(format!(
                "health check failed with status {}",
                status
            )));
        }

        debug!("Schema Registry is healthy");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- URL 構築テスト ----

    #[test]
    fn test_base_url_strips_trailing_slash() {
        let config = SchemaRegistryConfig::new("http://schema-registry:8081/");
        let client = HttpSchemaRegistryClient::new(config).unwrap();
        assert_eq!(client.base_url(), "http://schema-registry:8081");
    }

    #[test]
    fn test_base_url_no_trailing_slash() {
        let config = SchemaRegistryConfig::new("http://schema-registry:8081");
        let client = HttpSchemaRegistryClient::new(config).unwrap();
        assert_eq!(client.base_url(), "http://schema-registry:8081");
    }

    #[test]
    fn test_client_construction_with_timeout() {
        let mut config = SchemaRegistryConfig::new("http://localhost:8081");
        config.timeout_secs = 10;
        let client = HttpSchemaRegistryClient::new(config);
        assert!(client.is_ok());
    }

    // ---- URL 文字列組み立て検証 ----

    #[test]
    fn test_register_url_format() {
        let config = SchemaRegistryConfig::new("http://localhost:8081");
        let client = HttpSchemaRegistryClient::new(config).unwrap();
        let subject = "k1s0.system.auth.user-created.v1-value";
        let url = format!("{}/subjects/{}/versions", client.base_url(), subject);
        assert_eq!(
            url,
            "http://localhost:8081/subjects/k1s0.system.auth.user-created.v1-value/versions"
        );
    }

    #[test]
    fn test_schema_by_id_url_format() {
        let config = SchemaRegistryConfig::new("http://localhost:8081");
        let client = HttpSchemaRegistryClient::new(config).unwrap();
        let url = format!("{}/schemas/ids/{}", client.base_url(), 42);
        assert_eq!(url, "http://localhost:8081/schemas/ids/42");
    }

    #[test]
    fn test_latest_schema_url_format() {
        let config = SchemaRegistryConfig::new("http://localhost:8081");
        let client = HttpSchemaRegistryClient::new(config).unwrap();
        let subject = "orders-value";
        let url = format!("{}/subjects/{}/versions/latest", client.base_url(), subject);
        assert_eq!(
            url,
            "http://localhost:8081/subjects/orders-value/versions/latest"
        );
    }

    #[test]
    fn test_version_url_format() {
        let config = SchemaRegistryConfig::new("http://localhost:8081");
        let client = HttpSchemaRegistryClient::new(config).unwrap();
        let url = format!(
            "{}/subjects/{}/versions/{}",
            client.base_url(),
            "my-topic-value",
            3
        );
        assert_eq!(
            url,
            "http://localhost:8081/subjects/my-topic-value/versions/3"
        );
    }

    #[test]
    fn test_compatibility_url_format() {
        let config = SchemaRegistryConfig::new("http://localhost:8081");
        let client = HttpSchemaRegistryClient::new(config).unwrap();
        let url = format!(
            "{}/compatibility/subjects/{}/versions/latest",
            client.base_url(),
            "events-value"
        );
        assert_eq!(
            url,
            "http://localhost:8081/compatibility/subjects/events-value/versions/latest"
        );
    }

    #[test]
    fn test_delete_subject_url_format() {
        let config = SchemaRegistryConfig::new("http://localhost:8081");
        let client = HttpSchemaRegistryClient::new(config).unwrap();
        let url = format!("{}/subjects/{}", client.base_url(), "stale-topic-value");
        assert_eq!(
            url,
            "http://localhost:8081/subjects/stale-topic-value"
        );
    }

    #[test]
    fn test_health_check_url_format() {
        let config = SchemaRegistryConfig::new("http://localhost:8081");
        let client = HttpSchemaRegistryClient::new(config).unwrap();
        let url = format!("{}/", client.base_url());
        assert_eq!(url, "http://localhost:8081/");
    }

    // ---- Mock を使ったトレイト動作テスト ----

    #[cfg(feature = "mock")]
    #[tokio::test]
    async fn test_mock_register_schema() {
        use mockall::predicate::*;
        let mut mock = MockSchemaRegistryClient::new();
        mock.expect_register_schema()
            .with(eq("my-topic-value"), always(), eq(SchemaType::Protobuf))
            .times(1)
            .returning(|_, _, _| Box::pin(async { Ok(99) }));

        let id = mock
            .register_schema("my-topic-value", r#"syntax = "proto3";"#, SchemaType::Protobuf)
            .await
            .unwrap();
        assert_eq!(id, 99);
    }

    #[cfg(feature = "mock")]
    #[tokio::test]
    async fn test_mock_get_latest_schema() {
        let mut mock = MockSchemaRegistryClient::new();
        mock.expect_get_latest_schema()
            .times(1)
            .returning(|subject| {
                let subject = subject.to_string();
                Box::pin(async move {
                    Ok(RegisteredSchema {
                        id: 7,
                        subject,
                        version: 2,
                        schema: "syntax = \"proto3\";".to_string(),
                        schema_type: SchemaType::Protobuf,
                    })
                })
            });

        let schema = mock.get_latest_schema("events-value").await.unwrap();
        assert_eq!(schema.id, 7);
        assert_eq!(schema.version, 2);
        assert_eq!(schema.schema_type, SchemaType::Protobuf);
    }

    #[cfg(feature = "mock")]
    #[tokio::test]
    async fn test_mock_health_check_ok() {
        let mut mock = MockSchemaRegistryClient::new();
        mock.expect_health_check()
            .times(1)
            .returning(|| Box::pin(async { Ok(()) }));

        mock.health_check().await.unwrap();
    }

    #[cfg(feature = "mock")]
    #[tokio::test]
    async fn test_mock_health_check_unavailable() {
        let mut mock = MockSchemaRegistryClient::new();
        mock.expect_health_check().times(1).returning(|| {
            Box::pin(async {
                Err(SchemaRegistryError::Unavailable(
                    "connection refused".to_string(),
                ))
            })
        });

        let err = mock.health_check().await.unwrap_err();
        assert!(err.to_string().contains("connection refused"));
    }

    #[cfg(feature = "mock")]
    #[tokio::test]
    async fn test_mock_schema_not_found() {
        let mut mock = MockSchemaRegistryClient::new();
        mock.expect_get_schema_version()
            .times(1)
            .returning(|subject, version| {
                let subject = subject.to_string();
                Box::pin(async move {
                    Err(SchemaRegistryError::SchemaNotFound {
                        subject,
                        version: Some(version),
                    })
                })
            });

        let err = mock
            .get_schema_version("missing-topic-value", 99)
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            SchemaRegistryError::SchemaNotFound {
                version: Some(99),
                ..
            }
        ));
    }

    #[cfg(feature = "mock")]
    #[tokio::test]
    async fn test_mock_check_compatibility_false() {
        let mut mock = MockSchemaRegistryClient::new();
        mock.expect_check_compatibility()
            .times(1)
            .returning(|_, _, _| Box::pin(async { Ok(false) }));

        let compatible = mock
            .check_compatibility("events-value", "syntax = \"proto3\";", SchemaType::Protobuf)
            .await
            .unwrap();
        assert!(!compatible);
    }

    #[cfg(feature = "mock")]
    #[tokio::test]
    async fn test_mock_list_subjects() {
        let mut mock = MockSchemaRegistryClient::new();
        mock.expect_list_subjects().times(1).returning(|| {
            Box::pin(async {
                Ok(vec![
                    "topic-a-value".to_string(),
                    "topic-b-value".to_string(),
                ])
            })
        });

        let subjects = mock.list_subjects().await.unwrap();
        assert_eq!(subjects.len(), 2);
    }

    #[cfg(feature = "mock")]
    #[tokio::test]
    async fn test_mock_delete_subject() {
        let mut mock = MockSchemaRegistryClient::new();
        mock.expect_delete_subject()
            .times(1)
            .returning(|_| Box::pin(async { Ok(vec![1, 2, 3]) }));

        let versions = mock.delete_subject("old-topic-value").await.unwrap();
        assert_eq!(versions, vec![1, 2, 3]);
    }
}
