use serde::de::DeserializeOwned;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("config file not found: {0}")]
    NotFound(String),
    #[error("config parse error: {0}")]
    ParseError(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// YAML コンテンツ中の ${VAR:-default} パターンを環境変数で展開する。
/// serde_yaml はシェル変数展開を行わないため、YAML 読み込み前に手動展開が必要。
/// 対応パターン:
///   ${VAR}         → 環境変数 VAR の値（未設定時は空文字列）
///   ${VAR:-default} → 環境変数 VAR の値（未設定または空の場合は "default"）
fn expand_env_vars(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let bytes = content.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // ${ の開始を検出する
        if i + 1 < bytes.len() && bytes[i] == b'$' && bytes[i + 1] == b'{' {
            // 閉じ括弧 } を探す
            if let Some(end) = content[i + 2..].find('}') {
                let expr = &content[i + 2..i + 2 + end];
                // ${VAR:-default} パターンを解析する
                let (var_name, default_val) = if let Some(sep) = expr.find(":-") {
                    (&expr[..sep], Some(&expr[sep + 2..]))
                } else {
                    (expr, None)
                };
                let env_val = std::env::var(var_name).ok();
                let expanded = match (env_val.as_deref(), default_val) {
                    (Some(v), _) if !v.is_empty() => v.to_string(),
                    (_, Some(d)) => d.to_string(),
                    (Some(v), None) => v.to_string(),
                    (None, None) => String::new(),
                };
                result.push_str(&expanded);
                i += 2 + end + 1; // ${ + expr + }
                continue;
            }
        }
        result.push(bytes[i] as char);
        i += 1;
    }
    result
}

pub fn load_config<T: DeserializeOwned>(path: &str) -> Result<T, ConfigError> {
    let content =
        std::fs::read_to_string(path).map_err(|_| ConfigError::NotFound(path.to_string()))?;

    // YAML 読み込み前に環境変数展開を行う（serde_yaml は展開しないため）
    let expanded = expand_env_vars(&content);
    serde_yaml::from_str(&expanded).map_err(|e| ConfigError::ParseError(e.to_string()))
}
