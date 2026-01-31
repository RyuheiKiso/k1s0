//! C# 用 tree-sitter クエリ

/// K029: Environment.Exit/FailFast 検出
pub const PANIC_QUERY: &str = r#"
(invocation_expression
  function: (member_access_expression
    expression: (identifier) @obj
    name: (identifier) @method
    (#eq? @obj "Environment")
    (#any-of? @method "Exit" "FailFast")
  )
) @match
"#;

/// K050: SQL インジェクション検出（補間文字列・連結）
pub const SQL_INJECTION_QUERY: &str = r#"
(interpolated_string_expression) @match

(binary_expression
  left: (string_literal) @str
  operator: "+"
) @match
"#;

/// K020: 環境変数使用検出
pub const ENV_VAR_QUERY: &str = r#"
(invocation_expression
  function: (member_access_expression
    expression: (identifier) @obj
    name: (identifier) @method
    (#eq? @obj "Environment")
    (#any-of? @method "GetEnvironmentVariable" "GetEnvironmentVariables" "ExpandEnvironmentVariables")
  )
) @match

(invocation_expression
  function: (member_access_expression
    name: (identifier) @method
    (#eq? @method "AddEnvironmentVariables")
  )
) @match
"#;

/// K022: import 検出
pub const DEPENDENCY_IMPORT_QUERY: &str = r#"
(using_directive) @import
"#;

/// K026: プロトコル依存検出
pub const PROTOCOL_DEPENDENCY_QUERY: &str = r#"
(member_access_expression) @access
(identifier) @ident
"#;

/// K053: ログ関数検出
pub const LOG_FUNCTION_QUERY: &str = r#"
(invocation_expression
  function: (member_access_expression
    name: (identifier) @method
  )
  arguments: (argument_list) @args
) @match
"#;
