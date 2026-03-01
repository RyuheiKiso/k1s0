use super::super::validate::navigation::NavigationYaml;

/// TypeScript の route-types.ts を生成する
pub fn generate_typescript_routes(nav: &NavigationYaml) -> String {
    let mut out = String::new();
    out.push_str(
        "// src/navigation/__generated__/route-types.ts\n\
         // このファイルは CLI が自動生成する。直接編集しないこと。\n\
         // k1s0 generate navigation で再生成できます。\n\n",
    );

    // RouteIds const
    let all_routes = collect_all_routes_flat(&nav.routes);

    out.push_str("export const RouteIds = {\n");
    for route in &all_routes {
        let const_name = route.id.to_uppercase();
        out.push_str(&format!("  {const_name}: '{}',\n", route.id));
    }
    out.push_str("} as const;\n\n");

    out.push_str("export type RouteId = typeof RouteIds[keyof typeof RouteIds];\n\n");

    // RouteParams type
    out.push_str("export type RouteParams = {\n");
    for route in &all_routes {
        if route.params.is_empty() {
            out.push_str(&format!(
                "  {}: Record<string, never>;\n",
                route.id
            ));
        } else {
            out.push_str(&format!("  {}: {{ ", route.id));
            let params: Vec<String> = route
                .params
                .iter()
                .map(|p| {
                    let ts_type = match p.param_type.as_str() {
                        "int" => "number",
                        "uuid" | "string" => "string",
                        _ => "string",
                    };
                    format!("{}: {ts_type}", p.name)
                })
                .collect();
            out.push_str(&params.join("; "));
            out.push_str(" };\n");
        }
    }
    out.push_str("};\n");

    out
}

/// Dart の route_ids.dart を生成する
pub fn generate_dart_routes(nav: &NavigationYaml) -> String {
    let mut out = String::new();
    out.push_str(
        "// lib/navigation/__generated__/route_ids.dart\n\
         // このファイルは CLI が自動生成する。直接編集しないこと。\n\
         // k1s0 generate navigation で再生成できます。\n\n",
    );

    let all_routes = collect_all_routes_flat(&nav.routes);

    // enum RouteId
    out.push_str("enum RouteId {\n");
    for (i, route) in all_routes.iter().enumerate() {
        let camel = to_camel_case(&route.id);
        if i < all_routes.len() - 1 {
            out.push_str(&format!("  {camel},\n"));
        } else {
            out.push_str(&format!("  {camel};\n"));
        }
    }

    out.push_str("\n  String get path => switch (this) {\n");
    for route in &all_routes {
        let camel = to_camel_case(&route.id);
        out.push_str(&format!(
            "    RouteId.{camel} => '{}',\n",
            route.full_path
        ));
    }
    out.push_str("  };\n");
    out.push_str("}\n");

    out
}

/// ルート情報をフラットに収集するための中間構造体
struct FlatRoute {
    id: String,
    full_path: String,
    params: Vec<FlatParam>,
}

struct FlatParam {
    name: String,
    param_type: String,
}

fn collect_all_routes_flat(routes: &[super::super::validate::navigation::RouteYaml]) -> Vec<FlatRoute> {
    let mut result = Vec::new();
    collect_routes_recursive(routes, "", &mut result);
    result
}

fn collect_routes_recursive(
    routes: &[super::super::validate::navigation::RouteYaml],
    parent_path: &str,
    out: &mut Vec<FlatRoute>,
) {
    for route in routes {
        let full_path = if route.path.starts_with('/') || parent_path.is_empty() {
            route.path.clone()
        } else {
            format!("{}/{}", parent_path.trim_end_matches('/'), route.path)
        };

        out.push(FlatRoute {
            id: route.id.clone(),
            full_path: full_path.clone(),
            params: route
                .params
                .iter()
                .map(|p| FlatParam {
                    name: p.name.clone(),
                    param_type: p.param_type.clone(),
                })
                .collect(),
        });

        collect_routes_recursive(&route.children, &full_path, out);
    }
}

fn to_camel_case(snake: &str) -> String {
    let mut result = String::new();
    let mut capitalize = false;
    for ch in snake.chars() {
        if ch == '_' {
            capitalize = true;
        } else if capitalize {
            result.push(ch.to_uppercase().next().unwrap_or(ch));
            capitalize = false;
        } else {
            result.push(ch);
        }
    }
    result
}

