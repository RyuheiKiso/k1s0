//! 環境変数から config.yaml への変換

use std::path::Path;

use super::types::{ConfigConversion, EnvEntry};
use crate::Result;

/// .env ファイルをパースしてエントリ一覧を返す
pub fn parse_env_file(path: &Path) -> Result<Vec<EnvEntry>> {
    let content = std::fs::read_to_string(path)?;
    let mut entries = Vec::new();

    let mut pending_comment: Option<String> = None;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            pending_comment = None;
            continue;
        }

        if trimmed.starts_with('#') {
            let comment_text = trimmed.trim_start_matches('#').trim().to_string();
            pending_comment = Some(comment_text);
            continue;
        }

        if let Some((key, value)) = trimmed.split_once('=') {
            let key = key.trim().to_string();
            let value = strip_quotes(value.trim());

            entries.push(EnvEntry {
                key,
                value,
                comment: pending_comment.take(),
            });
        }
    }

    Ok(entries)
}

/// 環境変数エントリを k1s0 の config YAML 構造に変換する
///
/// 返り値は生成された YAML 内容、シークレット参照リスト、変換件数を含む。
pub fn convert_env_to_config(entries: &[EnvEntry]) -> ConfigConversion {
    let mut yaml_map = serde_yaml::Mapping::new();
    let mut secret_refs = Vec::new();
    let mut converted_count = 0usize;

    for entry in entries {
        let upper = entry.key.to_uppercase();

        if is_secret_key(&upper) {
            // シークレットは *_file 参照に変換
            let config_key = env_name_to_yaml_key(&entry.key);
            let file_key = format!("{}_file", config_key);
            let file_path = format!(
                "/var/run/secrets/k1s0/{}",
                entry.key.to_lowercase()
            );

            set_nested_value(&mut yaml_map, &file_key, &file_path);
            secret_refs.push(entry.key.clone());
            converted_count += 1;
            continue;
        }

        if let Some((key, value)) = convert_known_pattern(&entry.key, &entry.value) {
            set_nested_value(&mut yaml_map, &key, &value);
            converted_count += 1;
            continue;
        }

        // DATABASE_URL のパース
        if upper == "DATABASE_URL" {
            if let Some(parts) = parse_database_url(&entry.value) {
                set_nested_value(&mut yaml_map, "database.host", &parts.host);
                set_nested_value(&mut yaml_map, "database.port", &parts.port.to_string());
                set_nested_value(&mut yaml_map, "database.name", &parts.dbname);
                set_nested_value(&mut yaml_map, "database.username", &parts.user);
                // password は _file 参照にする
                set_nested_value(
                    &mut yaml_map,
                    "database.password_file",
                    "/var/run/secrets/k1s0/db_password",
                );
                secret_refs.push("DATABASE_URL (password)".to_string());
                converted_count += 1;
                continue;
            }
        }

        // REDIS_URL のパース
        if upper == "REDIS_URL" {
            if let Some(parts) = parse_redis_url(&entry.value) {
                set_nested_value(&mut yaml_map, "cache.host", &parts.host);
                set_nested_value(&mut yaml_map, "cache.port", &parts.port.to_string());
                if let Some(ref db) = parts.db {
                    set_nested_value(&mut yaml_map, "cache.database", db);
                }
                converted_count += 1;
                continue;
            }
        }

        // 不明な変数は app.{name} に配置
        let key = format!("app.{}", entry.key.to_lowercase());
        set_nested_value(&mut yaml_map, &key, &entry.value);
        converted_count += 1;
    }

    let yaml_value = serde_yaml::Value::Mapping(yaml_map);
    let yaml_content =
        serde_yaml::to_string(&yaml_value).unwrap_or_else(|_| "# conversion failed\n".to_string());

    ConfigConversion {
        yaml_content,
        secret_refs,
        converted_count,
    }
}

