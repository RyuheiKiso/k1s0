//! manifest.json スキーマ定義
//!
//! LSP の補完・ホバー機能で使用するスキーマ情報を提供する。

use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind, Documentation, MarkupContent, MarkupKind};

/// manifest.json のスキーマ定義
pub struct ManifestSchema {
    /// ルートレベルのキー
    pub root_keys: Vec<ManifestKey>,
}

/// スキーマキーの定義
#[derive(Debug, Clone)]
pub struct ManifestKey {
    /// キー名
    pub name: &'static str,
    /// 説明
    pub description: &'static str,
    /// 必須かどうか
    pub required: bool,
    /// 値の型
    pub value_type: ValueType,
    /// 値の例
    pub examples: Vec<&'static str>,
    /// 子キー（オブジェクトの場合）
    pub children: Option<Vec<ManifestKey>>,
}

/// 値の型
#[derive(Debug, Clone)]
pub enum ValueType {
    /// 文字列
    String,
    /// 数値
    Number,
    /// 真偽値
    Boolean,
    /// オブジェクト
    Object,
    /// 配列
    Array,
    /// 列挙型
    Enum(Vec<&'static str>),
}

impl ManifestSchema {
    /// デフォルトのスキーマを作成
    pub fn new() -> Self {
        Self {
            root_keys: vec![
                ManifestKey {
                    name: "schema_version",
                    description: "スキーマバージョン（セマンティックバージョニング）",
                    required: true,
                    value_type: ValueType::String,
                    examples: vec!["1.0.0"],
                    children: None,
                },
                ManifestKey {
                    name: "k1s0_version",
                    description: "k1s0 CLI のバージョン",
                    required: true,
                    value_type: ValueType::String,
                    examples: vec!["0.1.0"],
                    children: None,
                },
                ManifestKey {
                    name: "template",
                    description: "テンプレート情報",
                    required: true,
                    value_type: ValueType::Object,
                    examples: vec![],
                    children: Some(vec![
                        ManifestKey {
                            name: "name",
                            description: "テンプレート名",
                            required: true,
                            value_type: ValueType::Enum(vec!["backend-rust", "backend-go", "backend-csharp", "backend-python", "frontend-react", "frontend-flutter"]),
                            examples: vec!["backend-rust", "backend-go", "backend-csharp", "backend-python"],
                            children: None,
                        },
                        ManifestKey {
                            name: "version",
                            description: "テンプレートバージョン",
                            required: true,
                            value_type: ValueType::String,
                            examples: vec!["0.1.0", "1.0.0"],
                            children: None,
                        },
                        ManifestKey {
                            name: "source",
                            description: "テンプレートソース（local または registry）",
                            required: false,
                            value_type: ValueType::Enum(vec!["local", "registry"]),
                            examples: vec!["local"],
                            children: None,
                        },
                        ManifestKey {
                            name: "path",
                            description: "テンプレートのパス",
                            required: true,
                            value_type: ValueType::String,
                            examples: vec!["CLI/templates/backend-rust/feature"],
                            children: None,
                        },
                        ManifestKey {
                            name: "revision",
                            description: "Git リビジョン（オプション）",
                            required: false,
                            value_type: ValueType::String,
                            examples: vec!["abc1234", "main"],
                            children: None,
                        },
                        ManifestKey {
                            name: "fingerprint",
                            description: "テンプレートのフィンガープリント（16進数、最低8文字）",
                            required: true,
                            value_type: ValueType::String,
                            examples: vec!["abcd1234ef567890"],
                            children: None,
                        },
                    ]),
                },
                ManifestKey {
                    name: "service",
                    description: "サービス情報",
                    required: true,
                    value_type: ValueType::Object,
                    examples: vec![],
                    children: Some(vec![
                        ManifestKey {
                            name: "service_name",
                            description: "サービス名（小文字英数字とハイフン、1-63文字）",
                            required: true,
                            value_type: ValueType::String,
                            examples: vec!["user-service", "order-api", "auth"],
                            children: None,
                        },
                        ManifestKey {
                            name: "language",
                            description: "プログラミング言語",
                            required: true,
                            value_type: ValueType::Enum(vec!["rust", "go", "csharp", "typescript", "python", "dart"]),
                            examples: vec!["rust", "go", "csharp", "python"],
                            children: None,
                        },
                        ManifestKey {
                            name: "type",
                            description: "サービスタイプ",
                            required: true,
                            value_type: ValueType::Enum(vec!["backend", "frontend", "bff"]),
                            examples: vec!["backend"],
                            children: None,
                        },
                        ManifestKey {
                            name: "framework",
                            description: "使用するフレームワーク（オプション）",
                            required: false,
                            value_type: ValueType::String,
                            examples: vec!["axum", "actix-web", "gin", "react", "flutter"],
                            children: None,
                        },
                    ]),
                },
                ManifestKey {
                    name: "generated_at",
                    description: "生成日時（ISO 8601形式）",
                    required: true,
                    value_type: ValueType::String,
                    examples: vec!["2026-01-27T10:00:00Z"],
                    children: None,
                },
                ManifestKey {
                    name: "managed_paths",
                    description: "CLI が管理するパス（テンプレート更新時に上書きされる）",
                    required: true,
                    value_type: ValueType::Array,
                    examples: vec!["deploy/", "buf.yaml", ".github/"],
                    children: None,
                },
                ManifestKey {
                    name: "protected_paths",
                    description: "CLI が変更しないパス（ユーザーのカスタマイズを保護）",
                    required: true,
                    value_type: ValueType::Array,
                    examples: vec!["src/domain/", "src/application/"],
                    children: None,
                },
                ManifestKey {
                    name: "update_policy",
                    description: "パス別の更新ポリシー",
                    required: false,
                    value_type: ValueType::Object,
                    examples: vec![],
                    children: Some(vec![
                        ManifestKey {
                            name: "*",
                            description: "更新ポリシー（auto: 自動更新、suggest_only: 提案のみ、protected: 変更しない）",
                            required: false,
                            value_type: ValueType::Enum(vec!["auto", "suggest_only", "protected"]),
                            examples: vec!["auto"],
                            children: None,
                        },
                    ]),
                },
                ManifestKey {
                    name: "checksums",
                    description: "ファイルのチェックサム（変更検出用）",
                    required: false,
                    value_type: ValueType::Object,
                    examples: vec![],
                    children: None,
                },
                ManifestKey {
                    name: "dependencies",
                    description: "framework crate への依存情報",
                    required: false,
                    value_type: ValueType::Object,
                    examples: vec![],
                    children: Some(vec![
                        ManifestKey {
                            name: "framework_crates",
                            description: "依存する framework crate のリスト",
                            required: false,
                            value_type: ValueType::Array,
                            examples: vec![],
                            children: Some(vec![
                                ManifestKey {
                                    name: "name",
                                    description: "crate 名",
                                    required: true,
                                    value_type: ValueType::Enum(vec![
                                        "k1s0-config",
                                        "k1s0-error",
                                        "k1s0-validation",
                                        "k1s0-observability",
                                        "k1s0-resilience",
                                        "k1s0-grpc-server",
                                        "k1s0-grpc-client",
                                        "k1s0-health",
                                        "k1s0-db",
                                        "k1s0-cache",
                                        "k1s0-domain-event",
                                        "k1s0-auth",
                                    ]),
                                    examples: vec!["k1s0-config", "k1s0-db"],
                                    children: None,
                                },
                                ManifestKey {
                                    name: "version",
                                    description: "バージョン",
                                    required: true,
                                    value_type: ValueType::String,
                                    examples: vec!["0.1.0"],
                                    children: None,
                                },
                            ]),
                        },
                        ManifestKey {
                            name: "framework",
                            description: "依存する framework パッケージ情報",
                            required: false,
                            value_type: ValueType::Object,
                            examples: vec![],
                            children: None,
                        },
                        ManifestKey {
                            name: "domain",
                            description: "依存するドメインパッケージ情報",
                            required: false,
                            value_type: ValueType::Object,
                            examples: vec![],
                            children: None,
                        },
                    ]),
                },
                ManifestKey {
                    name: "layer",
                    description: "サービスの層（framework / domain / feature）",
                    required: false,
                    value_type: ValueType::Enum(vec!["framework", "domain", "feature"]),
                    examples: vec!["feature"],
                    children: None,
                },
                ManifestKey {
                    name: "domain",
                    description: "依存するドメイン名",
                    required: false,
                    value_type: ValueType::String,
                    examples: vec!["user-management", "order-processing"],
                    children: None,
                },
                ManifestKey {
                    name: "version",
                    description: "ドメインバージョン（SemVer）",
                    required: false,
                    value_type: ValueType::String,
                    examples: vec!["0.1.0", "1.0.0"],
                    children: None,
                },
                ManifestKey {
                    name: "domain_version",
                    description: "ドメインバージョン制約",
                    required: false,
                    value_type: ValueType::String,
                    examples: vec!["^0.1.0", ">=1.0.0"],
                    children: None,
                },
                ManifestKey {
                    name: "min_framework_version",
                    description: "最低 framework バージョン",
                    required: false,
                    value_type: ValueType::String,
                    examples: vec!["0.1.0"],
                    children: None,
                },
                ManifestKey {
                    name: "breaking_changes",
                    description: "破壊的変更一覧",
                    required: false,
                    value_type: ValueType::Object,
                    examples: vec![],
                    children: None,
                },
                ManifestKey {
                    name: "deprecated",
                    description: "非推奨フラグ",
                    required: false,
                    value_type: ValueType::Boolean,
                    examples: vec![],
                    children: None,
                },
                ManifestKey {
                    name: "template_snapshot",
                    description: "テンプレートスナップショット",
                    required: false,
                    value_type: ValueType::Object,
                    examples: vec![],
                    children: None,
                },
            ],
        }
    }

