//! ドメイン依存グラフ構築・循環検出・フォーマット

use std::collections::HashMap;

use petgraph::algo::tarjan_scc;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;

use super::types::{DomainGraphError, DomainInfo, FeatureInfo};

/// グラフノードの層
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeLayer {
    Domain,
    Feature,
}

/// グラフノード
#[derive(Debug, Clone)]
pub struct DomainNode {
    /// 名前
    pub name: String,
    /// バージョン
    pub version: String,
    /// 層
    pub layer: NodeLayer,
    /// 非推奨かどうか
    pub deprecated: bool,
}

/// 依存エッジ
#[derive(Debug, Clone)]
pub struct DependencyEdge {
    /// バージョン制約
    pub version_constraint: String,
}

/// ドメイン依存グラフ
#[derive(Debug)]
pub struct DomainGraph {
    graph: DiGraph<DomainNode, DependencyEdge>,
    node_map: HashMap<String, NodeIndex>,
}

impl DomainGraph {
    /// ドメインと feature からグラフを構築する
    pub fn from_domains(domains: &[DomainInfo], features: &[FeatureInfo]) -> Self {
        let mut graph = DiGraph::new();
        let mut node_map = HashMap::new();

        // ドメインノードを追加
        for d in domains {
            let idx = graph.add_node(DomainNode {
                name: d.name.clone(),
                version: d.version.clone(),
                layer: NodeLayer::Domain,
                deprecated: d.deprecated.is_some(),
            });
            node_map.insert(d.name.clone(), idx);
        }

        // feature ノードを追加
        for f in features {
            if !f.domain_dependencies.is_empty() {
                let key = format!("feature:{}", f.name);
                let idx = graph.add_node(DomainNode {
                    name: f.name.clone(),
                    version: String::new(),
                    layer: NodeLayer::Feature,
                    deprecated: false,
                });
                node_map.insert(key, idx);
            }
        }

        // ドメイン間依存エッジ
        for d in domains {
            if let Some(&from_idx) = node_map.get(&d.name) {
                for (dep_name, constraint) in &d.dependencies {
                    if let Some(&to_idx) = node_map.get(dep_name) {
                        graph.add_edge(
                            from_idx,
                            to_idx,
                            DependencyEdge {
                                version_constraint: constraint.clone(),
                            },
                        );
                    }
                }
            }
        }

        // feature -> ドメイン依存エッジ
        for f in features {
            let key = format!("feature:{}", f.name);
            if let Some(&from_idx) = node_map.get(&key) {
                for (dep_name, constraint) in &f.domain_dependencies {
                    if let Some(&to_idx) = node_map.get(dep_name) {
                        graph.add_edge(
                            from_idx,
                            to_idx,
                            DependencyEdge {
                                version_constraint: constraint.clone(),
                            },
                        );
                    }
                }
            }
        }

        Self { graph, node_map }
    }

    /// 循環依存を検出する
    pub fn detect_cycles(&self) -> Vec<Vec<String>> {
        let sccs = tarjan_scc(&self.graph);
        let mut cycles = Vec::new();

        for scc in sccs {
            if scc.len() > 1 {
                // ドメイン層のみの循環を報告
                let names: Vec<String> = scc
                    .iter()
                    .filter_map(|&idx| {
                        let node = &self.graph[idx];
                        if node.layer == NodeLayer::Domain {
                            Some(node.name.clone())
                        } else {
                            None
                        }
                    })
                    .collect();
                if names.len() > 1 {
                    cycles.push(names);
                }
            }
        }

        cycles
    }

    /// 指定ノードを起点とした部分グラフを構築する
    pub fn subgraph(&self, root: &str) -> Result<Self, DomainGraphError> {
        // ドメインノードまたは feature ノードを探す
        let root_idx = self
            .node_map
            .get(root)
            .or_else(|| self.node_map.get(&format!("feature:{}", root)))
            .ok_or_else(|| DomainGraphError::NotFound(root.to_string()))?;

        // BFS で到達可能ノードを収集
        let mut visited = HashMap::new();
        let mut queue = vec![*root_idx];
        while let Some(idx) = queue.pop() {
            if visited.contains_key(&idx) {
                continue;
            }
            visited.insert(idx, ());
            for edge in self.graph.edges(idx) {
                queue.push(edge.target());
            }
        }

        // サブグラフ構築
        let mut new_graph = DiGraph::new();
        let mut new_node_map = HashMap::new();
        let mut old_to_new: HashMap<NodeIndex, NodeIndex> = HashMap::new();

        for &old_idx in visited.keys() {
            let node = self.graph[old_idx].clone();
            let new_idx = new_graph.add_node(node.clone());
            old_to_new.insert(old_idx, new_idx);

            let key = if node.layer == NodeLayer::Feature {
                format!("feature:{}", node.name)
            } else {
                node.name.clone()
            };
            new_node_map.insert(key, new_idx);
        }

        for &old_idx in visited.keys() {
            for edge in self.graph.edges(old_idx) {
                if let (Some(&new_from), Some(&new_to)) =
                    (old_to_new.get(&old_idx), old_to_new.get(&edge.target()))
                {
                    new_graph.add_edge(new_from, new_to, edge.weight().clone());
                }
            }
        }

        Ok(Self {
            graph: new_graph,
            node_map: new_node_map,
        })
    }

