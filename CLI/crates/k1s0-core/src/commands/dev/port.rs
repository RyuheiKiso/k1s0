/// ポート管理モジュール。
///
/// デフォルトポートの定義と、使用中ポートの検出・空きポート解決を行う。
use std::net::TcpListener;

use super::types::PortAssignments;

/// デフォルトのポート割り当てを返す。
pub fn default_ports() -> PortAssignments {
    PortAssignments {
        postgres: 5432,
        kafka: 9092,
        redis: 6379,
        redis_session: 6380,
        pgadmin: 5050,
        kafka_ui: 8081,
        keycloak: 8180,
    }
}

/// 指定ポートが使用中かどうかを確認する。
///
/// `TcpListener::bind` を試みて、バインドに失敗した場合は使用中と判定する。
pub fn is_port_in_use(port: u16) -> bool {
    TcpListener::bind(("127.0.0.1", port)).is_err()
}

/// 使用中のポートを検出し、空きポートを解決する。
///
/// デフォルトポートが使用中の場合、+1 して空きポートを探す。
/// 最大100回まで試行する。
pub fn resolve_ports(defaults: &PortAssignments) -> PortAssignments {
    PortAssignments {
        postgres: find_available_port(defaults.postgres),
        kafka: find_available_port(defaults.kafka),
        redis: find_available_port(defaults.redis),
        redis_session: find_available_port(defaults.redis_session),
        pgadmin: find_available_port(defaults.pgadmin),
        kafka_ui: find_available_port(defaults.kafka_ui),
        keycloak: find_available_port(defaults.keycloak),
    }
}

/// 指定ポートから始めて、空いているポートを見つける。
///
/// 最大100回まで +1 して試行する。
fn find_available_port(start: u16) -> u16 {
    for offset in 0..100 {
        let port = start + offset;
        if !is_port_in_use(port) {
            return port;
        }
    }
    // 見つからなかった場合はデフォルトを返す（docker compose が起動時にエラーになる）
    start
}

// テストコードでは unwrap() の使用を許可する
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// デフォルトポートが正しいことを確認する。
    #[test]
    fn test_default_ports() {
        let ports = default_ports();
        assert_eq!(ports.postgres, 5432);
        assert_eq!(ports.kafka, 9092);
        assert_eq!(ports.redis, 6379);
        assert_eq!(ports.redis_session, 6380);
        assert_eq!(ports.pgadmin, 5050);
        assert_eq!(ports.kafka_ui, 8081);
        assert_eq!(ports.keycloak, 8180);
    }

    /// 未使用のポートは `is_port_in_use` で false を返す。
    #[test]
    fn test_is_port_in_use_free_port() {
        // 高番号のポートは通常空いている
        assert!(!is_port_in_use(59123));
    }

    /// 使用中のポートは `is_port_in_use` で true を返す。
    #[test]
    fn test_is_port_in_use_occupied() {
        // ポートをバインドしてから確認
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        assert!(is_port_in_use(port));
    }

    /// `resolve_ports` はデフォルトポートが空いていればそのまま返す。
    #[test]
    fn test_resolve_ports_uses_defaults_when_free() {
        // 高番号のポートをデフォルトにして確認
        let defaults = PortAssignments {
            postgres: 59200,
            kafka: 59201,
            redis: 59202,
            redis_session: 59203,
            pgadmin: 59204,
            kafka_ui: 59205,
            keycloak: 59206,
        };

        let resolved = resolve_ports(&defaults);
        assert_eq!(resolved, defaults);
    }

    /// `resolve_ports` は使用中のポートを回避する。
    #[test]
    fn test_resolve_ports_avoids_occupied() {
        // ポートを占有
        let listener = TcpListener::bind("127.0.0.1:59300").unwrap();
        let _port = listener.local_addr().unwrap().port();

        let defaults = PortAssignments {
            postgres: 59300, // 占有中
            kafka: 59400,
            redis: 59401,
            redis_session: 59402,
            pgadmin: 59403,
            kafka_ui: 59404,
            keycloak: 59405,
        };

        let resolved = resolve_ports(&defaults);
        // PostgreSQL のポートは 59300 以外になるはず
        assert_ne!(resolved.postgres, 59300);
        assert!(resolved.postgres > 59300);
    }

    /// `find_available_port` は使用中のポートをスキップする。
    #[test]
    fn test_find_available_port() {
        let port = find_available_port(59500);
        // 通常は 59500 そのものが返る
        assert!(port >= 59500);
    }
}