    /// キーパスからキーを検索
    ///
    /// 例: `["template", "name"]` で `template.name` を取得
    pub fn find_key(&self, path: &[&str]) -> Option<&ManifestKey> {
        if path.is_empty() {
            return None;
        }

        let mut current_keys = &self.root_keys;
        let mut result: Option<&ManifestKey> = None;

        for (i, key_name) in path.iter().enumerate() {
            let found = current_keys.iter().find(|k| k.name == *key_name || k.name == "*");
            match found {
                Some(key) => {
                    result = Some(key);
                    if i < path.len() - 1 {
                        // まだパスが残っている場合、子キーに進む
                        match &key.children {
                            Some(children) => current_keys = children,
                            None => return None,
                        }
                    }
                }
                None => return None,
            }
        }

        result
    }

    /// 指定パスの子キーを取得
    ///
    /// 例: `["template"]` で `template` の子キー一覧を取得
    pub fn get_child_keys(&self, path: &[&str]) -> Vec<&ManifestKey> {
        if path.is_empty() {
            return self.root_keys.iter().collect();
        }

        let key = self.find_key(path);
        match key {
            Some(k) => match &k.children {
                Some(children) => children.iter().collect(),
                None => vec![],
            },
            None => vec![],
        }
    }

    /// キーから補完アイテムを生成
    pub fn key_to_completion_item(&self, key: &ManifestKey, include_colon: bool) -> CompletionItem {
        let insert_text = if include_colon {
            format!("\"{}\": ", key.name)
        } else {
            format!("\"{}\"", key.name)
        };

        let detail = match &key.value_type {
            ValueType::String => "string".to_string(),
            ValueType::Number => "number".to_string(),
            ValueType::Boolean => "boolean".to_string(),
            ValueType::Object => "object".to_string(),
            ValueType::Array => "array".to_string(),
            ValueType::Enum(values) => format!("enum: {}", values.join(" | ")),
        };

        CompletionItem {
            label: key.name.to_string(),
            kind: Some(CompletionItemKind::PROPERTY),
            detail: Some(detail),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: self.format_key_documentation(key),
            })),
            insert_text: Some(insert_text),
            ..Default::default()
        }
    }

    /// 値の補完アイテムを生成
    pub fn value_to_completion_items(&self, key: &ManifestKey) -> Vec<CompletionItem> {
        match &key.value_type {
            ValueType::Enum(values) => {
                values.iter().map(|v| {
                    CompletionItem {
                        label: v.to_string(),
                        kind: Some(CompletionItemKind::VALUE),
                        detail: Some(format!("{} の値", key.name)),
                        insert_text: Some(format!("\"{}\"", v)),
                        ..Default::default()
                    }
                }).collect()
            }
            ValueType::Boolean => {
                vec![
                    CompletionItem {
                        label: "true".to_string(),
                        kind: Some(CompletionItemKind::VALUE),
                        ..Default::default()
                    },
                    CompletionItem {
                        label: "false".to_string(),
                        kind: Some(CompletionItemKind::VALUE),
                        ..Default::default()
                    },
                ]
            }
            _ => {
                // 例を補完候補として提供
                key.examples.iter().map(|ex| {
                    CompletionItem {
                        label: ex.to_string(),
                        kind: Some(CompletionItemKind::VALUE),
                        detail: Some("例".to_string()),
                        insert_text: Some(format!("\"{}\"", ex)),
                        ..Default::default()
                    }
                }).collect()
            }
        }
    }

    /// キーのドキュメントをフォーマット
    fn format_key_documentation(&self, key: &ManifestKey) -> String {
        let mut doc = String::new();

        doc.push_str(&format!("**{}**\n\n", key.name));
        doc.push_str(&format!("{}\n\n", key.description));

        if key.required {
            doc.push_str("**必須**\n\n");
        }

        if let ValueType::Enum(values) = &key.value_type {
            doc.push_str("**有効な値:**\n");
            for v in values {
                doc.push_str(&format!("- `{}`\n", v));
            }
            doc.push('\n');
        }

        if !key.examples.is_empty() {
            doc.push_str("**例:**\n");
            for ex in &key.examples {
                doc.push_str(&format!("- `{}`\n", ex));
            }
        }

        doc
    }
}

