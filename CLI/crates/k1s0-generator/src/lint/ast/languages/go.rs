//! Go 用 tree-sitter クエリ

/// K029: panic/log.Fatal 検出
pub const PANIC_QUERY: &str = r#"
(call_expression
  function: (identifier) @name
  (#eq? @name "panic")
) @match

(call_expression
  function: (selector_expression
    operand: (identifier) @obj
    field: (field_identifier) @method
    (#eq? @obj "log")
    (#eq? @method "Fatal")
  )
) @match
"#;

/// K050: SQL インジェクション検出
pub const SQL_INJECTION_QUERY: &str = r#"
(call_expression
  function: (selector_expression
    operand: (identifier) @obj
    field: (field_identifier) @method
    (#eq? @obj "fmt")
    (#eq? @method "Sprintf")
  )
  arguments: (argument_list) @args
) @match

(binary_expression
  left: (interpreted_string_literal) @str
  operator: "+"
) @match
"#;

/// K020: 環境変数使用検出
pub const ENV_VAR_QUERY: &str = r#"
(call_expression
  function: (selector_expression
    operand: (identifier) @obj
    field: (field_identifier) @method
    (#eq? @obj "os")
    (#any-of? @method "Getenv" "LookupEnv" "Setenv" "Unsetenv" "Environ")
  )
) @match

(import_spec
  path: (interpreted_string_literal) @path
  (#match? @path "godotenv")
) @match
"#;

/// K022: import 検出
pub const DEPENDENCY_IMPORT_QUERY: &str = r#"
(import_spec
  path: (interpreted_string_literal) @path
) @import
"#;

/// K026: プロトコル依存検出
pub const PROTOCOL_DEPENDENCY_QUERY: &str = r#"
(selector_expression
  operand: (identifier) @obj
  field: (field_identifier) @field
) @access
"#;

/// K053: ログ関数検出
pub const LOG_FUNCTION_QUERY: &str = r#"
(call_expression
  function: (selector_expression
    operand: (identifier) @obj
    field: (field_identifier) @method
  )
  arguments: (argument_list) @args
) @match
"#;
