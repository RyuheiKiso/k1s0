//! Python 用 tree-sitter クエリ

/// K029: sys.exit/os._exit 検出
pub const PANIC_QUERY: &str = r#"
(call
  function: (attribute
    object: (identifier) @obj
    attribute: (identifier) @method
    (#eq? @obj "sys")
    (#eq? @method "exit")
  )
) @match

(call
  function: (attribute
    object: (identifier) @obj
    attribute: (identifier) @method
    (#eq? @obj "os")
    (#eq? @method "_exit")
  )
) @match
"#;

/// K050: SQL インジェクション検出（f-string, .format()）
pub const SQL_INJECTION_QUERY: &str = r#"
(string) @match
(call
  function: (attribute
    attribute: (identifier) @method
    (#eq? @method "format")
  )
  arguments: (argument_list) @args
) @match
"#;

/// K020: 環境変数使用検出
pub const ENV_VAR_QUERY: &str = r#"
(call
  function: (attribute
    object: (identifier) @obj
    attribute: (identifier) @method
    (#eq? @obj "os")
    (#any-of? @method "environ" "getenv" "putenv" "unsetenv")
  )
) @match

(attribute
  object: (identifier) @obj
  attribute: (identifier) @attr
  (#eq? @obj "os")
  (#eq? @attr "environ")
) @match

(import_from_statement
  module_name: (dotted_name) @mod
  (#match? @mod "dotenv")
) @match

(import_statement
  name: (dotted_name) @mod
  (#match? @mod "dotenv")
) @match
"#;

/// K022: import 検出
pub const DEPENDENCY_IMPORT_QUERY: &str = r#"
(import_statement) @import
(import_from_statement) @import
"#;

/// K026: プロトコル依存検出
pub const PROTOCOL_DEPENDENCY_QUERY: &str = r#"
(attribute) @attr
(identifier) @ident
"#;

/// K053: ログ関数検出
pub const LOG_FUNCTION_QUERY: &str = r#"
(call
  function: (attribute
    object: (identifier) @obj
    attribute: (identifier) @method
  )
  arguments: (argument_list) @args
) @match

(call
  function: (identifier) @func
  (#eq? @func "print")
  arguments: (argument_list) @args
) @match
"#;
