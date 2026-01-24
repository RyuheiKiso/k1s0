//! テンプレートのレンダリング
//!
//! Tera を使用したテンプレート展開を提供する。

use std::path::Path;
use tera::{Context, Tera};

use crate::Result;

/// テンプレートレンダラー
pub struct TemplateRenderer {
    tera: Tera,
}

impl TemplateRenderer {
    /// 新しいレンダラーを作成する
    pub fn new<P: AsRef<Path>>(template_dir: P) -> Result<Self> {
        let pattern = format!("{}/**/*", template_dir.as_ref().display());
        let tera = Tera::new(&pattern)?;

        Ok(Self { tera })
    }

    /// テンプレートをレンダリングする
    pub fn render(&self, template_name: &str, context: &Context) -> Result<String> {
        let result = self.tera.render(template_name, context)?;
        Ok(result)
    }

    /// テンプレートディレクトリを展開する
    pub fn render_directory<P: AsRef<Path>>(
        &self,
        _output_dir: P,
        _context: &Context,
    ) -> Result<Vec<String>> {
        // TODO: フェーズ12 で実装
        Ok(vec![])
    }
}

/// テンプレート用のコンテキストを作成する
pub fn create_context(
    service_name: &str,
    language: &str,
    service_type: &str,
    k1s0_version: &str,
) -> Context {
    let mut context = Context::new();
    context.insert("feature_name", service_name);
    context.insert("service_name", service_name);
    context.insert("language", language);
    context.insert("service_type", service_type);
    context.insert("k1s0_version", k1s0_version);

    // 命名規則の変換
    context.insert("feature_name_snake", &service_name.replace('-', "_"));
    context.insert(
        "feature_name_pascal",
        &to_pascal_case(service_name),
    );

    context
}

/// kebab-case を PascalCase に変換する
fn to_pascal_case(s: &str) -> String {
    s.split('-')
        .map(|word| {
            let mut chars: Vec<char> = word.chars().collect();
            if let Some(first) = chars.first_mut() {
                *first = first.to_ascii_uppercase();
            }
            chars.into_iter().collect::<String>()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("user-management"), "UserManagement");
        assert_eq!(to_pascal_case("order"), "Order");
        assert_eq!(to_pascal_case("auth-service"), "AuthService");
    }
}
