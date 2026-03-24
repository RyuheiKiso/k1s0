use crate::error::CodegenError;

#[derive(Debug, Clone)]
pub struct ProtoService {
    pub package: String,
    pub service_name: String,
    pub methods: Vec<ProtoMethod>,
    pub messages: Vec<ProtoMessage>,
}

#[derive(Debug, Clone)]
pub struct ProtoMethod {
    pub name: String,
    pub input_type: String,
    pub output_type: String,
}

#[derive(Debug, Clone)]
pub struct ProtoMessage {
    pub name: String,
    pub fields: Vec<ProtoField>,
}

#[derive(Debug, Clone)]
pub struct ProtoField {
    pub name: String,
    pub field_type: String,
    pub number: u32,
}

pub fn parse_proto(path: &std::path::Path) -> Result<ProtoService, CodegenError> {
    let content = std::fs::read_to_string(path).map_err(|e| CodegenError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;
    parse_proto_content(&content)
}

// 正規表現はコンパイルコストが高いため LazyLock で初期化を1回のみに制限する
// proto ファイルのパッケージ宣言を抽出するパターン
static PACKAGE_REGEX: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| {
        regex::Regex::new(r"package\s+([\w.]+)\s*;")
            .expect("有効な正規表現")
    });

// proto ファイルのサービス定義ブロック（名前とボディ）を抽出するパターン
static SERVICE_REGEX: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| {
        regex::Regex::new(r"service\s+(\w+)\s*\{([^}]*)\}")
            .expect("有効な正規表現")
    });

// proto ファイルの RPC メソッド定義（名前・入力型・出力型）を抽出するパターン
static RPC_REGEX: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| {
        regex::Regex::new(r"rpc\s+(\w+)\s*\(\s*(\w+)\s*\)\s*returns\s*\(\s*(\w+)\s*\)")
            .expect("有効な正規表現")
    });

// proto ファイルのメッセージ定義ブロック（名前とボディ）を抽出するパターン
static MESSAGE_REGEX: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| {
        regex::Regex::new(r"message\s+(\w+)\s*\{([^}]*)\}")
            .expect("有効な正規表現")
    });

// proto ファイルのフィールド定義（型・名前・フィールド番号）を抽出するパターン
static FIELD_REGEX: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| {
        regex::Regex::new(r"(\w+)\s+(\w+)\s*=\s*(\d+)\s*;")
            .expect("有効な正規表現")
    });

pub fn parse_proto_content(content: &str) -> Result<ProtoService, CodegenError> {
    // 事前コンパイル済みの静的正規表現を使用してパースする（動的コンパイルによるオーバーヘッドを排除）
    let package_re = &*PACKAGE_REGEX;
    let service_re = &*SERVICE_REGEX;
    let rpc_re = &*RPC_REGEX;
    let message_re = &*MESSAGE_REGEX;
    let field_re = &*FIELD_REGEX;

    let package = package_re
        .captures(content)
        .map(|c| c[1].to_string())
        .ok_or_else(|| CodegenError::ProtoParse("missing package declaration".into()))?;

    let service_cap = service_re
        .captures(content)
        .ok_or_else(|| CodegenError::ProtoParse("missing service declaration".into()))?;
    let service_name = service_cap[1].to_string();
    let service_body = &service_cap[2];

    let methods: Vec<ProtoMethod> = rpc_re
        .captures_iter(service_body)
        .map(|c| ProtoMethod {
            name: c[1].to_string(),
            input_type: c[2].to_string(),
            output_type: c[3].to_string(),
        })
        .collect();

    let messages: Vec<ProtoMessage> = message_re
        .captures_iter(content)
        .map(|c| {
            let name = c[1].to_string();
            let body = &c[2];
            let fields: Vec<ProtoField> = field_re
                .captures_iter(body)
                .map(|f| ProtoField {
                    field_type: f[1].to_string(),
                    name: f[2].to_string(),
                    number: f[3].parse().unwrap_or(0),
                })
                .collect();
            ProtoMessage { name, fields }
        })
        .collect();

    Ok(ProtoService {
        package,
        service_name,
        methods,
        messages,
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    const SAMPLE_PROTO: &str = r#"
syntax = "proto3";
package k1s0.business.taskmanagement.v1;

service ProjectMasterService {
  rpc CreateProjectType(CreateProjectTypeRequest) returns (CreateProjectTypeResponse);
  rpc GetProjectType(GetProjectTypeRequest) returns (GetProjectTypeResponse);
}

message CreateProjectTypeRequest {
  string name = 1;
  string email = 2;
}

message CreateProjectTypeResponse {
  string id = 1;
}

message GetProjectTypeRequest {
  string id = 1;
}

message GetProjectTypeResponse {
  string id = 1;
  string name = 2;
  string email = 3;
}
"#;

    // proto ファイルからパッケージ名が正しく解析されることを確認する。
    #[test]
    fn parse_package() {
        let svc = parse_proto_content(SAMPLE_PROTO).unwrap();
        assert_eq!(svc.package, "k1s0.business.taskmanagement.v1");
    }

    // proto ファイルからサービス名が正しく解析されることを確認する。
    #[test]
    fn parse_service_name() {
        let svc = parse_proto_content(SAMPLE_PROTO).unwrap();
        assert_eq!(svc.service_name, "ProjectMasterService");
    }

    // proto ファイルから RPC メソッドの一覧が正しく解析されることを確認する。
    #[test]
    fn parse_methods() {
        let svc = parse_proto_content(SAMPLE_PROTO).unwrap();
        assert_eq!(svc.methods.len(), 2);
        assert_eq!(svc.methods[0].name, "CreateProjectType");
        assert_eq!(svc.methods[0].input_type, "CreateProjectTypeRequest");
        assert_eq!(svc.methods[0].output_type, "CreateProjectTypeResponse");
        assert_eq!(svc.methods[1].name, "GetProjectType");
    }

    // proto ファイルからメッセージ定義とフィールドが正しく解析されることを確認する。
    #[test]
    fn parse_messages() {
        let svc = parse_proto_content(SAMPLE_PROTO).unwrap();
        assert_eq!(svc.messages.len(), 4);
        let create_req = &svc.messages[0];
        assert_eq!(create_req.name, "CreateProjectTypeRequest");
        assert_eq!(create_req.fields.len(), 2);
        assert_eq!(create_req.fields[0].name, "name");
        assert_eq!(create_req.fields[0].field_type, "string");
        assert_eq!(create_req.fields[0].number, 1);
    }

    // パッケージ宣言がない proto コンテンツを解析した場合にエラーが返されることを確認する。
    #[test]
    fn missing_package_error() {
        let content = "service Foo { rpc Bar(Baz) returns (Qux); }";
        let result = parse_proto_content(content);
        assert!(result.is_err());
    }

    // サービス宣言がない proto コンテンツを解析した場合にエラーが返されることを確認する。
    #[test]
    fn missing_service_error() {
        let content = "package foo.bar;";
        let result = parse_proto_content(content);
        assert!(result.is_err());
    }

    // 最小構成の proto コンテンツが正しく解析されることを確認する。
    #[test]
    fn parse_minimal_proto() {
        let content = r#"
package test.v1;
service TestService {
  rpc Ping(PingRequest) returns (PingResponse);
}
message PingRequest {}
message PingResponse {}
"#;
        let svc = parse_proto_content(content).unwrap();
        assert_eq!(svc.service_name, "TestService");
        assert_eq!(svc.methods.len(), 1);
        assert_eq!(svc.messages.len(), 2);
    }
}
