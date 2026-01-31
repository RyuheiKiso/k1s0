//! Kotlin 用 tree-sitter クエリ

/// K029: !!, error(), TODO(), throw RuntimeException 検出
pub const PANIC_QUERY: &str = r#"
(postfix_expression
  operator: (non_null_assertion) @op
) @match

(call_expression
  function: (simple_identifier) @name
  (#any-of? @name "error" "TODO")
) @match

(throw_expression) @match
"#;

/// K050: SQL インジェクション検出（文字列テンプレート・連結）
pub const SQL_INJECTION_QUERY: &str = r#"
(string_literal) @match

(additive_expression
  left: (string_literal) @str
) @match
"#;

/// K020: 環境変数使用検出
pub const ENV_VAR_QUERY: &str = r#"
(call_expression
  function: (navigation_expression) @func
) @match

(import_header
  (identifier) @import
) @match
"#;

/// K022: import 検出
pub const DEPENDENCY_IMPORT_QUERY: &str = r#"
(import_header) @import
"#;

/// K026: プロトコル依存検出
pub const PROTOCOL_DEPENDENCY_QUERY: &str = r#"
(navigation_expression) @nav
(simple_identifier) @ident
"#;

/// K053: ログ関数検出
pub const LOG_FUNCTION_QUERY: &str = r#"
(call_expression
  function: (navigation_expression) @func
  arguments: (call_suffix) @args
) @match
"#;
