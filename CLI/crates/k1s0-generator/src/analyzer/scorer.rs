//! スコア計算

use super::structure::calculate_structure_score;
use super::types::{ComplianceScores, DependencyAnalysis, StructureAnalysis, Violation, ViolationSeverity};

/// 各種分析結果からコンプライアンススコアを計算する
pub fn calculate_scores(
    structure: &StructureAnalysis,
    violations: &[Violation],
    dependencies: &DependencyAnalysis,
) -> ComplianceScores {
    let structure_score = calculate_structure_score(structure);
    let convention_score = calculate_convention_score(violations);
    let dependency_score = calculate_dependency_score(dependencies);

    // 総合: structure * 0.4 + convention * 0.4 + dependency * 0.2
    let overall = (f64::from(structure_score) * 0.4
        + f64::from(convention_score) * 0.4
        + f64::from(dependency_score) * 0.2)
        .round()
        .min(100.0) as u32;

    ComplianceScores {
        structure: structure_score,
        convention: convention_score,
        dependency: dependency_score,
        overall,
    }
}

/// 規約スコア: 100 から開始、エラー1件 -5、警告1件 -2
fn calculate_convention_score(violations: &[Violation]) -> u32 {
    let mut score: i32 = 100;

    for v in violations {
        match v.severity {
            ViolationSeverity::Error => score -= 5,
            ViolationSeverity::Warning => score -= 2,
        }
    }

    score.max(0) as u32
}

/// 依存関係スコア: env ファイルや env 変数使用、ハードコードシークレットに応じて減点
fn calculate_dependency_score(deps: &DependencyAnalysis) -> u32 {
    let mut score: i32 = 100;

    // .env ファイルが存在: 1ファイルあたり -10
    score -= (deps.env_files.len() as i32) * 10;

    // ソースコード内の env 変数参照: 1件あたり -3
    score -= (deps.env_var_usages.len() as i32) * 3;

    // ハードコードされたシークレット: 1件あたり -10
    score -= (deps.hardcoded_secrets.len() as i32) * 10;

    score.max(0) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer::types::EnvVarUsage;

    #[test]
    fn test_perfect_score() {
        let structure = StructureAnalysis {
            existing_dirs: vec!["src".to_string()],
            missing_dirs: vec![],
            existing_files: vec!["Cargo.toml".to_string()],
            missing_files: vec![],
            detected_layers: vec![
                "domain".to_string(),
                "application".to_string(),
                "infrastructure".to_string(),
                "presentation".to_string(),
            ],
            missing_layers: vec![],
        };
        let violations = vec![];
        let deps = DependencyAnalysis {
            env_var_usages: vec![],
            hardcoded_secrets: vec![],
            external_dependencies: vec!["axum".to_string()],
            env_files: vec![],
        };

        let scores = calculate_scores(&structure, &violations, &deps);
        assert_eq!(scores.convention, 100);
        assert_eq!(scores.dependency, 100);
        assert_eq!(scores.overall, 100);
    }

    #[test]
    fn test_violations_reduce_convention_score() {
        let structure = StructureAnalysis {
            existing_dirs: vec![],
            missing_dirs: vec![],
            existing_files: vec![],
            missing_files: vec![],
            detected_layers: vec![],
            missing_layers: vec![],
        };
        let violations = vec![
            Violation {
                rule_id: "K020".to_string(),
                severity: ViolationSeverity::Error,
                message: "test".to_string(),
                file: None,
                line: None,
                auto_fixable: false,
            },
            Violation {
                rule_id: "K053".to_string(),
                severity: ViolationSeverity::Warning,
                message: "test".to_string(),
                file: None,
                line: None,
                auto_fixable: false,
            },
        ];
        let deps = DependencyAnalysis {
            env_var_usages: vec![],
            hardcoded_secrets: vec![],
            external_dependencies: vec![],
            env_files: vec![],
        };

        let scores = calculate_scores(&structure, &violations, &deps);
        // 100 - 5 (error) - 2 (warning) = 93
        assert_eq!(scores.convention, 93);
    }

    #[test]
    fn test_env_files_reduce_dependency_score() {
        let structure = StructureAnalysis {
            existing_dirs: vec![],
            missing_dirs: vec![],
            existing_files: vec![],
            missing_files: vec![],
            detected_layers: vec![],
            missing_layers: vec![],
        };
        let deps = DependencyAnalysis {
            env_var_usages: vec![EnvVarUsage {
                file: "src/main.rs".to_string(),
                line: 1,
                pattern: "env::var(".to_string(),
                var_name: None,
            }],
            hardcoded_secrets: vec![],
            external_dependencies: vec![],
            env_files: vec![".env".to_string()],
        };

        let scores = calculate_scores(&structure, &[], &deps);
        // 100 - 10 (.env) - 3 (env var usage) = 87
        assert_eq!(scores.dependency, 87);
    }

    #[test]
    fn test_score_floor_at_zero() {
        let violations: Vec<Violation> = (0..30)
            .map(|i| Violation {
                rule_id: format!("K{:03}", i),
                severity: ViolationSeverity::Error,
                message: "test".to_string(),
                file: None,
                line: None,
                auto_fixable: false,
            })
            .collect();

        let score = calculate_convention_score(&violations);
        assert_eq!(score, 0);
    }
}