    /// Mermaid 形式で出力する
    pub fn to_mermaid(&self) -> String {
        let mut lines = vec!["graph TD".to_string()];

        // ドメイン層サブグラフ
        let domain_nodes: Vec<_> = self
            .graph
            .node_indices()
            .filter(|&idx| self.graph[idx].layer == NodeLayer::Domain)
            .collect();

        if !domain_nodes.is_empty() {
            lines.push("    subgraph Domain Layer".to_string());
            for &idx in &domain_nodes {
                let node = &self.graph[idx];
                let id = node_id(&node.name);
                let label = format!("{}<br/>v{}", node.name, node.version);
                if node.deprecated {
                    lines.push(format!("        {}[\"{}\"]\n        style {} fill:#ffcccc,stroke:#cc0000", id, label, id));
                } else {
                    lines.push(format!("        {}[\"{}\"]\n        style {} fill:#ccffcc", id, label, id));
                }
            }
            lines.push("    end".to_string());
        }

        // feature 層サブグラフ
        let feature_nodes: Vec<_> = self
            .graph
            .node_indices()
            .filter(|&idx| self.graph[idx].layer == NodeLayer::Feature)
            .collect();

        if !feature_nodes.is_empty() {
            lines.push("    subgraph Feature Layer".to_string());
            for &idx in &feature_nodes {
                let node = &self.graph[idx];
                let id = format!("f_{}", node_id(&node.name));
                lines.push(format!("        {}(\"{}\")", id, node.name));
            }
            lines.push("    end".to_string());
        }

        // エッジ
        for edge_ref in self.graph.edge_references() {
            let source = &self.graph[edge_ref.source()];
            let target = &self.graph[edge_ref.target()];
            let from_id = if source.layer == NodeLayer::Feature {
                format!("f_{}", node_id(&source.name))
            } else {
                node_id(&source.name)
            };
            let to_id = if target.layer == NodeLayer::Feature {
                format!("f_{}", node_id(&target.name))
            } else {
                node_id(&target.name)
            };
            let constraint = &edge_ref.weight().version_constraint;
            if constraint.is_empty() {
                lines.push(format!("    {} --> {}", from_id, to_id));
            } else {
                lines.push(format!("    {} -->|\"{}\"| {}", from_id, constraint, to_id));
            }
        }

        lines.join("\n")
    }

    /// DOT 形式で出力する
    pub fn to_dot(&self) -> String {
        let mut lines = vec![
            "digraph domain_dependencies {".to_string(),
            "    rankdir=TB;".to_string(),
            "    node [shape=box, fontname=\"Arial\"];".to_string(),
            "    edge [fontname=\"Arial\", fontsize=10];".to_string(),
        ];

        // ドメイン層クラスタ
        let domain_nodes: Vec<_> = self
            .graph
            .node_indices()
            .filter(|&idx| self.graph[idx].layer == NodeLayer::Domain)
            .collect();

        if !domain_nodes.is_empty() {
            lines.push("    subgraph cluster_domain {".to_string());
            lines.push("        label=\"Domain Layer\";".to_string());
            lines.push("        style=dashed;".to_string());
            for &idx in &domain_nodes {
                let node = &self.graph[idx];
                let id = node_id(&node.name);
                let label = format!("{}\\nv{}", node.name, node.version);
                if node.deprecated {
                    lines.push(format!(
                        "        {} [label=\"{}\", style=filled, fillcolor=\"#ffcccc\", fontcolor=\"#cc0000\"];",
                        id, label
                    ));
                } else {
                    lines.push(format!(
                        "        {} [label=\"{}\", style=filled, fillcolor=\"#ccffcc\"];",
                        id, label
                    ));
                }
            }
            lines.push("    }".to_string());
        }

        // feature 層クラスタ
        let feature_nodes: Vec<_> = self
            .graph
            .node_indices()
            .filter(|&idx| self.graph[idx].layer == NodeLayer::Feature)
            .collect();

        if !feature_nodes.is_empty() {
            lines.push("    subgraph cluster_feature {".to_string());
            lines.push("        label=\"Feature Layer\";".to_string());
            lines.push("        style=dashed;".to_string());
            for &idx in &feature_nodes {
                let node = &self.graph[idx];
                let id = format!("f_{}", node_id(&node.name));
                lines.push(format!(
                    "        {} [label=\"{}\", style=filled, fillcolor=\"#cce5ff\"];",
                    id, node.name
                ));
            }
            lines.push("    }".to_string());
        }

        // エッジ
        for edge_ref in self.graph.edge_references() {
            let source = &self.graph[edge_ref.source()];
            let target = &self.graph[edge_ref.target()];
            let from_id = if source.layer == NodeLayer::Feature {
                format!("f_{}", node_id(&source.name))
            } else {
                node_id(&source.name)
            };
            let to_id = if target.layer == NodeLayer::Feature {
                format!("f_{}", node_id(&target.name))
            } else {
                node_id(&target.name)
            };
            let constraint = &edge_ref.weight().version_constraint;
            if constraint.is_empty() {
                lines.push(format!("    {} -> {};", from_id, to_id));
            } else {
                lines.push(format!(
                    "    {} -> {} [label=\"{}\"];",
                    from_id, to_id, constraint
                ));
            }
        }

        lines.push("}".to_string());
        lines.join("\n")
    }
}