impl Default for ManifestSchema {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_creation() {
        let schema = ManifestSchema::new();
        assert!(!schema.root_keys.is_empty());
    }

    #[test]
    fn test_find_root_key() {
        let schema = ManifestSchema::new();

        let key = schema.find_key(&["k1s0_version"]);
        assert!(key.is_some());
        assert_eq!(key.unwrap().name, "k1s0_version");
    }

    #[test]
    fn test_find_nested_key() {
        let schema = ManifestSchema::new();

        let key = schema.find_key(&["template", "name"]);
        assert!(key.is_some());
        assert_eq!(key.unwrap().name, "name");
    }

    #[test]
    fn test_find_deeply_nested_key() {
        let schema = ManifestSchema::new();

        let key = schema.find_key(&["service", "language"]);
        assert!(key.is_some());
        let key = key.unwrap();
        assert_eq!(key.name, "language");
        assert!(matches!(key.value_type, ValueType::Enum(_)));
    }

    #[test]
    fn test_find_nonexistent_key() {
        let schema = ManifestSchema::new();

        let key = schema.find_key(&["nonexistent"]);
        assert!(key.is_none());
    }

    #[test]
    fn test_get_child_keys_root() {
        let schema = ManifestSchema::new();

        let children = schema.get_child_keys(&[]);
        assert!(!children.is_empty());
        assert!(children.iter().any(|k| k.name == "schema_version"));
        assert!(children.iter().any(|k| k.name == "template"));
    }

