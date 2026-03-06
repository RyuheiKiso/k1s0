/// Convert kebab-case to snake_case.
pub fn to_snake(kebab: &str) -> String {
    kebab.replace('-', "_")
}

/// Convert kebab-case to PascalCase.
pub fn to_pascal(kebab: &str) -> String {
    kebab
        .split('-')
        .map(|seg| {
            let mut chars = seg.chars();
            match chars.next() {
                Some(c) => {
                    let upper: String = c.to_uppercase().collect();
                    upper + chars.as_str()
                }
                None => String::new(),
            }
        })
        .collect()
}

/// Convert kebab-case to camelCase.
pub fn to_camel(kebab: &str) -> String {
    let pascal = to_pascal(kebab);
    let mut chars = pascal.chars();
    match chars.next() {
        Some(c) => {
            let lower: String = c.to_lowercase().collect();
            lower + chars.as_str()
        }
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snake_case() {
        assert_eq!(to_snake("auth-server"), "auth_server");
        assert_eq!(to_snake("my-cool-service"), "my_cool_service");
        assert_eq!(to_snake("single"), "single");
    }

    #[test]
    fn pascal_case() {
        assert_eq!(to_pascal("auth-server"), "AuthServer");
        assert_eq!(to_pascal("my-cool-service"), "MyCoolService");
        assert_eq!(to_pascal("single"), "Single");
    }

    #[test]
    fn camel_case() {
        assert_eq!(to_camel("auth-server"), "authServer");
        assert_eq!(to_camel("my-cool-service"), "myCoolService");
        assert_eq!(to_camel("single"), "single");
    }
}
