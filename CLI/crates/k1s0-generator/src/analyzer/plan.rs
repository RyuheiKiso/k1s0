//! 移行計画生成

use super::types::{
    AnalysisResult, MigrationAction, MigrationPhase, MigrationPlan, MigrationStep, StepRisk,
    StepStatus,
};

/// 分析結果から移行計画を生成する
pub fn generate_migration_plan(analysis: &AnalysisResult, feature_name: &str) -> MigrationPlan {
    let template_type = analysis.project_type.to_string();
    let mut phases = Vec::new();
    let mut step_counter = 0u32;

    // Phase 1: Backup
    phases.push(generate_backup_phase(&mut step_counter));

    // Phase 2: Structure
    phases.push(generate_structure_phase(analysis, &mut step_counter));

    // Phase 3: Management Files
    phases.push(generate_management_files_phase(
        feature_name,
        &template_type,
        &mut step_counter,
    ));

    // Phase 4: Convention Fixes
    phases.push(generate_convention_fixes_phase(analysis, &mut step_counter));

    // Phase 5: Verification
    phases.push(generate_verification_phase(
        &template_type,
        &mut step_counter,
    ));

    let source_path = String::new();
    let created_at = chrono::Utc::now().to_rfc3339();

    MigrationPlan {
        name: format!("{} migration plan", feature_name),
        project_type: analysis.project_type.clone(),
        source_path,
        phases,
        scores_before: analysis.scores.clone(),
        scores_after: None,
        created_at,
    }
}

fn next_step_id(counter: &mut u32) -> String {
    *counter += 1;
    format!("step-{:03}", counter)
}

fn generate_backup_phase(counter: &mut u32) -> MigrationPhase {
    MigrationPhase {
        number: 1,
        name: "Backup".to_string(),
        description: "既存プロジェクトのバックアップを作成します".to_string(),
        steps: vec![MigrationStep {
            id: next_step_id(counter),
            description: "プロジェクト全体のバックアップを作成".to_string(),
            action: MigrationAction::Backup {
                source: ".".to_string(),
                destination: "../backup".to_string(),
            },
            risk: StepRisk::Low,
            status: StepStatus::Pending,
            error: None,
        }],
    }
}

fn generate_structure_phase(analysis: &AnalysisResult, counter: &mut u32) -> MigrationPhase {
    let mut steps = Vec::new();

    // Create missing directories
    for dir in &analysis.structure.missing_dirs {
        steps.push(MigrationStep {
            id: next_step_id(counter),
            description: format!("ディレクトリ '{}' を作成", dir),
            action: MigrationAction::CreateDirectory {
                path: dir.clone(),
            },
            risk: StepRisk::Low,
            status: StepStatus::Pending,
            error: None,
        });
    }

    // Move directories for layer aliases that need renaming
    for missing_layer in &analysis.structure.missing_layers {
        let aliases = layer_aliases(missing_layer);
        for alias in aliases {
            // If the alias directory exists but the canonical name doesn't,
            // suggest a move
            if analysis
                .structure
                .existing_dirs
                .iter()
                .any(|d| d.ends_with(alias))
                && !analysis
                    .structure
                    .existing_dirs
                    .iter()
                    .any(|d| d.ends_with(missing_layer))
            {
                let from_dir = analysis
                    .structure
                    .existing_dirs
                    .iter()
                    .find(|d| d.ends_with(alias))
                    .cloned()
                    .unwrap_or_else(|| alias.to_string());

                let to_dir = from_dir.replace(alias, missing_layer);

                steps.push(MigrationStep {
                    id: next_step_id(counter),
                    description: format!(
                        "ディレクトリ '{}' を '{}' にリネーム",
                        from_dir, to_dir
                    ),
                    action: MigrationAction::MoveDirectory {
                        source: from_dir,
                        destination: to_dir,
                    },
                    risk: StepRisk::Medium,
                    status: StepStatus::Pending,
                    error: None,
                });
                break;
            }
        }
    }

    MigrationPhase {
        number: 2,
        name: "Structure".to_string(),
        description: "ディレクトリ構造を k1s0 の規約に合わせて整備します".to_string(),
        steps,
    }
}