    #[test]
    fn test_get_child_keys_template() {
        let schema = ManifestSchema::new();

        let children = schema.get_child_keys(&["template"]);
        assert!(!children.is_empty());
        assert!(children.iter().any(|k| k.name == "name"));
        assert!(children.iter().any(|k| k.name == "version"));
        assert!(children.iter().any(|k| k.name == "fingerprint"));
    }

    #[test]
    fn test_key_to_completion_item() {
        let schema = ManifestSchema::new();

        let key = schema.find_key(&["service", "language"]).unwrap();
        let item = schema.key_to_completion_item(key, true);

        assert_eq!(item.label, "language");
        assert!(item.insert_text.unwrap().contains("language"));
    }

    #[test]
    fn test_value_to_completion_items_enum() {
        let schema = ManifestSchema::new();

        let key = schema.find_key(&["service", "language"]).unwrap();
        let items = schema.value_to_completion_items(key);

        assert!(!items.is_empty());
        assert!(items.iter().any(|i| i.label == "rust"));
        assert!(items.iter().any(|i| i.label == "go"));
    }

    #[test]
    fn test_value_to_completion_items_boolean() {
        let key = ManifestKey {
            name: "test_bool",
            description: "Test",
            required: false,
            value_type: ValueType::Boolean,
            examples: vec![],
            children: None,
        };

        let schema = ManifestSchema::new();
        let items = schema.value_to_completion_items(&key);

        assert_eq!(items.len(), 2);
        assert!(items.iter().any(|i| i.label == "true"));
        assert!(items.iter().any(|i| i.label == "false"));
    }

