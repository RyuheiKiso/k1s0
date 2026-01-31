//! AST コンテキスト: パース済みソースに対するクエリ実行

use streaming_iterator::StreamingIterator;
use tree_sitter::{Node, Tree};

use super::parser::{Language, ParserPool};
use super::query::{QueryCache, QueryId};

/// パース済みソースコードのコンテキスト
pub struct AstContext<'a> {
    tree: Tree,
    source: &'a [u8],
    lang: Language,
}

impl<'a> AstContext<'a> {
    /// ソースコードをパースして `AstContext` を生成
    pub fn parse(pool: &mut ParserPool, lang: Language, source: &'a [u8]) -> Option<Self> {
        let tree = pool.parse(lang, source)?;
        Some(Self { tree, source, lang })
    }

    /// ノードのテキストを取得
    pub fn node_text(&self, node: &Node<'_>) -> &str {
        node.utf8_text(self.source).unwrap_or("")
    }

    /// ノードの行番号（1-indexed）を取得
    pub fn node_line(&self, node: &Node<'_>) -> usize {
        node.start_position().row + 1
    }

    /// ノードがコメントまたは文字列リテラル内かどうか判定
    pub fn is_non_code(&self, node: &Node<'_>) -> bool {
        let mut current = *node;
        loop {
            let kind = current.kind();
            if is_comment_kind(kind) || is_string_literal_kind(kind) {
                return true;
            }
            match current.parent() {
                Some(p) => current = p,
                None => break,
            }
        }
        false
    }

    /// ノードがテストコード内かどうか判定
    pub fn is_in_test(&self, node: &Node<'_>) -> bool {
        let mut current = *node;
        loop {
            let kind = current.kind();
            let text = self.node_text(&current);

            match self.lang {
                Language::Rust => {
                    // #[cfg(test)] mod や #[test] fn
                    if kind == "attribute_item" && text.contains("test") {
                        return true;
                    }
                }
                Language::Go => {
                    if kind == "function_declaration" {
                        if let Some(name_node) = current.child_by_field_name("name") {
                            let name = self.node_text(&name_node);
                            if name.starts_with("Test") || name.starts_with("Benchmark") {
                                return true;
                            }
                        }
                    }
                }
                Language::TypeScript => {
                    if (kind == "call_expression" || kind == "expression_statement") &&
                       (text.starts_with("describe(") || text.starts_with("it(") || text.starts_with("test("))
                    {
                        return true;
                    }
                }
                Language::Python => {
                    if kind == "function_definition" {
                        if let Some(name_node) = current.child_by_field_name("name") {
                            let name = self.node_text(&name_node);
                            if name.starts_with("test_") {
                                return true;
                            }
                        }
                    }
                }
                Language::CSharp => {
                    if kind == "attribute" && (text.contains("Test") || text.contains("Fact") || text.contains("Theory")) {
                        return true;
                    }
                }
                Language::Kotlin => {
                    if kind == "annotation" && text.contains("Test") {
                        return true;
                    }
                }
            }

            match current.parent() {
                Some(p) => current = p,
                None => break,
            }
        }
        false
    }

    /// クエリを実行してマッチしたノードごとにコールバックを呼び出す
    pub fn query_matches<F>(
        &self,
        cache: &mut QueryCache,
        query_id: QueryId,
        query_source: &str,
        mut callback: F,
    ) where
        F: FnMut(&Node<'_>, &str),
    {
        let ts_lang = self.tree.language();
        let query = match cache.get_or_compile(self.lang, query_id, query_source, &ts_lang) {
            Ok(q) => q,
            Err(_) => return,
        };

        let mut cursor = tree_sitter::QueryCursor::new();
        let mut matches = cursor.matches(query, self.tree.root_node(), self.source);

        let capture_names = query.capture_names();

        while let Some(m) = matches.next() {
            for capture in m.captures {
                let name: &str = capture_names
                    .get(capture.index as usize)
                    .copied()
                    .unwrap_or("");
                callback(&capture.node, name);
            }
        }
    }
}

/// コメント系ノード種別の判定
fn is_comment_kind(kind: &str) -> bool {
    matches!(
        kind,
        "line_comment"
            | "block_comment"
            | "comment"
            | "multiline_comment"
    )
}

/// 文字列リテラル系ノード種別の判定
fn is_string_literal_kind(kind: &str) -> bool {
    matches!(
        kind,
        "string_literal"
            | "raw_string_literal"
            | "interpreted_string_literal"
            | "string"
            | "template_string"
            | "string_content"
    )
}