/// 既知のパターン辞書による変換
fn convert_known_pattern(key: &str, value: &str) -> Option<(String, String)> {
    let upper = key.to_uppercase();
    match upper.as_str() {
        "PORT" | "APP_PORT" | "SERVER_PORT" => {
            Some(("server.port".to_string(), value.to_string()))
        }
        "HOST" | "APP_HOST" | "SERVER_HOST" | "BIND_ADDRESS" => {
            Some(("server.host".to_string(), value.to_string()))
        }
        "DB_HOST" | "DATABASE_HOST" | "POSTGRES_HOST" => {
            Some(("database.host".to_string(), value.to_string()))
        }
        "DB_PORT" | "DATABASE_PORT" | "POSTGRES_PORT" => {
            Some(("database.port".to_string(), value.to_string()))
        }
        "DB_NAME" | "DATABASE_NAME" | "POSTGRES_DB" => {
            Some(("database.name".to_string(), value.to_string()))
        }
        "DB_USER" | "DATABASE_USER" | "POSTGRES_USER" => {
            Some(("database.username".to_string(), value.to_string()))
        }
        "REDIS_HOST" | "CACHE_HOST" => Some(("cache.host".to_string(), value.to_string())),
        "REDIS_PORT" | "CACHE_PORT" => Some(("cache.port".to_string(), value.to_string())),
        "LOG_LEVEL" | "RUST_LOG" => Some(("logging.level".to_string(), value.to_string())),
        "APP_NAME" | "SERVICE_NAME" => Some(("app.name".to_string(), value.to_string())),
        "APP_ENV" | "NODE_ENV" | "ENVIRONMENT" => {
            Some(("app.environment".to_string(), value.to_string()))
        }
        _ => None,
    }
}

fn is_secret_key(upper_key: &str) -> bool {
    let secret_keywords = [
        "SECRET", "PASSWORD", "PASSWD", "KEY", "TOKEN", "CREDENTIAL",
    ];

    // Exclude known non-secret keys that happen to contain "KEY"
    if upper_key == "API_KEY_FILE" || upper_key.ends_with("_FILE") || upper_key.ends_with("_PATH")
    {
        return false;
    }

    secret_keywords.iter().any(|kw| upper_key.contains(kw))
}

fn env_name_to_yaml_key(env_name: &str) -> String {
    env_name.to_lowercase().replace("__", ".")
}

struct DatabaseUrlParts {
    host: String,
    port: u16,
    dbname: String,
    user: String,
}

/// postgres://user:pass@host:port/dbname 形式をパース
fn parse_database_url(url: &str) -> Option<DatabaseUrlParts> {
    // Strip scheme
    let rest = url
        .strip_prefix("postgres://")
        .or_else(|| url.strip_prefix("postgresql://"))?;

    // user:pass@host:port/dbname
    let (userinfo, hostinfo) = rest.split_once('@')?;
    let user = userinfo.split(':').next().unwrap_or("").to_string();

    let (host_port, dbname) = hostinfo.split_once('/')?;
    // Remove query string from dbname
    let dbname = dbname.split('?').next().unwrap_or(dbname).to_string();

    let (host, port_str) = if let Some((h, p)) = host_port.split_once(':') {
        (h.to_string(), p)
    } else {
        (host_port.to_string(), "5432")
    };

    let port = port_str.parse::<u16>().unwrap_or(5432);

    Some(DatabaseUrlParts {
        host,
        port,
        dbname,
        user,
    })
}

struct RedisUrlParts {
    host: String,
    port: u16,
    db: Option<String>,
}

/// redis://host:port/db 形式をパース
fn parse_redis_url(url: &str) -> Option<RedisUrlParts> {
    let rest = url.strip_prefix("redis://")?;

    // Remove optional userinfo
    let host_part = if let Some((_, after_at)) = rest.split_once('@') {
        after_at
    } else {
        rest
    };

    let (host_port, db) = if let Some((hp, d)) = host_part.split_once('/') {
        let db_str = d.split('?').next().unwrap_or(d);
        (
            hp,
            if db_str.is_empty() {
                None
            } else {
                Some(db_str.to_string())
            },
        )
    } else {
        (host_part, None)
    };

    let (host, port_str) = if let Some((h, p)) = host_port.split_once(':') {
        (h.to_string(), p)
    } else {
        (host_port.to_string(), "6379")
    };

    let port = port_str.parse::<u16>().unwrap_or(6379);

    Some(RedisUrlParts { host, port, db })
}

/// ドット区切りのキーでネストされた YAML マッピングに値を設定する
fn set_nested_value(map: &mut serde_yaml::Mapping, dotted_key: &str, value: &str) {
    let parts: Vec<&str> = dotted_key.split('.').collect();

    if parts.len() == 1 {
        map.insert(
            serde_yaml::Value::String(parts[0].to_string()),
            serde_yaml::Value::String(value.to_string()),
        );
        return;
    }

    let first = parts[0];
    let rest = parts[1..].join(".");

    let sub_map = map
        .entry(serde_yaml::Value::String(first.to_string()))
        .or_insert_with(|| serde_yaml::Value::Mapping(serde_yaml::Mapping::new()));

    if let serde_yaml::Value::Mapping(inner) = sub_map {
        set_nested_value(inner, &rest, value);
    }
}