    #[test]
    fn test_schema_default() {
        let schema1 = ManifestSchema::new();
        let schema2 = ManifestSchema::default();

        assert_eq!(schema1.root_keys.len(), schema2.root_keys.len());
    }

    #[test]
    fn test_find_key_empty_path() {
        let schema = ManifestSchema::new();
        let key = schema.find_key(&[]);
        assert!(key.is_none());
    }

    #[test]
    fn test_find_key_wildcard_match() {
        let schema = ManifestSchema::new();

        // update_policy 内の "*" キーにマッチ
        let key = schema.find_key(&["update_policy", "some_path"]);
        // ワイルドカードマッチが動作する
        if let Some(k) = key {
            assert_eq!(k.name, "*");
        }
    }

    #[test]
    fn test_get_child_keys_nonexistent() {
        let schema = ManifestSchema::new();
        let children = schema.get_child_keys(&["nonexistent"]);
        assert!(children.is_empty());
    }

    #[test]
    fn test_get_child_keys_leaf_node() {
        let schema = ManifestSchema::new();
        // schema_version は子を持たない
        let children = schema.get_child_keys(&["schema_version"]);
        assert!(children.is_empty());
    }

    #[test]
    fn test_get_child_keys_service() {
        let schema = ManifestSchema::new();
        let children = schema.get_child_keys(&["service"]);

        assert!(!children.is_empty());
        assert!(children.iter().any(|k| k.name == "service_name"));
        assert!(children.iter().any(|k| k.name == "language"));
        assert!(children.iter().any(|k| k.name == "type"));
    }

    #[test]
    fn test_key_to_completion_item_without_colon() {
        let schema = ManifestSchema::new();
        let key = schema.find_key(&["schema_version"]).unwrap();
        let item = schema.key_to_completion_item(key, false);

        assert_eq!(item.label, "schema_version");
        assert!(item.insert_text.as_ref().unwrap().contains("\"schema_version\""));
        assert!(!item.insert_text.as_ref().unwrap().contains(":"));
    }

    #[test]
    fn test_value_to_completion_items_string_with_examples() {
        let key = ManifestKey {
            name: "test_string",
            description: "Test",
            required: true,
            value_type: ValueType::String,
            examples: vec!["example1", "example2"],
            children: None,
        };

        let schema = ManifestSchema::new();
        let items = schema.value_to_completion_items(&key);

        assert_eq!(items.len(), 2);
        assert!(items.iter().any(|i| i.label == "example1"));
        assert!(items.iter().any(|i| i.label == "example2"));
    }