fn generate_management_files_phase(
    feature_name: &str,
    template_type: &str,
    counter: &mut u32,
) -> MigrationPhase {
    let mut steps = Vec::new();

    // Generate manifest.json
    let manifest_content = format!(
        r#"{{
  "template": "{}",
  "version": "0.1.0",
  "service": {{
    "name": "{}",
    "language": "{}",
    "layer": "feature"
  }}
}}"#,
        template_type,
        feature_name,
        language_from_template(template_type),
    );

    steps.push(MigrationStep {
        id: next_step_id(counter),
        description: ".k1s0/manifest.json を生成".to_string(),
        action: MigrationAction::GenerateFile {
            path: ".k1s0/manifest.json".to_string(),
            content: manifest_content,
        },
        risk: StepRisk::Low,
        status: StepStatus::Pending,
        error: None,
    });

    // Generate config/default.yaml if missing
    steps.push(MigrationStep {
        id: next_step_id(counter),
        description: "config/default.yaml を生成".to_string(),
        action: MigrationAction::GenerateFile {
            path: "config/default.yaml".to_string(),
            content: format!(
                "# {} default configuration\nserver:\n  port: 8080\n  host: 0.0.0.0\n",
                feature_name
            ),
        },
        risk: StepRisk::Low,
        status: StepStatus::Pending,
        error: None,
    });

    // Generate environment-specific configs
    for env in &["dev", "stg", "prod"] {
        steps.push(MigrationStep {
            id: next_step_id(counter),
            description: format!("config/{}.yaml を生成", env),
            action: MigrationAction::GenerateFile {
                path: format!("config/{}.yaml", env),
                content: format!(
                    "# {} {} configuration\n# Override values from default.yaml here\n",
                    feature_name, env
                ),
            },
            risk: StepRisk::Low,
            status: StepStatus::Pending,
            error: None,
        });
    }

    MigrationPhase {
        number: 3,
        name: "ManagementFiles".to_string(),
        description: "k1s0 管理ファイル（manifest.json、config YAML）を生成します".to_string(),
        steps,
    }
}

fn generate_convention_fixes_phase(
    analysis: &AnalysisResult,
    counter: &mut u32,
) -> MigrationPhase {
    let mut steps = Vec::new();

    // Group violations by type
    for violation in &analysis.violations {
        match violation.rule_id.as_str() {
            "K020" => {
                // Environment variable references -> suggest config replacement
                if let Some(ref file) = violation.file {
                    steps.push(MigrationStep {
                        id: next_step_id(counter),
                        description: format!(
                            "{} の環境変数参照を config に置換 (行 {})",
                            file,
                            violation.line.unwrap_or(0)
                        ),
                        action: MigrationAction::ManualAction {
                            instruction: format!(
                                "{}:{} - {} を config/{{env}}.yaml の設定値に置換してください",
                                file,
                                violation.line.unwrap_or(0),
                                violation.message
                            ),
                        },
                        risk: StepRisk::High,
                        status: StepStatus::Pending,
                        error: None,
                    });
                }
            }
            "K021" => {
                // Hardcoded secrets -> suggest _file reference
                if let Some(ref file) = violation.file {
                    steps.push(MigrationStep {
                        id: next_step_id(counter),
                        description: format!(
                            "{} の機密情報直書きを _file 参照に変換",
                            file
                        ),
                        action: MigrationAction::ManualAction {
                            instruction: format!(
                                "{}:{} - {}。キーに _file サフィックスを付けてファイルパス参照に変更してください",
                                file,
                                violation.line.unwrap_or(0),
                                violation.message
                            ),
                        },
                        risk: StepRisk::High,
                        status: StepStatus::Pending,
                        error: None,
                    });
                }
            }
            "K022" => {
                // Dependency direction violation
                if let Some(ref file) = violation.file {
                    steps.push(MigrationStep {
                        id: next_step_id(counter),
                        description: format!("{} の依存方向違反を修正", file),
                        action: MigrationAction::ManualAction {
                            instruction: format!(
                                "{}:{} - {}。依存関係の方向を見直してください",
                                file,
                                violation.line.unwrap_or(0),
                                violation.message
                            ),
                        },
                        risk: StepRisk::High,
                        status: StepStatus::Pending,
                        error: None,
                    });
                }
            }
            "K029" => {
                // panic/unwrap/expect
                if let Some(ref file) = violation.file {
                    steps.push(MigrationStep {
                        id: next_step_id(counter),
                        description: format!(
                            "{} のパニック呼び出しを安全なエラーハンドリングに置換",
                            file
                        ),
                        action: MigrationAction::ManualAction {
                            instruction: format!(
                                "{}:{} - {}。Result/Option を適切にハンドリングしてください",
                                file,
                                violation.line.unwrap_or(0),
                                violation.message
                            ),
                        },
                        risk: StepRisk::Medium,
                        status: StepStatus::Pending,
                        error: None,
                    });
                }
            }
            "K060" => {
                // Dockerfile unpinned
                steps.push(MigrationStep {
                    id: next_step_id(counter),
                    description: "Dockerfile ベースイメージのバージョンを固定".to_string(),
                    action: MigrationAction::ManualAction {
                        instruction: format!(
                            "Dockerfile:{} - {}。具体的なバージョンタグを指定してください",
                            violation.line.unwrap_or(0),
                            violation.message
                        ),
                    },
                    risk: StepRisk::Low,
                    status: StepStatus::Pending,
                    error: None,
                });
            }
            _ => {
                // Other violations
                steps.push(MigrationStep {
                    id: next_step_id(counter),
                    description: format!(
                        "規約違反 {} を修正",
                        violation.rule_id
                    ),
                    action: MigrationAction::ManualAction {
                        instruction: format!(
                            "{}:{} - {}",
                            violation.file.as_deref().unwrap_or("(unknown)"),
                            violation.line.unwrap_or(0),
                            violation.message
                        ),
                    },
                    risk: StepRisk::Medium,
                    status: StepStatus::Pending,
                    error: None,
                });
            }
        }
    }

    // Delete .env files if found
    for env_file in &analysis.dependencies.env_files {
        steps.push(MigrationStep {
            id: next_step_id(counter),
            description: format!("{} を削除（config YAML に移行済み）", env_file),
            action: MigrationAction::DeleteFile {
                path: env_file.clone(),
            },
            risk: StepRisk::High,
            status: StepStatus::Pending,
            error: None,
        });
    }

    MigrationPhase {
        number: 4,
        name: "ConventionFixes".to_string(),
        description: "規約違反を修正します".to_string(),
        steps,
    }
}

