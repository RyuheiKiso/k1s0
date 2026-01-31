//! tree-sitter パーサー管理

use std::collections::HashMap;
use std::path::Path;

use tree_sitter::Parser;

/// 対応言語
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    Rust,
    Go,
    TypeScript,
    Python,
    CSharp,
    Kotlin,
}

impl Language {
    /// ファイル拡張子から言語を判定
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            "rs" => Some(Self::Rust),
            "go" => Some(Self::Go),
            "ts" | "tsx" | "js" | "jsx" => Some(Self::TypeScript),
            "py" => Some(Self::Python),
            "cs" => Some(Self::CSharp),
            "kt" | "kts" => Some(Self::Kotlin),
            _ => None,
        }
    }

    /// ファイルパスから言語を判定
    pub fn from_path(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(|e| e.to_str())
            .and_then(Self::from_extension)
    }

    /// tree-sitter の Language オブジェクトを取得
    fn tree_sitter_language(&self) -> tree_sitter::Language {
        match self {
            Self::Rust => tree_sitter_rust::LANGUAGE.into(),
            Self::Go => tree_sitter_go::LANGUAGE.into(),
            Self::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            Self::Python => tree_sitter_python::LANGUAGE.into(),
            Self::CSharp => tree_sitter_c_sharp::LANGUAGE.into(),
            Self::Kotlin => tree_sitter_kotlin_ng::LANGUAGE.into(),
        }
    }
}

/// 言語ごとにパーサーをプールして再利用
pub struct ParserPool {
    parsers: HashMap<Language, Parser>,
}

impl ParserPool {
    /// 新しいパーサープールを作成
    pub fn new() -> Self {
        Self {
            parsers: HashMap::new(),
        }
    }

    /// 指定言語のパーサーを取得（なければ作成）。ABI 非互換の場合は None。
    pub fn get_parser(&mut self, lang: Language) -> Option<&mut Parser> {
        use std::collections::hash_map::Entry;
        if let Entry::Vacant(entry) = self.parsers.entry(lang) {
            let mut parser = Parser::new();
            if parser.set_language(&lang.tree_sitter_language()).is_err() {
                return None;
            }
            entry.insert(parser);
        }
        self.parsers.get_mut(&lang)
    }

    /// ソースコードをパースして構文木を返す
    pub fn parse(&mut self, lang: Language, source: &[u8]) -> Option<tree_sitter::Tree> {
        let parser = self.get_parser(lang)?;
        parser.parse(source, None)
    }
}

impl Default for ParserPool {
    fn default() -> Self {
        Self::new()
    }
}