    #[test]
    fn test_value_to_completion_items_string_without_examples() {
        let key = ManifestKey {
            name: "test_string",
            description: "Test",
            required: true,
            value_type: ValueType::String,
            examples: vec![],
            children: None,
        };

        let schema = ManifestSchema::new();
        let items = schema.value_to_completion_items(&key);

        assert!(items.is_empty());
    }

    #[test]
    fn test_value_to_completion_items_number() {
        let key = ManifestKey {
            name: "test_number",
            description: "Test",
            required: true,
            value_type: ValueType::Number,
            examples: vec!["42", "100"],
            children: None,
        };

        let schema = ManifestSchema::new();
        let items = schema.value_to_completion_items(&key);

        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_value_to_completion_items_object() {
        let key = ManifestKey {
            name: "test_object",
            description: "Test",
            required: true,
            value_type: ValueType::Object,
            examples: vec![],
            children: None,
        };

        let schema = ManifestSchema::new();
        let items = schema.value_to_completion_items(&key);

        // オブジェクトは例がない場合は空
        assert!(items.is_empty());
    }

    #[test]
    fn test_value_to_completion_items_array() {
        let key = ManifestKey {
            name: "test_array",
            description: "Test",
            required: true,
            value_type: ValueType::Array,
            examples: vec!["item1"],
            children: None,
        };

        let schema = ManifestSchema::new();
        let items = schema.value_to_completion_items(&key);

        assert!(!items.is_empty());
    }

    #[test]
    fn test_key_to_completion_item_detail_types() {
        let schema = ManifestSchema::new();

        // String type
        let key = schema.find_key(&["schema_version"]).unwrap();
        let item = schema.key_to_completion_item(key, true);
        assert!(item.detail.as_ref().unwrap().contains("string"));

        // Enum type
        let key = schema.find_key(&["service", "language"]).unwrap();
        let item = schema.key_to_completion_item(key, true);
        assert!(item.detail.as_ref().unwrap().contains("enum"));

        // Object type
        let key = schema.find_key(&["template"]).unwrap();
        let item = schema.key_to_completion_item(key, true);
        assert!(item.detail.as_ref().unwrap().contains("object"));
    }

    #[test]
    fn test_manifest_key_debug() {
        let key = ManifestKey {
            name: "test",
            description: "Test key",
            required: true,
            value_type: ValueType::String,
            examples: vec!["ex1"],
            children: None,
        };

        let debug_str = format!("{:?}", key);
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_value_type_debug() {
        let vt = ValueType::String;
        assert!(format!("{:?}", vt).contains("String"));

        let vt = ValueType::Enum(vec!["a", "b"]);
        assert!(format!("{:?}", vt).contains("Enum"));
    }

    #[test]
    fn test_manifest_key_clone() {
        let key = ManifestKey {
            name: "test",
            description: "Test",
            required: true,
            value_type: ValueType::String,
            examples: vec!["ex"],
            children: None,
        };

        let cloned = key.clone();
        assert_eq!(cloned.name, key.name);
        assert_eq!(cloned.required, key.required);
    }

    #[test]
    fn test_value_type_clone() {
        let vt = ValueType::Enum(vec!["a", "b"]);
        let cloned = vt.clone();

        if let ValueType::Enum(values) = cloned {
            assert_eq!(values.len(), 2);
        } else {
            panic!("Clone failed");
        }
    }

    #[test]
    fn test_find_dependencies_framework_crates() {
        let schema = ManifestSchema::new();

        // dependencies.framework_crates を探す
        let key = schema.find_key(&["dependencies", "framework_crates"]);
        assert!(key.is_some());
        assert_eq!(key.unwrap().name, "framework_crates");
    }

    #[test]
    fn test_format_key_documentation() {
        let schema = ManifestSchema::new();
        let key = schema.find_key(&["service", "language"]).unwrap();
        let doc = schema.format_key_documentation(key);

        assert!(doc.contains("language"));
        assert!(doc.contains("有効な値"));
        assert!(doc.contains("rust"));
    }
}