fn generate_verification_phase(template_type: &str, counter: &mut u32) -> MigrationPhase {
    let mut steps = Vec::new();

    // Build command
    let (build_cmd, build_args) = match template_type {
        "backend-rust" => ("cargo", vec!["build".to_string()]),
        "backend-go" => ("go", vec!["build".to_string(), "./...".to_string()]),
        "backend-csharp" => ("dotnet", vec!["build".to_string()]),
        "backend-python" => ("uv", vec!["run".to_string(), "pytest".to_string()]),
        "frontend-react" => ("pnpm", vec!["build".to_string()]),
        "frontend-flutter" => ("dart", vec!["analyze".to_string()]),
        _ => ("echo", vec!["No build command configured".to_string()]),
    };

    steps.push(MigrationStep {
        id: next_step_id(counter),
        description: format!("ビルド確認: {} {}", build_cmd, build_args.join(" ")),
        action: MigrationAction::RunCommand {
            command: build_cmd.to_string(),
            args: build_args,
            working_dir: None,
        },
        risk: StepRisk::Low,
        status: StepStatus::Pending,
        error: None,
    });

    // k1s0 lint
    steps.push(MigrationStep {
        id: next_step_id(counter),
        description: "k1s0 lint を実行して規約準拠を確認".to_string(),
        action: MigrationAction::RunCommand {
            command: "k1s0".to_string(),
            args: vec!["lint".to_string()],
            working_dir: None,
        },
        risk: StepRisk::Low,
        status: StepStatus::Pending,
        error: None,
    });

    MigrationPhase {
        number: 5,
        name: "Verification".to_string(),
        description: "移行後のビルドと lint を検証します".to_string(),
        steps,
    }
}

fn language_from_template(template_type: &str) -> &str {
    match template_type {
        "backend-rust" => "rust",
        "backend-go" => "go",
        "backend-csharp" => "csharp",
        "backend-python" => "python",
        "frontend-react" => "typescript",
        "frontend-flutter" => "dart",
        _ => "unknown",
    }
}

