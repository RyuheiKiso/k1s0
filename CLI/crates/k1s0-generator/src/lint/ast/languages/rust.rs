//! Rust 用 tree-sitter クエリ

/// K029: panic/unwrap/expect 検出
pub const PANIC_QUERY: &str = r#"
(macro_invocation
  macro: (identifier) @name
  (#any-of? @name "panic" "todo" "unimplemented" "unreachable")
) @match

(call_expression
  function: (field_expression
    field: (field_identifier) @method
    (#any-of? @method "unwrap" "expect")
  )
) @match
"#;

/// K050: SQL インジェクション検出（format! マクロ内の SQL）
pub const SQL_INJECTION_QUERY: &str = r#"
(macro_invocation
  macro: (identifier) @name
  (#eq? @name "format")
  (token_tree) @args
) @match
"#;

/// K020: 環境変数使用検出
pub const ENV_VAR_QUERY: &str = r#"
(call_expression
  function: (scoped_identifier) @func
  (#any-of? @func
    "std::env::var" "std::env::var_os" "std::env::vars"
    "std::env::vars_os" "std::env::set_var" "std::env::remove_var"
    "env::var" "env::var_os" "env::vars" "env::set_var" "env::remove_var"
  )
) @match

(use_declaration
  (scoped_identifier) @import
  (#match? @import "dotenv|dotenvy")
) @match
"#;

/// K022: import 検出
pub const DEPENDENCY_IMPORT_QUERY: &str = r#"
(use_declaration) @import
"#;

/// K026: プロトコル依存検出
pub const PROTOCOL_DEPENDENCY_QUERY: &str = r#"
(scoped_identifier) @ident
"#;

/// K053: ログ関数検出
pub const LOG_FUNCTION_QUERY: &str = r#"
(macro_invocation
  macro: (identifier) @name
  (#any-of? @name "info" "warn" "error" "debug" "trace")
  (token_tree) @args
) @match

(macro_invocation
  macro: (scoped_identifier) @name
  (#match? @name "^(tracing|log)::(info|warn|error|debug|trace)$")
  (token_tree) @args
) @match
"#;
