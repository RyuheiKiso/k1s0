use super::ValidationDiagnostic;
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

/// Collect validation diagnostics for `navigation.yaml`.
///
/// # Errors
///
/// Returns an error when the navigation file cannot be read.
pub fn collect_navigation_diagnostics(
    path: &str,
) -> Result<Vec<ValidationDiagnostic>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let nav: NavigationYaml = match serde_yaml::from_str(&content) {
        Ok(nav) => nav,
        Err(error) => {
            return Ok(vec![ValidationDiagnostic {
                rule: "yaml-parse".to_string(),
                path: "$".to_string(),
                message: error.to_string(),
                line: error.location().map(|location| location.line()),
            }]);
        }
    };

    let mut diagnostics = Vec::new();
    let guard_ids: HashSet<&str> = nav.guards.iter().map(|guard| guard.id.as_str()).collect();
    let all_routes = collect_all_routes(&nav.routes);

    for (path, route) in &all_routes {
        for guard_id in &route.guards {
            if !guard_ids.contains(guard_id.as_str()) {
                diagnostics.push(ValidationDiagnostic {
                    rule: "undefined-guard".to_string(),
                    path: format!("{path}.guards"),
                    message: format!(
                        "route '{}' references undefined guard '{}'",
                        route.id, guard_id
                    ),
                    line: None,
                });
            }
        }

        if route.redirect_to.is_some() && route.component_id.is_some() {
            diagnostics.push(ValidationDiagnostic {
                rule: "exclusive-route-target".to_string(),
                path: path.clone(),
                message: format!(
                    "route '{}' cannot define both redirect_to and component_id",
                    route.id
                ),
                line: None,
            });
        }
    }

    let mut seen_ids = HashSet::new();
    collect_route_id_diagnostics(&nav.routes, "routes", &mut seen_ids, &mut diagnostics);
    collect_redirect_cycle_diagnostics(&all_routes, &mut diagnostics);

    Ok(diagnostics)
}

/// Validate `navigation.yaml` and print a summary.
///
/// # Errors
///
/// Returns an error when diagnostics cannot be collected.
pub fn validate_navigation(path: &str) -> Result<usize, Box<dyn std::error::Error>> {
    println!("Checking navigation.yaml...");
    let diagnostics = collect_navigation_diagnostics(path)?;

    if diagnostics.is_empty() {
        println!("  OK YAML parse");
        println!("  OK guard references");
        println!("  OK route ids");
        println!("  OK redirect/component exclusivity");
        println!("  OK redirect graph");
    } else {
        for diagnostic in &diagnostics {
            println!(
                "  ERROR [{}] {}: {}",
                diagnostic.rule, diagnostic.path, diagnostic.message
            );
        }
    }

    if diagnostics.is_empty() {
        println!("\nValidation succeeded. No errors found.");
    } else {
        println!("\nValidation failed. {} error(s) found.", diagnostics.len());
    }

    Ok(diagnostics.len())
}

fn collect_route_id_diagnostics(
    routes: &[RouteYaml],
    base_path: &str,
    seen_ids: &mut HashSet<String>,
    diagnostics: &mut Vec<ValidationDiagnostic>,
) {
    for (index, route) in routes.iter().enumerate() {
        let route_path = format!("{base_path}[{index}]");
        if !seen_ids.insert(route.id.clone()) {
            diagnostics.push(ValidationDiagnostic {
                rule: "duplicate-route-id".to_string(),
                path: format!("{route_path}.id"),
                message: format!("route id '{}' is duplicated", route.id),
                line: None,
            });
        }
        collect_route_id_diagnostics(
            &route.children,
            &format!("{route_path}.children"),
            seen_ids,
            diagnostics,
        );
    }
}

fn collect_all_routes(routes: &[RouteYaml]) -> Vec<(String, &RouteYaml)> {
    let mut result = Vec::new();
    collect_routes_recursive(routes, "routes", &mut result);
    result
}

fn collect_routes_recursive<'a>(
    routes: &'a [RouteYaml],
    base_path: &str,
    result: &mut Vec<(String, &'a RouteYaml)>,
) {
    for (index, route) in routes.iter().enumerate() {
        let route_path = format!("{base_path}[{index}]");
        result.push((route_path.clone(), route));
        collect_routes_recursive(&route.children, &format!("{route_path}.children"), result);
    }
}

fn collect_redirect_cycle_diagnostics(
    all_routes: &[(String, &RouteYaml)],
    diagnostics: &mut Vec<ValidationDiagnostic>,
) {
    let redirect_map: HashMap<&str, (&str, &str)> = all_routes
        .iter()
        .filter_map(|(path, route)| {
            route
                .redirect_to
                .as_deref()
                .map(|redirect_to| (route.path.as_str(), (redirect_to, path.as_str())))
        })
        .collect();

    for start_path in redirect_map.keys() {
        let mut visited = HashSet::new();
        let mut current = *start_path;

        while let Some((next, route_path)) = redirect_map.get(current).copied() {
            if !visited.insert(current) {
                diagnostics.push(ValidationDiagnostic {
                    rule: "redirect-cycle".to_string(),
                    path: route_path.to_string(),
                    message: format!("redirect cycle detected starting from '{start_path}'"),
                    line: None,
                });
                break;
            }
            current = next;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_yaml(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file
    }

    #[test]
    fn test_validate_valid_navigation() {
        let yaml = r"
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
";
        let file = write_yaml(yaml);
        let errors = validate_navigation(file.path().to_str().unwrap()).unwrap();
        assert_eq!(errors, 0);
    }

    #[test]
    fn test_validate_undefined_guard() {
        let yaml = r"
version: 1
guards: []
routes:
  - id: dashboard
    path: /dashboard
    component_id: DashboardPage
    guards: [nonexistent_guard]
";
        let file = write_yaml(yaml);
        let errors = validate_navigation(file.path().to_str().unwrap()).unwrap();
        assert!(errors > 0);
    }

    #[test]
    fn test_validate_duplicate_route_ids() {
        let yaml = r"
version: 1
guards: []
routes:
  - id: home
    path: /
    component_id: HomePage
  - id: home
    path: /home
    component_id: HomePage2
";
        let file = write_yaml(yaml);
        let errors = validate_navigation(file.path().to_str().unwrap()).unwrap();
        assert!(errors > 0);
    }

    #[test]
    fn test_validate_duplicate_route_ids_in_children() {
        let yaml = r"
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
";
        let file = write_yaml(yaml);
        let errors = validate_navigation(file.path().to_str().unwrap()).unwrap();
        assert!(errors > 0);
    }

    #[test]
    fn test_validate_redirect_and_component_exclusive() {
        let yaml = r"
version: 1
guards: []
routes:
  - id: root
    path: /
    component_id: HomePage
    redirect_to: /dashboard
";
        let file = write_yaml(yaml);
        let errors = validate_navigation(file.path().to_str().unwrap()).unwrap();
        assert!(errors > 0);
    }

    #[test]
    fn test_validate_circular_redirect() {
        let yaml = r"
version: 1
guards: []
routes:
  - id: a
    path: /a
    redirect_to: /b
  - id: b
    path: /b
    redirect_to: /a
";
        let file = write_yaml(yaml);
        let errors = validate_navigation(file.path().to_str().unwrap()).unwrap();
        assert!(errors > 0);
    }

    #[test]
    fn test_validate_invalid_yaml() {
        let yaml = "{{{{ invalid yaml ::::";
        let file = write_yaml(yaml);
        let errors = validate_navigation(file.path().to_str().unwrap()).unwrap();
        assert!(errors > 0);
    }
}
