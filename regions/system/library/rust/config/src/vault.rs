use std::collections::HashMap;

use crate::Config;

/// Vault シークレットで設定値を上書きする。
/// 異なるハッシャーを持つ `HashMap` にも対応するため、型パラメータを汎化する。
pub fn merge_vault_secrets<S: std::hash::BuildHasher>(config: &mut Config, secrets: &HashMap<String, String, S>) {
    if let Some(v) = secrets.get("database.password") {
        if let Some(ref mut db) = config.database {
            // clone_from はアロケーション再利用のため clone よりも効率的
            db.password.clone_from(v);
        }
    }
    if let Some(v) = secrets.get("redis.password") {
        if let Some(ref mut redis) = config.redis {
            redis.password = Some(v.clone());
        }
    }
    if let Some(v) = secrets.get("kafka.sasl.username") {
        if let Some(ref mut kafka) = config.kafka {
            if let Some(ref mut sasl) = kafka.sasl {
                // clone_from はアロケーション再利用のため clone よりも効率的
                sasl.username.clone_from(v);
            }
        }
    }
    if let Some(v) = secrets.get("kafka.sasl.password") {
        if let Some(ref mut kafka) = config.kafka {
            if let Some(ref mut sasl) = kafka.sasl {
                // clone_from はアロケーション再利用のため clone よりも効率的
                sasl.password.clone_from(v);
            }
        }
    }
    if let Some(v) = secrets.get("redis_session.password") {
        if let Some(ref mut redis_session) = config.redis_session {
            redis_session.password = Some(v.clone());
        }
    }
    if let Some(v) = secrets.get("auth.oidc.client_secret") {
        if let Some(ref mut oidc) = config.auth.oidc {
            oidc.client_secret = Some(v.clone());
        }
    }
}
