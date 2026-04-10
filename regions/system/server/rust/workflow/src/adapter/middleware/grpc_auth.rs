// HIGH-004 対応: 独自実装を廃止し server-common の GrpcAuthLayer を使用する。
// Health Check バイパスと Tier 検証は server-common 側で実装されている。
pub use k1s0_server_common::middleware::grpc_auth::GrpcAuthLayer;

/// gRPC メソッド名を RBAC アクション文字列に変換する。
/// server-common の GrpcAuthLayer に action_mapper として注入する。
/// read: 参照系、write: 作成・更新・承認・却下・再割当系、admin: 削除・キャンセル系
pub fn action_mapper(method: &str) -> &'static str {
    match method {
        "ListWorkflows" | "GetWorkflow" | "GetInstance" | "ListInstances" | "ListTasks" => "read",
        "DeleteWorkflow" | "CancelInstance" => "admin",
        _ => "write",
    }
}

#[cfg(test)]
mod tests {
    use k1s0_auth::claims::{Audience, RealmAccess};
    use k1s0_auth::Claims;
    use k1s0_server_common::middleware::rbac::check_permission;
    use k1s0_server_common::middleware::rbac::Tier;

    use super::*;

    fn make_claims(role_names: &[&str]) -> Claims {
        Claims {
            sub: "user-uuid-1234".to_string(),
            iss: "https://auth.k1s0.internal.example.com/realms/k1s0".to_string(),
            aud: Audience(vec!["k1s0-api".to_string()]),
            exp: 9999999999,
            iat: 1000000000,
            jti: None,
            typ: None,
            azp: None,
            scope: None,
            preferred_username: Some("taro.yamada".to_string()),
            email: Some("taro@example.com".to_string()),
            realm_access: Some(RealmAccess {
                roles: role_names.iter().map(|s| s.to_string()).collect(),
            }),
            resource_access: None,
            tier_access: None,
            tenant_id: String::new(),
        }
    }

    #[test]
    fn extracts_method_name_via_action_mapper() {
        assert_eq!(action_mapper("ListWorkflows"), "read");
    }

    #[test]
    fn maps_grpc_method_to_action() {
        assert_eq!(action_mapper("ListWorkflows"), "read");
        assert_eq!(action_mapper("CancelInstance"), "admin");
        assert_eq!(action_mapper("ApproveTask"), "write");
    }

    #[test]
    fn authorizes_using_same_role_mapping_as_rest() {
        assert!(check_permission(
            Tier::System,
            make_claims(&["sys_auditor"]).realm_roles(),
            "read"
        ));
        assert!(!check_permission(
            Tier::System,
            make_claims(&["sys_auditor"]).realm_roles(),
            "write"
        ));
        assert!(check_permission(
            Tier::System,
            make_claims(&["sys_operator"]).realm_roles(),
            "write"
        ));
        assert!(!check_permission(
            Tier::System,
            make_claims(&["sys_operator"]).realm_roles(),
            "admin"
        ));
        assert!(check_permission(
            Tier::System,
            make_claims(&["sys_admin"]).realm_roles(),
            "admin"
        ));
    }
}
