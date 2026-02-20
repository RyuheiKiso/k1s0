use serde::{Deserialize, Serialize};

/// Schema Registry に登録されたスキーマを表す。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredSchema {
    /// Schema Registry が割り当てたグローバルスキーマ ID。
    pub id: i32,
    /// サブジェクト名（例: `k1s0.system.auth.user-created.v1-value`）。
    pub subject: String,
    /// サブジェクト内のバージョン番号。
    pub version: i32,
    /// スキーマ定義文字列（Protobuf の場合は .proto ファイルの内容）。
    pub schema: String,
    /// スキーマのフォーマット種別。
    pub schema_type: SchemaType,
}

/// スキーマのフォーマット種別。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchemaType {
    /// Apache Avro 形式。
    Avro,
    /// JSON Schema 形式。
    Json,
    /// Protocol Buffers 形式。
    Protobuf,
}

impl SchemaType {
    /// Confluent Schema Registry API で使用する文字列表現を返す。
    pub fn as_str(&self) -> &'static str {
        match self {
            SchemaType::Avro => "AVRO",
            SchemaType::Json => "JSON",
            SchemaType::Protobuf => "PROTOBUF",
        }
    }
}

impl std::fmt::Display for SchemaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Schema Registry REST API の `/subjects/{subject}/versions` エンドポイントへの
/// スキーマ登録リクエストペイロード。
#[derive(Debug, Serialize)]
pub(crate) struct RegisterSchemaRequest<'a> {
    /// スキーマ定義文字列。
    pub schema: &'a str,
    /// スキーマフォーマット種別文字列。
    #[serde(rename = "schemaType")]
    pub schema_type: &'a str,
}

/// Schema Registry REST API のスキーマ登録レスポンス。
#[derive(Debug, Deserialize)]
pub(crate) struct RegisterSchemaResponse {
    /// 割り当てられたスキーマ ID。
    pub id: i32,
}

/// Schema Registry REST API の `/schemas/ids/{id}` エンドポイントのレスポンス。
#[derive(Debug, Deserialize)]
pub(crate) struct SchemaByIdResponse {
    /// スキーマ定義文字列。
    pub schema: String,
    /// スキーマフォーマット種別文字列。
    #[serde(rename = "schemaType", default = "default_schema_type_str")]
    pub schema_type: String,
    /// サブジェクト情報（オプション）。
    pub subject: Option<String>,
    /// バージョン（オプション）。
    pub version: Option<i32>,
    /// スキーマ ID。
    pub id: Option<i32>,
}

fn default_schema_type_str() -> String {
    "AVRO".to_string()
}

/// Schema Registry REST API の `/subjects/{subject}/versions/{version}` レスポンス。
#[derive(Debug, Deserialize)]
pub(crate) struct SchemaVersionResponse {
    /// サブジェクト名。
    pub subject: String,
    /// バージョン番号。
    pub version: i32,
    /// グローバルスキーマ ID。
    pub id: i32,
    /// スキーマ定義文字列。
    pub schema: String,
    /// スキーマフォーマット種別文字列。
    #[serde(rename = "schemaType", default = "default_schema_type_str")]
    pub schema_type: String,
}

/// 文字列から SchemaType へ変換する。
///
/// 大文字小文字を区別しない。不明な文字列の場合は Avro を返す。
pub(crate) fn parse_schema_type(s: &str) -> SchemaType {
    match s.to_uppercase().as_str() {
        "PROTOBUF" => SchemaType::Protobuf,
        "JSON" => SchemaType::Json,
        _ => SchemaType::Avro,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_type_as_str() {
        assert_eq!(SchemaType::Avro.as_str(), "AVRO");
        assert_eq!(SchemaType::Json.as_str(), "JSON");
        assert_eq!(SchemaType::Protobuf.as_str(), "PROTOBUF");
    }

    #[test]
    fn test_schema_type_display() {
        assert_eq!(SchemaType::Protobuf.to_string(), "PROTOBUF");
        assert_eq!(SchemaType::Avro.to_string(), "AVRO");
    }

    #[test]
    fn test_parse_schema_type_protobuf() {
        assert_eq!(parse_schema_type("PROTOBUF"), SchemaType::Protobuf);
        assert_eq!(parse_schema_type("protobuf"), SchemaType::Protobuf);
    }

    #[test]
    fn test_parse_schema_type_json() {
        assert_eq!(parse_schema_type("JSON"), SchemaType::Json);
    }

    #[test]
    fn test_parse_schema_type_avro_and_unknown() {
        assert_eq!(parse_schema_type("AVRO"), SchemaType::Avro);
        assert_eq!(parse_schema_type("UNKNOWN"), SchemaType::Avro);
    }

    #[test]
    fn test_registered_schema_clone() {
        let s = RegisteredSchema {
            id: 42,
            subject: "my-topic-value".to_string(),
            version: 1,
            schema: "syntax = \"proto3\";".to_string(),
            schema_type: SchemaType::Protobuf,
        };
        let cloned = s.clone();
        assert_eq!(cloned.id, 42);
        assert_eq!(cloned.subject, "my-topic-value");
        assert_eq!(cloned.schema_type, SchemaType::Protobuf);
    }

    #[test]
    fn test_registered_schema_serialize_deserialize() {
        let s = RegisteredSchema {
            id: 1,
            subject: "test-value".to_string(),
            version: 2,
            schema: r#"syntax = "proto3"; message Foo {}"#.to_string(),
            schema_type: SchemaType::Protobuf,
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: RegisteredSchema = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, 1);
        assert_eq!(back.version, 2);
        assert_eq!(back.schema_type, SchemaType::Protobuf);
    }

    #[test]
    fn test_register_schema_request_serializes_schema_type_field() {
        let req = RegisterSchemaRequest {
            schema: r#"syntax = "proto3";"#,
            schema_type: SchemaType::Protobuf.as_str(),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("schemaType"));
        assert!(json.contains("PROTOBUF"));
    }

    #[test]
    fn test_schema_type_copy() {
        let t = SchemaType::Json;
        let copied = t;
        assert_eq!(t, copied);
    }
}