/// ファイルパスから TypeScript ルート定義を生成する
pub fn generate_typescript_routes_from_file(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let nav: NavigationYaml = serde_yaml::from_str(&content)?;
    Ok(generate_typescript_routes(&nav))
}

/// ファイルパスから Dart ルート定義を生成する
pub fn generate_dart_routes_from_file(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let nav: NavigationYaml = serde_yaml::from_str(&content)?;
    Ok(generate_dart_routes(&nav))
}

// ============================================================================
// テスト
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::validate::navigation::{
        GuardYaml, NavigationYaml, ParamYaml, RouteYaml,
    };

    fn sample_navigation() -> NavigationYaml {
        NavigationYaml {
            version: 1,
            guards: vec![GuardYaml {
                id: "auth_required".to_string(),
                guard_type: "auth_required".to_string(),
                redirect_to: "/login".to_string(),
                roles: vec![],
            }],
            routes: vec![
                RouteYaml {
                    id: "root".to_string(),
                    path: "/".to_string(),
                    component_id: None,
                    guards: vec![],
                    transition: None,
                    redirect_to: Some("/dashboard".to_string()),
                    children: vec![],
                    params: vec![],
                },
                RouteYaml {
                    id: "login".to_string(),
                    path: "/login".to_string(),
                    component_id: Some("LoginPage".to_string()),
                    guards: vec![],
                    transition: None,
                    redirect_to: None,
                    children: vec![],
                    params: vec![],
                },
                RouteYaml {
                    id: "dashboard".to_string(),
                    path: "/dashboard".to_string(),
                    component_id: Some("DashboardPage".to_string()),
                    guards: vec!["auth_required".to_string()],
                    transition: Some("fade".to_string()),
                    redirect_to: None,
                    children: vec![],
                    params: vec![],
                },
                RouteYaml {
                    id: "users".to_string(),
                    path: "/users".to_string(),
                    component_id: Some("UsersPage".to_string()),
                    guards: vec!["auth_required".to_string()],
                    transition: Some("slide".to_string()),
                    redirect_to: None,
                    children: vec![RouteYaml {
                        id: "user_detail".to_string(),
                        path: ":id".to_string(),
                        component_id: Some("UserDetailPage".to_string()),
                        guards: vec!["auth_required".to_string()],
                        transition: None,
                        redirect_to: None,
                        children: vec![],
                        params: vec![ParamYaml {
                            name: "id".to_string(),
                            param_type: "string".to_string(),
                        }],
                    }],
                    params: vec![],
                },
            ],
        }
    }

    #[test]
    fn test_generate_typescript_routes() {
        let nav = sample_navigation();
        let ts = generate_typescript_routes(&nav);
        assert!(ts.contains("export const RouteIds"));
        assert!(ts.contains("ROOT: 'root'"));
        assert!(ts.contains("LOGIN: 'login'"));
        assert!(ts.contains("DASHBOARD: 'dashboard'"));
        assert!(ts.contains("USERS: 'users'"));
        assert!(ts.contains("USER_DETAIL: 'user_detail'"));
        assert!(ts.contains("export type RouteId"));
        assert!(ts.contains("export type RouteParams"));
        assert!(ts.contains("user_detail: { id: string }"));
        assert!(ts.contains("root: Record<string, never>"));
    }

    #[test]
    fn test_generate_dart_routes() {
        let nav = sample_navigation();
        let dart = generate_dart_routes(&nav);
        assert!(dart.contains("enum RouteId"));
        assert!(dart.contains("root,"));
        assert!(dart.contains("login,"));
        assert!(dart.contains("dashboard,"));
        assert!(dart.contains("users,"));
        assert!(dart.contains("userDetail;"));
        assert!(dart.contains("String get path"));
        assert!(dart.contains("RouteId.root => '/'"));
        assert!(dart.contains("RouteId.userDetail => '/users/:id'"));
    }

    #[test]
    fn test_to_camel_case() {
        assert_eq!(to_camel_case("user_detail"), "userDetail");
        assert_eq!(to_camel_case("not_found"), "notFound");
        assert_eq!(to_camel_case("root"), "root");
        assert_eq!(to_camel_case("a_b_c"), "aBC");
    }
}
