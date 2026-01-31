//! TypeScript 用 tree-sitter クエリ

/// K029: process.exit 検出
pub const PANIC_QUERY: &str = r#"
(call_expression
  function: (member_expression
    object: (identifier) @obj
    property: (property_identifier) @method
    (#eq? @obj "process")
    (#eq? @method "exit")
  )
) @match
"#;

/// K050: SQL インジェクション検出（テンプレートリテラル・文字列連結）
pub const SQL_INJECTION_QUERY: &str = r#"
(template_string) @match

(binary_expression
  left: (string) @str
  operator: "+"
) @match
"#;

/// K020: 環境変数使用検出
pub const ENV_VAR_QUERY: &str = r#"
(member_expression
  object: (identifier) @obj
  property: (property_identifier) @prop
  (#eq? @obj "process")
  (#eq? @prop "env")
) @match

(member_expression
  object: (member_expression
    object: (identifier) @obj1
    property: (property_identifier) @prop1
    (#eq? @obj1 "import")
    (#eq? @prop1 "meta")
  )
  property: (property_identifier) @prop2
  (#eq? @prop2 "env")
) @match

(import_statement
  source: (string) @src
  (#match? @src "dotenv")
) @match
"#;

/// K022: import 検出
pub const DEPENDENCY_IMPORT_QUERY: &str = r#"
(import_statement) @import
"#;

/// K026: プロトコル依存検出
pub const PROTOCOL_DEPENDENCY_QUERY: &str = r#"
(identifier) @ident
(member_expression) @member
"#;

/// K053: ログ関数検出
pub const LOG_FUNCTION_QUERY: &str = r#"
(call_expression
  function: (member_expression
    object: (identifier) @obj
    property: (property_identifier) @method
  )
  arguments: (arguments) @args
) @match
"#;
