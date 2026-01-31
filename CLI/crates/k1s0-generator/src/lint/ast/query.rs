//! tree-sitter Query のキャッシュ管理

use std::collections::HashMap;

use super::parser::Language;

/// ルール識別子（Query キャッシュ用）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QueryId {
    /// K029: panic/unwrap 検出
    PanicDetection,
    /// K050: SQL インジェクション検出
    SqlInjection,
    /// K022: Clean Architecture 依存違反（import 検出）
    DependencyImports,
    /// K020: 環境変数使用検出
    EnvVarUsage,
    /// K026: プロトコル型使用検出
    ProtocolDependency,
    /// K053: ログ関数検出
    LogFunctions,
}

/// コンパイル済み Query をキャッシュ
pub struct QueryCache {
    cache: HashMap<(Language, QueryId), tree_sitter::Query>,
}

impl QueryCache {
    /// 新しいキャッシュを作成
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// キャッシュから Query を取得。なければコンパイルしてキャッシュ
    pub fn get_or_compile(
        &mut self,
        lang: Language,
        query_id: QueryId,
        query_source: &str,
        ts_lang: &tree_sitter::Language,
    ) -> Result<&tree_sitter::Query, tree_sitter::QueryError> {
        use std::collections::hash_map::Entry;
        let key = (lang, query_id);
        match self.cache.entry(key) {
            Entry::Occupied(entry) => Ok(entry.into_mut()),
            Entry::Vacant(entry) => {
                let query = tree_sitter::Query::new(ts_lang, query_source)?;
                Ok(entry.insert(query))
            }
        }
    }
}

impl Default for QueryCache {
    fn default() -> Self {
        Self::new()
    }
}