/// ノード名を Mermaid/DOT 用の ID に変換する
fn node_id(name: &str) -> String {
    name.replace('-', "_")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn domain(name: &str, deps: &[(&str, &str)]) -> DomainInfo {
        let mut dependencies = HashMap::new();
        for (d, v) in deps {
            dependencies.insert(d.to_string(), v.to_string());
        }
        DomainInfo {
            name: name.to_string(),
            version: "0.1.0".to_string(),
            domain_type: "backend-rust".to_string(),
            language: "rust".to_string(),
            path: PathBuf::from(format!("domain/backend/rust/{}", name)),
            dependencies,
            min_framework_version: None,
            deprecated: None,
            breaking_changes: None,
        }
    }

    fn feature(name: &str, deps: &[(&str, &str)]) -> FeatureInfo {
        let mut domain_dependencies = HashMap::new();
        for (d, v) in deps {
            domain_dependencies.insert(d.to_string(), v.to_string());
        }
        FeatureInfo {
            name: name.to_string(),
            feature_type: "backend-rust".to_string(),
            path: PathBuf::from(format!("feature/backend/rust/{}", name)),
            domain_dependencies,
        }
    }

    #[test]
    fn test_no_cycles() {
        let domains = vec![
            domain("a", &[("b", "^0.1.0")]),
            domain("b", &[]),
        ];
        let graph = DomainGraph::from_domains(&domains, &[]);
        let cycles = graph.detect_cycles();
        assert!(cycles.is_empty());
    }

    #[test]
    fn test_detect_cycle() {
        let domains = vec![
            domain("a", &[("b", "^0.1.0")]),
            domain("b", &[("c", "^0.1.0")]),
            domain("c", &[("a", "^0.1.0")]),
        ];
        let graph = DomainGraph::from_domains(&domains, &[]);
        let cycles = graph.detect_cycles();
        assert_eq!(cycles.len(), 1);
        assert_eq!(cycles[0].len(), 3);
    }

    #[test]
    fn test_subgraph() {
        let domains = vec![
            domain("a", &[("b", "^0.1.0")]),
            domain("b", &[]),
            domain("c", &[]),
        ];
        let graph = DomainGraph::from_domains(&domains, &[]);
        let sub = graph.subgraph("a").unwrap();
        // a と b のみ
        assert_eq!(sub.graph.node_count(), 2);
    }

    #[test]
    fn test_to_mermaid() {
        let domains = vec![
            domain("a", &[("b", "^0.1.0")]),
            domain("b", &[]),
        ];
        let features = vec![feature("svc", &[("a", "^0.1.0")])];
        let graph = DomainGraph::from_domains(&domains, &features);
        let mermaid = graph.to_mermaid();
        assert!(mermaid.contains("graph TD"));
        assert!(mermaid.contains("Domain Layer"));
        assert!(mermaid.contains("Feature Layer"));
    }

    #[test]
    fn test_to_dot() {
        let domains = vec![
            domain("a", &[("b", "^0.1.0")]),
            domain("b", &[]),
        ];
        let graph = DomainGraph::from_domains(&domains, &[]);
        let dot = graph.to_dot();
        assert!(dot.contains("digraph domain_dependencies"));
        assert!(dot.contains("cluster_domain"));
    }

    #[test]
    fn test_subgraph_not_found() {
        let domains = vec![domain("a", &[])];
        let graph = DomainGraph::from_domains(&domains, &[]);
        assert!(graph.subgraph("nonexistent").is_err());
    }
}
