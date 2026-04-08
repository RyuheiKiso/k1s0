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
    #[must_use] 
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

/// 文字列から `SchemaType` へ変換する。
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
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // SchemaType の as_str メソッドが正しい文字列を返すことを確認する。
    #[test]
    fn test_schema_type_as_str() {
        assert_eq!(SchemaType::Avro.as_str(), "AVRO");
        assert_eq!(SchemaType::Json.as_str(), "JSON");
        assert_eq!(SchemaType::Protobuf.as_str(), "PROTOBUF");
    }

    // SchemaType の Display 実装が正しい文字列を返すことを確認する。
    #[test]
    fn test_schema_type_display() {
        assert_eq!(SchemaType::Protobuf.to_string(), "PROTOBUF");
        assert_eq!(SchemaType::Avro.to_string(), "AVRO");
    }

    // PROTOBUF 文字列が大文字小文字を問わず正しくパースされることを確認する。
    #[test]
    fn test_parse_schema_type_protobuf() {
        assert_eq!(parse_schema_type("PROTOBUF"), SchemaType::Protobuf);
        assert_eq!(parse_schema_type("protobuf"), SchemaType::Protobuf);
    }

    // JSON 文字列が正しくパースされることを確認する。
    #[test]
    fn test_parse_schema_type_json() {
        assert_eq!(parse_schema_type("JSON"), SchemaType::Json);
    }

    // AVRO および未知の文字列が Avro にフォールバックすることを確認する。
    #[test]
    fn test_parse_schema_type_avro_and_unknown() {
        assert_eq!(parse_schema_type("AVRO"), SchemaType::Avro);
        assert_eq!(parse_schema_type("UNKNOWN"), SchemaType::Avro);
    }

    // RegisteredSchema が正しくクローンできることを確認する。
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

    // RegisteredSchema がシリアライズ・デシリアライズを経ても正しい値を保持することを確認する。
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

    // RegisterSchemaRequest の JSON に schemaType フィールドが含まれることを確認する。
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

    // SchemaType が Copy トレイトで正しくコピーできることを確認する。
    #[test]
    fn test_schema_type_copy() {
        let t = SchemaType::Json;
        let copied = t;
        assert_eq!(t, copied);
    }

    // JSON 文字列が大文字小文字を問わず正しくパースされることを確認する。
    #[test]
    fn test_parse_schema_type_json_case_insensitive() {
        assert_eq!(parse_schema_type("json"), SchemaType::Json);
        assert_eq!(parse_schema_type("Json"), SchemaType::Json);
        assert_eq!(parse_schema_type("jSoN"), SchemaType::Json);
    }

    // AVRO が大文字小文字を問わず正しくパースされることを確認する。
    #[test]
    fn test_parse_schema_type_avro_case_insensitive() {
        assert_eq!(parse_schema_type("avro"), SchemaType::Avro);
        assert_eq!(parse_schema_type("Avro"), SchemaType::Avro);
    }

    // 空文字列が Avro にフォールバックすることを確認する。
    #[test]
    fn test_parse_schema_type_empty_string() {
        assert_eq!(parse_schema_type(""), SchemaType::Avro);
    }

    // SchemaType の Display 出力が JSON の場合に正しいことを確認する。
    #[test]
    fn test_schema_type_display_json() {
        assert_eq!(SchemaType::Json.to_string(), "JSON");
    }

    // RegisteredSchema のデフォルト値がないフィールドが正しく設定されることを確認する。
    #[test]
    fn test_registered_schema_all_fields() {
        let s = RegisteredSchema {
            id: 100,
            subject: "k1s0.system.auth.user-created.v1-value".to_string(),
            version: 5,
            schema: r#"{"type": "record", "name": "User"}"#.to_string(),
            schema_type: SchemaType::Avro,
        };
        assert_eq!(s.id, 100);
        assert_eq!(s.subject, "k1s0.system.auth.user-created.v1-value");
        assert_eq!(s.version, 5);
        assert_eq!(s.schema_type, SchemaType::Avro);
    }

    // RegisterSchemaRequest が schema と schemaType の両フィールドを JSON に含むことを確認する。
    #[test]
    fn test_register_schema_request_json_structure() {
        let req = RegisterSchemaRequest {
            schema: r#"{"type": "record"}"#,
            schema_type: SchemaType::Avro.as_str(),
        };
        let json: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&req).unwrap()).unwrap();
        assert!(json.get("schema").is_some());
        assert!(json.get("schemaType").is_some());
        assert_eq!(json["schemaType"], "AVRO");
    }

    // RegisterSchemaRequest の JSON schema type が JSON の場合に正しいことを確認する。
    #[test]
    fn test_register_schema_request_json_type() {
        let req = RegisterSchemaRequest {
            schema: r#"{"$schema": "http://json-schema.org/draft-07/schema#"}"#,
            schema_type: SchemaType::Json.as_str(),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("JSON"));
    }

    // RegisteredSchema の JSON シリアライズに schema_type が含まれることを確認する。
    #[test]
    fn test_registered_schema_json_contains_schema_type() {
        let s = RegisteredSchema {
            id: 1,
            subject: "test".to_string(),
            version: 1,
            schema: "schema".to_string(),
            schema_type: SchemaType::Json,
        };
        let json = serde_json::to_string(&s).unwrap();
        assert!(json.contains("Json"));
    }
}
