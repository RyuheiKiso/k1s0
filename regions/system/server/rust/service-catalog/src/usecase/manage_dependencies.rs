use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use uuid::Uuid;

use crate::domain::entity::dependency::Dependency;
use crate::domain::repository::DependencyRepository;

/// `ManageDependenciesError` は依存関係管理に関するエラーを表す。
#[derive(Debug, thiserror::Error)]
pub enum ManageDependenciesError {
    #[error("dependency cycle detected involving service: {0}")]
    CycleDetected(Uuid),

    #[error("internal error: {0}")]
    Internal(String),
}

/// `ManageDependenciesUseCase` は依存関係管理ユースケース。
pub struct ManageDependenciesUseCase {
    dep_repo: Arc<dyn DependencyRepository>,
}

impl ManageDependenciesUseCase {
    pub fn new(dep_repo: Arc<dyn DependencyRepository>) -> Self {
        Self { dep_repo }
    }

    /// 指定サービスの依存関係を取得する。
    pub async fn list(&self, service_id: Uuid) -> Result<Vec<Dependency>, ManageDependenciesError> {
        self.dep_repo
            .list_by_service(service_id)
            .await
            .map_err(|e| ManageDependenciesError::Internal(e.to_string()))
    }

    /// 指定サービスの依存関係を設定する。サイクル検出を行う。
    pub async fn set(
        &self,
        service_id: Uuid,
        deps: Vec<Dependency>,
    ) -> Result<(), ManageDependenciesError> {
        // Build the adjacency list from existing dependencies
        let all_deps = self
            .dep_repo
            .get_all_dependencies()
            .await
            .map_err(|e| ManageDependenciesError::Internal(e.to_string()))?;

        let mut adjacency: HashMap<Uuid, HashSet<Uuid>> = HashMap::new();
        for dep in &all_deps {
            // Skip existing deps for this service (we'll replace them)
            if dep.source_service_id == service_id {
                continue;
            }
            adjacency
                .entry(dep.source_service_id)
                .or_default()
                .insert(dep.target_service_id);
        }

        // Add the new deps
        for dep in &deps {
            adjacency
                .entry(dep.source_service_id)
                .or_default()
                .insert(dep.target_service_id);
        }

        // Iterative DFS cycle detection
        if Self::has_cycle(&adjacency) {
            return Err(ManageDependenciesError::CycleDetected(service_id));
        }

        self.dep_repo
            .set_dependencies(service_id, deps)
            .await
            .map_err(|e| ManageDependenciesError::Internal(e.to_string()))
    }

    /// Iterative DFS cycle detection to avoid stack overflow.
    fn has_cycle(adjacency: &HashMap<Uuid, HashSet<Uuid>>) -> bool {
        let all_nodes: HashSet<Uuid> = adjacency
            .iter()
            .flat_map(|(src, targets)| std::iter::once(*src).chain(targets.iter().copied()))
            .collect();

        // 0 = unvisited, 1 = in-progress, 2 = done
        let mut state: HashMap<Uuid, u8> = HashMap::new();

        for &start in &all_nodes {
            if *state.get(&start).unwrap_or(&0) != 0 {
                continue;
            }

            // Stack stores (node, iterator_index) pairs for iterative DFS
            let mut stack: Vec<(Uuid, Vec<Uuid>)> = vec![];
            state.insert(start, 1);

            let neighbors: Vec<Uuid> = adjacency
                .get(&start)
                .map(|s| s.iter().copied().collect())
                .unwrap_or_default();
            stack.push((start, neighbors));

            while let Some((node, remaining)) = stack.last_mut() {
                if let Some(next) = remaining.pop() {
                    let next_state = *state.get(&next).unwrap_or(&0);
                    if next_state == 1 {
                        // Back edge found: cycle
                        return true;
                    }
                    if next_state == 0 {
                        state.insert(next, 1);
                        let next_neighbors: Vec<Uuid> = adjacency
                            .get(&next)
                            .map(|s| s.iter().copied().collect())
                            .unwrap_or_default();
                        stack.push((next, next_neighbors));
                    }
                } else {
                    let finished = *node;
                    state.insert(finished, 2);
                    stack.pop();
                }
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::dependency::DependencyType;
    use crate::domain::repository::dependency_repository::MockDependencyRepository;

    #[test]
    fn test_no_cycle() {
        let mut adj: HashMap<Uuid, HashSet<Uuid>> = HashMap::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();
        adj.entry(a).or_default().insert(b);
        adj.entry(b).or_default().insert(c);
        assert!(!ManageDependenciesUseCase::has_cycle(&adj));
    }

    #[test]
    fn test_cycle_detected() {
        let mut adj: HashMap<Uuid, HashSet<Uuid>> = HashMap::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();
        adj.entry(a).or_default().insert(b);
        adj.entry(b).or_default().insert(c);
        adj.entry(c).or_default().insert(a);
        assert!(ManageDependenciesUseCase::has_cycle(&adj));
    }

    #[test]
    fn test_self_cycle() {
        let mut adj: HashMap<Uuid, HashSet<Uuid>> = HashMap::new();
        let a = Uuid::new_v4();
        adj.entry(a).or_default().insert(a);
        assert!(ManageDependenciesUseCase::has_cycle(&adj));
    }

    #[tokio::test]
    async fn test_set_dependencies_cycle_error() {
        let service_a = Uuid::new_v4();
        let service_b = Uuid::new_v4();

        let mut mock = MockDependencyRepository::new();
        // Existing: B -> A
        mock.expect_get_all_dependencies().returning(move || {
            Ok(vec![Dependency {
                source_service_id: service_b,
                target_service_id: service_a,
                dependency_type: DependencyType::Runtime,
                description: None,
            }])
        });

        let uc = ManageDependenciesUseCase::new(Arc::new(mock));

        // Try to set A -> B, which would create a cycle (A -> B -> A)
        let deps = vec![Dependency {
            source_service_id: service_a,
            target_service_id: service_b,
            dependency_type: DependencyType::Runtime,
            description: None,
        }];

        let result = uc.set(service_a, deps).await;
        assert!(matches!(
            result,
            Err(ManageDependenciesError::CycleDetected(_))
        ));
    }
}
