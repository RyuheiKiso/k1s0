//! AST ベース解析基盤
//!
//! tree-sitter を使用してソースコードの構文木を解析し、
//! grep ベースよりも正確な lint 検出を提供する。

#[cfg(feature = "ast")]
mod context;
#[cfg(feature = "ast")]
pub(crate) mod languages;
#[cfg(feature = "ast")]
pub(crate) mod parser;
#[cfg(feature = "ast")]
pub(crate) mod query;

#[cfg(feature = "ast")]
pub(crate) use context::AstContext;
#[cfg(feature = "ast")]
pub(crate) use parser::ParserPool;
#[cfg(feature = "ast")]
pub(crate) use query::QueryCache;