fn strip_quotes(s: &str) -> String {
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_parse_env_file() {
        let tmp = tempfile::tempdir().expect("tempdir failed");
        let env_path = tmp.path().join(".env");
        fs::write(
            &env_path,
            "# Server config\nPORT=3000\nHOST=0.0.0.0\n\n# Database\nDATABASE_URL=postgres://user:pass@localhost:5432/mydb\nDB_PASSWORD=secret123\n",
        )
        .expect("write failed");

        let entries = parse_env_file(&env_path).expect("parse failed");
        assert_eq!(entries.len(), 4);
        assert_eq!(entries[0].key, "PORT");
        assert_eq!(entries[0].value, "3000");
        assert_eq!(entries[0].comment.as_deref(), Some("Server config"));
    }

    #[test]
    fn test_convert_known_patterns() {
        let entries = vec![
            EnvEntry {
                key: "PORT".to_string(),
                value: "8080".to_string(),
                comment: None,
            },
            EnvEntry {
                key: "HOST".to_string(),
                value: "0.0.0.0".to_string(),
                comment: None,
            },
            EnvEntry {
                key: "LOG_LEVEL".to_string(),
                value: "debug".to_string(),
                comment: None,
            },
        ];

        let result = convert_env_to_config(&entries);
        assert_eq!(result.converted_count, 3);
        assert!(result.yaml_content.contains("port"));
        assert!(result.yaml_content.contains("8080"));
        assert!(result.secret_refs.is_empty());
    }

    #[test]
    fn test_convert_secrets() {
        let entries = vec![EnvEntry {
            key: "DB_PASSWORD".to_string(),
            value: "supersecret".to_string(),
            comment: None,
        }];

        let result = convert_env_to_config(&entries);
        assert_eq!(result.secret_refs.len(), 1);
        assert!(result.yaml_content.contains("_file"));
        assert!(result.yaml_content.contains("/var/run/secrets/k1s0/"));
    }

    #[test]
    fn test_convert_database_url() {
        let entries = vec![EnvEntry {
            key: "DATABASE_URL".to_string(),
            value: "postgres://myuser:mypass@db.example.com:5432/mydb".to_string(),
            comment: None,
        }];

        let result = convert_env_to_config(&entries);
        assert!(result.yaml_content.contains("db.example.com"));
        assert!(result.yaml_content.contains("mydb"));
        assert!(result.yaml_content.contains("myuser"));
        assert!(result.yaml_content.contains("password_file"));
        assert!(!result.secret_refs.is_empty());
    }

    #[test]
    fn test_convert_redis_url() {
        let entries = vec![EnvEntry {
            key: "REDIS_URL".to_string(),
            value: "redis://localhost:6379/0".to_string(),
            comment: None,
        }];

        let result = convert_env_to_config(&entries);
        assert!(result.yaml_content.contains("localhost"));
        assert!(result.yaml_content.contains("6379"));
    }

    #[test]
    fn test_convert_unknown_var() {
        let entries = vec![EnvEntry {
            key: "MY_CUSTOM_VAR".to_string(),
            value: "custom_value".to_string(),
            comment: None,
        }];

        let result = convert_env_to_config(&entries);
        assert!(result.yaml_content.contains("app"));
        assert!(result.yaml_content.contains("my_custom_var"));
        assert!(result.yaml_content.contains("custom_value"));
    }

    #[test]
    fn test_parse_database_url() {
        let parts =
            parse_database_url("postgres://admin:pass@db.host.com:5433/production").unwrap();
        assert_eq!(parts.host, "db.host.com");
        assert_eq!(parts.port, 5433);
        assert_eq!(parts.dbname, "production");
        assert_eq!(parts.user, "admin");
    }

    #[test]
    fn test_strip_quotes() {
        assert_eq!(strip_quotes("\"hello\""), "hello");
        assert_eq!(strip_quotes("'world'"), "world");
        assert_eq!(strip_quotes("noquotes"), "noquotes");
    }
}