fn layer_aliases(layer: &str) -> &[&str] {
    match layer {
        "domain" => &["entities", "model", "models", "core"],
        "application" => &["app", "usecase", "usecases", "use_cases", "services"],
        "infrastructure" => &[
            "infra",
            "adapter",
            "adapters",
            "persistence",
            "repository",
            "repositories",
            "external",
        ],
        "presentation" => &[
            "api",
            "handler",
            "handlers",
            "controller",
            "controllers",
            "rest",
            "grpc",
            "web",
            "ui",
        ],
        _ => &[],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer::types::{
        ComplianceScores, DependencyAnalysis, DetectedProjectType, StructureAnalysis, Violation,
        ViolationSeverity,
    };

    fn make_analysis() -> AnalysisResult {
        AnalysisResult {
            project_type: DetectedProjectType::BackendRust,
            structure: StructureAnalysis {
                existing_dirs: vec!["src".to_string()],
                missing_dirs: vec!["config".to_string(), "deploy".to_string()],
                existing_files: vec!["Cargo.toml".to_string()],
                missing_files: vec![".k1s0/manifest.json".to_string()],
                detected_layers: vec!["domain".to_string()],
                missing_layers: vec![
                    "application".to_string(),
                    "infrastructure".to_string(),
                    "presentation".to_string(),
                ],
            },
            violations: vec![
                Violation {
                    rule_id: "K020".to_string(),
                    severity: ViolationSeverity::Error,
                    message: "env::var( detected".to_string(),
                    file: Some("src/config.rs".to_string()),
                    line: Some(10),
                    auto_fixable: false,
                },
                Violation {
                    rule_id: "K060".to_string(),
                    severity: ViolationSeverity::Warning,
                    message: "Unpinned base image".to_string(),
                    file: Some("Dockerfile".to_string()),
                    line: Some(1),
                    auto_fixable: false,
                },
            ],
            dependencies: DependencyAnalysis {
                env_var_usages: vec![],
                hardcoded_secrets: vec![],
                external_dependencies: vec!["axum".to_string()],
                env_files: vec![".env".to_string()],
            },
            scores: ComplianceScores {
                structure: 30,
                convention: 85,
                dependency: 70,
                overall: 60,
            },
        }
    }

    #[test]
    fn test_generate_migration_plan() {
        let analysis = make_analysis();
        let plan = generate_migration_plan(&analysis, "my-service");

        assert_eq!(plan.phases.len(), 5);
        assert_eq!(plan.phases[0].name, "Backup");
        assert_eq!(plan.phases[1].name, "Structure");
        assert_eq!(plan.phases[2].name, "ManagementFiles");
        assert_eq!(plan.phases[3].name, "ConventionFixes");
        assert_eq!(plan.phases[4].name, "Verification");
    }

    #[test]
    fn test_plan_has_structure_steps_for_missing_dirs() {
        let analysis = make_analysis();
        let plan = generate_migration_plan(&analysis, "my-service");

        let structure_phase = &plan.phases[1];
        // Should have steps for "config" and "deploy" missing dirs
        let create_dir_steps: Vec<_> = structure_phase
            .steps
            .iter()
            .filter(|s| matches!(&s.action, MigrationAction::CreateDirectory { .. }))
            .collect();
        assert_eq!(create_dir_steps.len(), 2);
    }

    #[test]
    fn test_plan_has_convention_fix_steps() {
        let analysis = make_analysis();
        let plan = generate_migration_plan(&analysis, "my-service");

        let convention_phase = &plan.phases[3];
        // 2 violations + 1 .env file deletion = 3 steps
        assert_eq!(convention_phase.steps.len(), 3);
    }

    #[test]
    fn test_plan_total_steps() {
        let analysis = make_analysis();
        let plan = generate_migration_plan(&analysis, "my-service");
        assert!(plan.total_steps() > 0);
    }

    #[test]
    fn test_plan_verification_phase() {
        let analysis = make_analysis();
        let plan = generate_migration_plan(&analysis, "my-service");

        let verification_phase = &plan.phases[4];
        assert_eq!(verification_phase.steps.len(), 2);

        // First step should be cargo build
        if let MigrationAction::RunCommand { ref command, .. } = verification_phase.steps[0].action
        {
            assert_eq!(command, "cargo");
        } else {
            panic!("Expected RunCommand action");
        }
    }
}
