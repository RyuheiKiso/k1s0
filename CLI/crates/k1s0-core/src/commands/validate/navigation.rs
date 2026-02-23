use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;

#[derive(Debug, Deserialize, Serialize)]
pub struct NavigationYaml {
    pub version: u32,
    #[serde(default)]
    pub guards: Vec<GuardYaml>,
    pub routes: Vec<RouteYaml>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GuardYaml {
    pub id: String,
    #[serde(rename = "type")]
    pub guard_type: String,
    pub redirect_to: String,
    #[serde(default)]
    pub roles: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RouteYaml {
    pub id: String,
    pub path: String,
    pub component_id: Option<String>,
    #[serde(default)]
    pub guards: Vec<String>,
    pub transition: Option<String>,
    pub redirect_to: Option<String>,
    #[serde(default)]
    pub children: Vec<RouteYaml>,
    #[serde(default)]
    pub params: Vec<ParamYaml>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ParamYaml {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,
}

/// navigation.yaml をバリデーションし、エラー数を返す
pub fn validate_navigation(path: &str) -> Result<usize, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    println!("Checking navigation.yaml...");

    // 1. YAML パース確認
    let nav: NavigationYaml = match serde_yaml::from_str(&content) {
        Ok(n) => {
            println!("  \u{2705} YAML パース OK");
            n
        }
        Err(e) => {
            println!("  \u{274c} YAML パースエラー: {e}");
            return Ok(1);
        }
    };

    let mut errors = 0usize;
    let guard_ids: HashSet<&str> = nav.guards.iter().map(|g| g.id.as_str()).collect();

    // 2. guard 参照の整合性チェック
    let mut guard_ok = true;
    for route in collect_all_routes(&nav.routes) {
        for gid in &route.guards {
            if !guard_ids.contains(gid.as_str()) {
                println!(
                    "  \u{274c} route '{}' が未定義の guard '{}' を参照しています",
                    route.id, gid
                );
                errors += 1;
                guard_ok = false;
            }
        }
    }
    if guard_ok {
        println!("  \u{2705} guard 参照の整合性 OK");
    }

    // 3. route ID の重複なしチェック（子ルートも再帰的に）
    let mut seen_ids = HashSet::new();
    let dup_count = collect_route_ids(&nav.routes, &mut seen_ids);
    errors += dup_count;
    if dup_count == 0 {
        println!("  \u{2705} route ID の重複なし");
    }

    // 4. redirect_to と component_id の排他チェック
    let mut exclusive_ok = true;
    for route in collect_all_routes(&nav.routes) {
        if route.redirect_to.is_some() && route.component_id.is_some() {
            println!(
                "  \u{274c} route '{}' に redirect_to と component_id の両方が指定されています",
                route.id
            );
            errors += 1;
            exclusive_ok = false;
        }
    }
    if exclusive_ok {
        println!("  \u{2705} redirect_to / component_id 排他チェック OK");
    }

    // 5. 循環リダイレクト検出
    let redirect_map: HashMap<&str, &str> = nav
        .routes
        .iter()
        .filter_map(|r| {
            r.redirect_to
                .as_deref()
                .map(|target| (r.path.as_str(), target))
        })
        .collect();
    let mut cycle_ok = true;
    for (start_path, _) in &redirect_map {
        let mut visited = HashSet::new();
        let mut current = *start_path;
        while let Some(&next) = redirect_map.get(current) {
            if !visited.insert(current) {
                println!(
                    "  \u{274c} 循環リダイレクトを検出: '{}' から始まるチェーンが循環しています",
                    start_path
                );
                errors += 1;
                cycle_ok = false;
                break;
            }
            current = next;
        }
    }
    if cycle_ok {
        println!("  \u{2705} 循環リダイレクトなし");
    }

    if errors == 0 {
        println!("\nバリデーション完了: エラーなし");
    } else {
        println!("\nバリデーション完了: {errors} 件のエラー");
    }

    Ok(errors)
}

fn collect_route_ids<'a>(routes: &'a [RouteYaml], ids: &mut HashSet<String>) -> usize {
    let mut duplicates = 0;
    for route in routes {
        if !ids.insert(route.id.clone()) {
            duplicates += 1;
            println!("  \u{274c} route ID '{}' が重複しています", route.id);
        }
        duplicates += collect_route_ids(&route.children, ids);
    }
    duplicates
}

fn collect_all_routes(routes: &[RouteYaml]) -> Vec<&RouteYaml> {
    let mut result = Vec::new();
    for route in routes {
        result.push(route);
        result.extend(collect_all_routes(&route.children));
    }
    result
}

// ============================================================================
// テスト
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_yaml(content: &str) -> NamedTempFile {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(content.as_bytes()).unwrap();
        f
    }

    #[test]
    fn test_validate_valid_navigation() {
        let yaml = r#"
version: 1
guards:
  - id: auth_required
    type: auth_required
    redirect_to: /login
routes:
  - id: root
    path: /
    redirect_to: /dashboard
  - id: login
    path: /login
    component_id: LoginPage
  - id: dashboard
    path: /dashboard
    component_id: DashboardPage
    guards: [auth_required]
    transition: fade
"#;
        let f = write_yaml(yaml);
        let errors = validate_navigation(f.path().to_str().unwrap()).unwrap();
        assert_eq!(errors, 0);
    }

    #[test]
    fn test_validate_undefined_guard() {
        let yaml = r#"
version: 1
guards: []
routes:
  - id: dashboard
    path: /dashboard
    component_id: DashboardPage
    guards: [nonexistent_guard]
"#;
        let f = write_yaml(yaml);
        let errors = validate_navigation(f.path().to_str().unwrap()).unwrap();
        assert!(errors > 0);
    }

    #[test]
    fn test_validate_duplicate_route_ids() {
        let yaml = r#"
version: 1
guards: []
routes:
  - id: home
    path: /
    component_id: HomePage
  - id: home
    path: /home
    component_id: HomePage2
"#;
        let f = write_yaml(yaml);
        let errors = validate_navigation(f.path().to_str().unwrap()).unwrap();
        assert!(errors > 0);
    }

    #[test]
    fn test_validate_duplicate_route_ids_in_children() {
        let yaml = r#"
version: 1
guards: []
routes:
  - id: users
    path: /users
    component_id: UsersPage
    children:
      - id: users
        path: :id
        component_id: UserDetailPage
"#;
        let f = write_yaml(yaml);
        let errors = validate_navigation(f.path().to_str().unwrap()).unwrap();
        assert!(errors > 0);
    }

    #[test]
    fn test_validate_redirect_and_component_exclusive() {
        let yaml = r#"
version: 1
guards: []
routes:
  - id: root
    path: /
    component_id: HomePage
    redirect_to: /dashboard
"#;
        let f = write_yaml(yaml);
        let errors = validate_navigation(f.path().to_str().unwrap()).unwrap();
        assert!(errors > 0);
    }

    #[test]
    fn test_validate_circular_redirect() {
        let yaml = r#"
version: 1
guards: []
routes:
  - id: a
    path: /a
    redirect_to: /b
  - id: b
    path: /b
    redirect_to: /a
"#;
        let f = write_yaml(yaml);
        let errors = validate_navigation(f.path().to_str().unwrap()).unwrap();
        assert!(errors > 0);
    }

    #[test]
    fn test_validate_invalid_yaml() {
        let yaml = "{{{{ invalid yaml ::::";
        let f = write_yaml(yaml);
        let errors = validate_navigation(f.path().to_str().unwrap()).unwrap();
        assert!(errors > 0);
    }
}
