//! `k1s0 migrate` コマンド
//!
//! 既存プロジェクトを k1s0 構造に移行するための分析・計画・実行を支援する。

use std::path::{Path, PathBuf};
use std::process::Command;

use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};

use k1s0_generator::analyzer::{
    self, AnalysisResult, ComplianceScores, DetectedProjectType, MigrationAction, MigrationPlan,
    StepRisk, StepStatus,
};

use crate::error::{CliError, Result};
use crate::output::output;

/// `k1s0 migrate` の引数
#[derive(Args, Debug)]
#[command(after_long_help = r#"例:
  k1s0 migrate analyze --path ./my-project
  k1s0 migrate plan --path ./my-project --name my-service
  k1s0 migrate apply --path ./my-project
  k1s0 migrate status --path ./my-project

既存プロジェクトを k1s0 構造に移行するための分析・計画・実行を支援します。
"#)]
pub struct MigrateArgs {
    /// migrate サブコマンド
    #[command(subcommand)]
    pub action: MigrateAction,
}

/// Migrate サブコマンド
#[derive(Subcommand, Debug)]
pub enum MigrateAction {
    /// プロジェクトを分析する
    Analyze(AnalyzeArgs),
    /// 移行プランを生成する
    Plan(PlanArgs),
    /// 移行プランを適用する
    Apply(ApplyArgs),
    /// 移行の進捗状況を表示する
    Status(MigrateStatusArgs),
}

/// `k1s0 migrate analyze` の引数
#[derive(Args, Debug)]
pub struct AnalyzeArgs {
    /// 分析対象のプロジェクトパス
    #[arg(long, default_value = ".")]
    pub path: String,

    /// プロジェクトタイプ（省略時は自動検出）
    #[arg(long = "type")]
    pub project_type: Option<String>,

    /// JSON 形式で出力する
    #[arg(long)]
    pub json: bool,

    /// 詳細な出力を有効にする
    #[arg(long)]
    pub verbose: bool,
}

/// `k1s0 migrate plan` の引数
#[derive(Args, Debug)]
pub struct PlanArgs {
    /// 対象プロジェクトパス
    #[arg(long, default_value = ".")]
    pub path: String,

    /// プロジェクトタイプ（省略時は自動検出）
    #[arg(long = "type")]
    pub project_type: Option<String>,

    /// feature 名（省略時はディレクトリ名から推定）
    #[arg(long)]
    pub name: Option<String>,

    /// 出力ファイルパス
    #[arg(long, default_value = "migration-plan.json")]
    pub output: String,

    /// 自動実行可能なステップのみプランに含める
    #[arg(long)]
    pub auto_only: bool,

    /// ドライラン（ファイルに書き込まない）
    #[arg(long)]
    pub dry_run: bool,
}

/// `k1s0 migrate apply` の引数
#[derive(Args, Debug)]
pub struct ApplyArgs {
    /// 対象プロジェクトパス
    #[arg(long, default_value = ".")]
    pub path: String,

    /// 移行プランファイルのパス
    #[arg(long, default_value = "migration-plan.json")]
    pub plan: String,

    /// 特定のフェーズのみ適用（フェーズ番号）
    #[arg(long)]
    pub phase: Option<usize>,

    /// ドライラン（実際には変更しない）
    #[arg(long)]
    pub dry_run: bool,

    /// 確認をスキップする
    #[arg(short, long)]
    pub yes: bool,

    /// バックアップをスキップする
    #[arg(long)]
    pub skip_backup: bool,
}

/// `k1s0 migrate status` の引数
#[derive(Args, Debug)]
pub struct MigrateStatusArgs {
    /// 対象プロジェクトパス
    #[arg(long, default_value = ".")]
    pub path: String,

    /// 移行プランファイルのパス
    #[arg(long, default_value = "migration-plan.json")]
    pub plan: String,

    /// JSON 形式で出力する
    #[arg(long)]
    pub json: bool,
}

/// 分析結果の JSON 出力用
#[derive(Debug, Serialize, Deserialize)]
struct AnalysisOutput {
    project_type: String,
    scores: ComplianceScores,
    structure: StructureSummary,
    violations: Vec<ViolationOutput>,
    dependencies: DependencySummary,
}

#[derive(Debug, Serialize, Deserialize)]
struct StructureSummary {
    existing_dirs: usize,
    missing_dirs: usize,
    existing_files: usize,
    missing_files: usize,
    detected_layers: Vec<String>,
    missing_layers: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ViolationOutput {
    rule_id: String,
    severity: String,
    message: String,
    file: Option<String>,
    line: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DependencySummary {
    env_var_count: usize,
    secret_count: usize,
    env_file_count: usize,
    external_dep_count: usize,
}

/// `k1s0 migrate` を実行する
pub fn execute(args: MigrateArgs) -> Result<()> {
    match args.action {
        MigrateAction::Analyze(a) => execute_analyze(a),
        MigrateAction::Plan(a) => execute_plan(a),
        MigrateAction::Apply(a) => execute_apply(a),
        MigrateAction::Status(a) => execute_status(a),
    }
}

/// プロジェクト分析の内部実装
fn run_analysis(path: &Path, explicit_type: Option<&str>) -> Result<AnalysisResult> {
    let project_type = if let Some(t) = explicit_type {
        parse_project_type(t)?
    } else {
        analyzer::detect_project_type(path)
    };

    if project_type == DetectedProjectType::Unknown {
        return Err(
            CliError::validation("プロジェクトタイプを検出できません")
                .with_hint("--type オプションでプロジェクトタイプを指定してください")
                .with_path(path),
        );
    }

    let structure = analyzer::analyze_structure(path, &project_type);
    let violations = analyzer::scan_violations(path, &project_type);
    let dependencies = analyzer::analyze_dependencies(path, &project_type);
    let scores = analyzer::calculate_scores(&structure, &violations, &dependencies);

    Ok(AnalysisResult {
        project_type,
        structure,
        violations,
        dependencies,
        scores,
    })
}

/// `k1s0 migrate analyze` を実行する
fn execute_analyze(args: AnalyzeArgs) -> Result<()> {
    let out = output();
    let path = resolve_path(&args.path)?;

    out.info(&format!("プロジェクトを分析しています: {}", path.display()));

    let result = run_analysis(&path, args.project_type.as_deref())?;

    if args.json {
        let json_output = build_analysis_output(&result);
        out.print_json(&json_output);
        return Ok(());
    }

    // ヘッダー
    out.newline();
    out.header("分析結果:");
    out.newline();

    // プロジェクトタイプ
    out.list_item("プロジェクトタイプ", &result.project_type.to_string());
    out.newline();

    // スコア
    out.header("コンプライアンススコア:");
    display_scores(&result.scores);
    out.newline();

    // 構造
    out.header("ディレクトリ構造:");
    out.list_item(
        "  既存ディレクトリ",
        &result.structure.existing_dirs.len().to_string(),
    );
    out.list_item(
        "  不足ディレクトリ",
        &result.structure.missing_dirs.len().to_string(),
    );
    out.list_item(
        "  既存ファイル",
        &result.structure.existing_files.len().to_string(),
    );
    out.list_item(
        "  不足ファイル",
        &result.structure.missing_files.len().to_string(),
    );

    if !result.structure.detected_layers.is_empty() {
        out.list_item(
            "  検出された層",
            &result.structure.detected_layers.join(", "),
        );
    }
    if !result.structure.missing_layers.is_empty() {
        out.list_item(
            "  不足している層",
            &result.structure.missing_layers.join(", "),
        );
    }
    out.newline();

    // 規約違反
    if result.violations.is_empty() {
        out.success("規約違反はありません");
    } else {
        out.header(&format!("規約違反 ({} 件):", result.violations.len()));
        let display_count = if args.verbose {
            result.violations.len()
        } else {
            result.violations.len().min(20)
        };

        for violation in result.violations.iter().take(display_count) {
            let severity_label = match violation.severity {
                k1s0_generator::analyzer::ViolationSeverity::Error => "ERROR",
                k1s0_generator::analyzer::ViolationSeverity::Warning => "WARN ",
            };
            let location = match (&violation.file, violation.line) {
                (Some(f), Some(l)) => format!(" ({f}:{l})"),
                (Some(f), None) => format!(" ({f})"),
                _ => String::new(),
            };
            out.list_item(
                &format!("  [{severity_label}] {}", violation.rule_id),
                &format!("{}{location}", violation.message),
            );
        }

        if !args.verbose && result.violations.len() > 20 {
            out.hint(&format!(
                "他 {} 件の違反があります。--verbose で全て表示します。",
                result.violations.len() - 20
            ));
        }
    }
    out.newline();

    // 依存関係
    out.header("依存関係:");
    out.list_item(
        "  環境変数使用",
        &format!("{} 箇所", result.dependencies.env_var_usages.len()),
    );
    out.list_item(
        "  ハードコードシークレット",
        &format!("{} 箇所", result.dependencies.hardcoded_secrets.len()),
    );
    out.list_item(
        "  .env ファイル",
        &format!("{} 件", result.dependencies.env_files.len()),
    );
    out.list_item(
        "  外部依存",
        &format!("{} 件", result.dependencies.external_dependencies.len()),
    );
    out.newline();

    if args.verbose && !result.dependencies.env_var_usages.is_empty() {
        out.header("環境変数使用箇所:");
        for usage in &result.dependencies.env_var_usages {
            let var = usage.var_name.as_deref().unwrap_or("?");
            out.list_item(
                &format!("  {}:{}", usage.file, usage.line),
                &format!("{var} ({})  ", usage.pattern),
            );
        }
        out.newline();
    }

    // 次のステップ
    out.hint("次のステップ: k1s0 migrate plan でプランを生成してください");

    Ok(())
}

/// `k1s0 migrate plan` を実行する
fn execute_plan(args: PlanArgs) -> Result<()> {
    let out = output();
    let path = resolve_path(&args.path)?;

    out.info(&format!("プロジェクトを分析しています: {}", path.display()));

    let result = run_analysis(&path, args.project_type.as_deref())?;

    // feature 名の決定
    let feature_name = args.name.unwrap_or_else(|| {
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("my-service")
            .to_string()
            .replace('_', "-")
            .to_lowercase()
    });

    out.info(&format!("feature 名: {feature_name}"));

    // プラン生成
    let mut plan =
        analyzer::generate_migration_plan(&result, &feature_name);

    // --auto-only: 手動ステップを除去
    if args.auto_only {
        for phase in &mut plan.phases {
            phase.steps.retain(|s| {
                !matches!(s.action, MigrationAction::ManualAction { .. })
            });
        }
        plan.phases.retain(|p| !p.steps.is_empty());
        // フェーズ番号を振り直し
        for (i, phase) in plan.phases.iter_mut().enumerate() {
            phase.number = i + 1;
        }
    }

    // サマリー表示
    out.newline();
    out.header("移行プラン:");
    out.list_item("feature 名", &plan.name);
    out.list_item("プロジェクトタイプ", &plan.project_type.to_string());
    out.list_item("フェーズ数", &plan.phases.len().to_string());
    out.list_item("総ステップ数", &plan.total_steps().to_string());
    out.newline();

    for phase in &plan.phases {
        out.header(&format!(
            "Phase {}: {} ({} ステップ)",
            phase.number,
            phase.name,
            phase.steps.len()
        ));
        out.hint(&phase.description);

        for step in &phase.steps {
            let risk_label = match step.risk {
                StepRisk::Low => "LOW ",
                StepRisk::Medium => "MED ",
                StepRisk::High => "HIGH",
            };
            let action_type = action_type_label(&step.action);
            out.list_item(
                &format!("  [{risk_label}] {action_type}"),
                &step.description,
            );
        }
        out.newline();
    }

    // スコア比較
    out.header("スコア予測:");
    out.list_item(
        "  現在",
        &format!(
            "構造: {} / 規約: {} / 依存: {} / 総合: {}",
            plan.scores_before.structure,
            plan.scores_before.convention,
            plan.scores_before.dependency,
            plan.scores_before.overall
        ),
    );
    if let Some(ref after) = plan.scores_after {
        out.list_item(
            "  予測",
            &format!(
                "構造: {} / 規約: {} / 依存: {} / 総合: {}",
                after.structure, after.convention, after.dependency, after.overall
            ),
        );
    }
    out.newline();

    // ファイル出力
    if args.dry_run {
        out.info(&format!(
            "ドライラン: {} には書き込みません",
            args.output
        ));
    } else {
        let json = serde_json::to_string_pretty(&plan)
            .map_err(|e| CliError::io(format!("JSON シリアライズに失敗: {e}")))?;
        let output_path = path.join(&args.output);
        std::fs::write(&output_path, &json)
            .map_err(|e| CliError::io(format!("プランファイルの書き込みに失敗: {e}")))?;
        out.success(&format!("プランを保存しました: {}", output_path.display()));
    }

    out.hint("次のステップ: k1s0 migrate apply でプランを適用してください");

    Ok(())
}

/// `k1s0 migrate apply` を実行する
fn execute_apply(args: ApplyArgs) -> Result<()> {
    let out = output();
    let path = resolve_path(&args.path)?;
    let plan_path = path.join(&args.plan);

    // プランの読み込み
    let plan_content = std::fs::read_to_string(&plan_path).map_err(|e| {
        CliError::io(format!(
            "プランファイルの読み取りに失敗: {e}"
        ))
        .with_path(&plan_path)
        .with_hint("先に k1s0 migrate plan を実行してプランを生成してください")
        .with_recovery(
            format!("k1s0 migrate plan --path {}", args.path),
            "プランを生成",
        )
    })?;

    let mut plan: MigrationPlan = serde_json::from_str(&plan_content).map_err(|e| {
        CliError::config(format!("プランファイルのパースに失敗: {e}")).with_path(&plan_path)
    })?;

    // 完了済みステップ数
    let already_completed = plan.completed_steps();
    let total = plan.total_steps();
    if already_completed > 0 {
        out.info(&format!(
            "進捗: {already_completed}/{total} ステップ完了済み"
        ));
    }

    // 対象フェーズの決定
    let target_phases: Vec<usize> = if let Some(phase_num) = args.phase {
        vec![phase_num]
    } else {
        plan.phases.iter().map(|p| p.number).collect()
    };

    // ドライラン表示
    if args.dry_run {
        out.header("ドライラン: 以下のステップが実行されます");
        out.newline();

        for phase in &plan.phases {
            if !target_phases.contains(&phase.number) {
                continue;
            }
            out.header(&format!("Phase {}: {}", phase.number, phase.name));
            for step in &phase.steps {
                if step.status == StepStatus::Completed || step.status == StepStatus::Skipped {
                    continue;
                }
                let action_desc = describe_action(&step.action);
                out.list_item(&format!("  [{}]", step.id), &action_desc);
            }
            out.newline();
        }
        return Ok(());
    }

    // 確認プロンプト
    let pending_count = plan
        .phases
        .iter()
        .filter(|p| target_phases.contains(&p.number))
        .flat_map(|p| &p.steps)
        .filter(|s| s.status == StepStatus::Pending)
        .count();

    if pending_count == 0 {
        out.info("実行するステップがありません");
        return Ok(());
    }

    if !args.yes {
        let msg = format!("{pending_count} ステップを実行しますか?");
        if !out.confirm_proceed(&msg) {
            return Err(CliError::cancelled("操作がキャンセルされました"));
        }
    }

    // バックアップ
    if !args.skip_backup {
        out.info("バックアップを作成しています...");
        create_backup(&path)?;
        out.success("バックアップを作成しました");
    } else {
        out.warning("バックアップをスキップしました。データ損失のリスクがあります。");
    }

    // ステップの実行
    let mut completed = 0u32;
    let mut failed = 0u32;
    let mut skipped = 0u32;

    for phase_idx in 0..plan.phases.len() {
        if !target_phases.contains(&plan.phases[phase_idx].number) {
            continue;
        }

        out.newline();
        out.header(&format!(
            "Phase {}: {}",
            plan.phases[phase_idx].number, plan.phases[phase_idx].name
        ));

        for step_idx in 0..plan.phases[phase_idx].steps.len() {
            let step = &plan.phases[phase_idx].steps[step_idx];
            if step.status == StepStatus::Completed {
                out.info(&format!("[{}] スキップ (完了済み)", step.id));
                continue;
            }
            if step.status == StepStatus::Skipped {
                continue;
            }

            out.info(&format!("[{}] {}", step.id, step.description));

            let result = execute_step(&step.action, &path);

            let step = &mut plan.phases[phase_idx].steps[step_idx];
            match result {
                Ok(()) => {
                    step.status = StepStatus::Completed;
                    step.error = None;
                    completed += 1;
                    out.success(&format!("[{}] 完了", step.id));
                }
                Err(StepError::Skipped(reason)) => {
                    step.status = StepStatus::Skipped;
                    step.error = Some(reason.clone());
                    skipped += 1;
                    out.warning(&format!("[{}] スキップ: {reason}", step.id));
                }
                Err(StepError::Failed(reason)) => {
                    step.status = StepStatus::Failed;
                    step.error = Some(reason.clone());
                    failed += 1;
                    out.warning(&format!("[{}] 失敗: {reason}", step.id));
                }
            }
        }

        // フェーズ完了後にプランを保存
        save_plan(&plan, &plan_path)?;
    }

    // 結果サマリー
    out.newline();
    out.header("適用結果:");
    out.list_item("完了", &completed.to_string());
    out.list_item("スキップ", &skipped.to_string());
    out.list_item("失敗", &failed.to_string());
    out.newline();

    if failed > 0 {
        out.warning("一部のステップが失敗しました。k1s0 migrate status で詳細を確認してください。");
    } else {
        out.success("移行ステップの適用が完了しました");
    }

    out.hint("次のステップ: k1s0 migrate status で進捗を確認してください");
    out.hint("最終確認: k1s0 lint で規約チェックを実行してください");

    Ok(())
}

/// `k1s0 migrate status` を実行する
fn execute_status(args: MigrateStatusArgs) -> Result<()> {
    let out = output();
    let path = resolve_path(&args.path)?;
    let plan_path = path.join(&args.plan);

    let plan_content = std::fs::read_to_string(&plan_path).map_err(|e| {
        CliError::io(format!("プランファイルの読み取りに失敗: {e}"))
            .with_path(&plan_path)
            .with_hint("先に k1s0 migrate plan を実行してプランを生成してください")
    })?;

    let plan: MigrationPlan = serde_json::from_str(&plan_content).map_err(|e| {
        CliError::config(format!("プランファイルのパースに失敗: {e}")).with_path(&plan_path)
    })?;

    if args.json {
        out.print_json(&plan);
        return Ok(());
    }

    let total = plan.total_steps();
    let completed = plan.completed_steps();
    let skipped_count = plan.skipped_steps();
    let pending = total - completed - skipped_count;
    let failed_count = plan
        .phases
        .iter()
        .flat_map(|p| &p.steps)
        .filter(|s| s.status == StepStatus::Failed)
        .count();

    // 全体進捗
    out.header("移行進捗:");
    out.list_item("feature 名", &plan.name);
    out.list_item("プロジェクトタイプ", &plan.project_type.to_string());
    out.newline();

    let progress_pct = if total > 0 {
        (completed * 100) / total
    } else {
        0
    };
    out.list_item("進捗", &format!("{completed}/{total} ({progress_pct}%)"));
    out.list_item("完了", &completed.to_string());
    out.list_item("保留", &pending.to_string());
    out.list_item("スキップ", &skipped_count.to_string());
    out.list_item("失敗", &failed_count.to_string());
    out.newline();

    // フェーズ別進捗
    out.header("フェーズ別進捗:");
    for phase in &plan.phases {
        let phase_total = phase.steps.len();
        let phase_completed = phase
            .steps
            .iter()
            .filter(|s| s.status == StepStatus::Completed)
            .count();
        let phase_pct = if phase_total > 0 {
            (phase_completed * 100) / phase_total
        } else {
            0
        };

        let status_icon = if phase_completed == phase_total {
            "DONE"
        } else if phase_completed > 0 {
            "WIP "
        } else {
            "TODO"
        };

        out.list_item(
            &format!("  [{status_icon}] Phase {}", phase.number),
            &format!(
                "{} ({phase_completed}/{phase_total}, {phase_pct}%)",
                phase.name
            ),
        );
    }
    out.newline();

    // 手動アクションの残件
    let manual_pending: Vec<_> = plan
        .phases
        .iter()
        .flat_map(|p| &p.steps)
        .filter(|s| {
            s.status == StepStatus::Pending
                && matches!(s.action, MigrationAction::ManualAction { .. })
        })
        .collect();

    if !manual_pending.is_empty() {
        out.header(&format!(
            "手動対応が必要な項目 ({} 件):",
            manual_pending.len()
        ));
        for step in &manual_pending {
            if let MigrationAction::ManualAction { ref instruction } = step.action {
                out.list_item(&format!("  [{}]", step.id), &step.description);
                out.hint(&format!("    {instruction}"));
            }
        }
        out.newline();
    }

    // 失敗ステップ
    let failed_steps: Vec<_> = plan
        .phases
        .iter()
        .flat_map(|p| &p.steps)
        .filter(|s| s.status == StepStatus::Failed)
        .collect();

    if !failed_steps.is_empty() {
        out.header(&format!("失敗したステップ ({} 件):", failed_steps.len()));
        for step in &failed_steps {
            out.list_item(
                &format!("  [{}]", step.id),
                &format!(
                    "{}: {}",
                    step.description,
                    step.error.as_deref().unwrap_or("不明")
                ),
            );
        }
        out.newline();
    }

    // スコア比較
    out.header("スコア比較:");
    out.list_item(
        "  移行前",
        &format!(
            "構造: {} / 規約: {} / 依存: {} / 総合: {}",
            plan.scores_before.structure,
            plan.scores_before.convention,
            plan.scores_before.dependency,
            plan.scores_before.overall
        ),
    );
    if let Some(ref after) = plan.scores_after {
        out.list_item(
            "  予測後",
            &format!(
                "構造: {} / 規約: {} / 依存: {} / 総合: {}",
                after.structure, after.convention, after.dependency, after.overall
            ),
        );
    }
    out.newline();

    if pending > 0 {
        out.hint("次のステップ: k1s0 migrate apply で残りのステップを適用してください");
    } else if failed_count > 0 {
        out.hint("失敗したステップを確認し、手動で修正してから再度 k1s0 migrate apply を実行してください");
    } else {
        out.success("全てのステップが完了しました");
        out.hint("最終確認: k1s0 lint で規約チェックを実行してください");
    }

    Ok(())
}

// --- ヘルパー関数 ---

/// パスを解決し、存在確認する
fn resolve_path(path_str: &str) -> Result<PathBuf> {
    let path = PathBuf::from(path_str)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(path_str));

    if !path.is_dir() {
        return Err(CliError::validation(format!(
            "指定されたパスが見つからないか、ディレクトリではありません: {}",
            path.display()
        )));
    }

    Ok(path)
}

/// プロジェクトタイプ文字列をパースする
fn parse_project_type(s: &str) -> Result<DetectedProjectType> {
    match s {
        "backend-rust" => Ok(DetectedProjectType::BackendRust),
        "backend-go" => Ok(DetectedProjectType::BackendGo),
        "backend-csharp" => Ok(DetectedProjectType::BackendCsharp),
        "backend-python" => Ok(DetectedProjectType::BackendPython),
        "frontend-react" => Ok(DetectedProjectType::FrontendReact),
        "frontend-flutter" => Ok(DetectedProjectType::FrontendFlutter),
        _ => Err(CliError::usage(format!(
            "無効なプロジェクトタイプ: {s}"
        ))
        .with_hint(
            "利用可能なタイプ: backend-rust, backend-go, backend-csharp, backend-python, frontend-react, frontend-flutter",
        )),
    }
}

/// スコアを表示する
fn display_scores(scores: &ComplianceScores) {
    let out = output();
    out.list_item("  構造", &format!("{}/100", scores.structure));
    out.list_item("  規約", &format!("{}/100", scores.convention));
    out.list_item("  依存関係", &format!("{}/100", scores.dependency));
    out.list_item("  総合", &format!("{}/100", scores.overall));
}

/// アクションタイプのラベル
fn action_type_label(action: &MigrationAction) -> &'static str {
    match action {
        MigrationAction::Backup { .. } => "BACKUP",
        MigrationAction::CreateDirectory { .. } => "MKDIR ",
        MigrationAction::MoveDirectory { .. } => "MVDIR ",
        MigrationAction::MoveFile { .. } => "MVFILE",
        MigrationAction::GenerateFile { .. } => "CREATE",
        MigrationAction::ReplaceInFile { .. } => "EDIT  ",
        MigrationAction::DeleteFile { .. } => "DELETE",
        MigrationAction::RunCommand { .. } => "CMD   ",
        MigrationAction::ManualAction { .. } => "MANUAL",
    }
}

/// アクションの説明文を生成する
fn describe_action(action: &MigrationAction) -> String {
    match action {
        MigrationAction::Backup {
            source,
            destination,
        } => format!("バックアップ: {source} -> {destination}"),
        MigrationAction::CreateDirectory { path } => format!("ディレクトリ作成: {path}"),
        MigrationAction::MoveDirectory {
            source,
            destination,
        } => format!("ディレクトリ移動: {source} -> {destination}"),
        MigrationAction::MoveFile {
            source,
            destination,
        } => format!("ファイル移動: {source} -> {destination}"),
        MigrationAction::GenerateFile { path, .. } => format!("ファイル生成: {path}"),
        MigrationAction::ReplaceInFile { path, search, .. } => {
            format!("文字列置換: {path} ({search})")
        }
        MigrationAction::DeleteFile { path } => format!("ファイル削除: {path}"),
        MigrationAction::RunCommand { command, args, .. } => {
            format!("コマンド実行: {command} {}", args.join(" "))
        }
        MigrationAction::ManualAction { instruction } => {
            let truncated = if instruction.len() > 80 {
                format!("{}...", &instruction[..77])
            } else {
                instruction.clone()
            };
            format!("手動: {truncated}")
        }
    }
}

/// JSON 出力用の分析結果を構築する
fn build_analysis_output(result: &AnalysisResult) -> AnalysisOutput {
    AnalysisOutput {
        project_type: result.project_type.to_string(),
        scores: result.scores.clone(),
        structure: StructureSummary {
            existing_dirs: result.structure.existing_dirs.len(),
            missing_dirs: result.structure.missing_dirs.len(),
            existing_files: result.structure.existing_files.len(),
            missing_files: result.structure.missing_files.len(),
            detected_layers: result.structure.detected_layers.clone(),
            missing_layers: result.structure.missing_layers.clone(),
        },
        violations: result
            .violations
            .iter()
            .map(|v| ViolationOutput {
                rule_id: v.rule_id.clone(),
                severity: v.severity.to_string(),
                message: v.message.clone(),
                file: v.file.clone(),
                line: v.line,
            })
            .collect(),
        dependencies: DependencySummary {
            env_var_count: result.dependencies.env_var_usages.len(),
            secret_count: result.dependencies.hardcoded_secrets.len(),
            env_file_count: result.dependencies.env_files.len(),
            external_dep_count: result.dependencies.external_dependencies.len(),
        },
    }
}

/// ステップ実行エラー
enum StepError {
    /// スキップされた（手動アクション等）
    Skipped(String),
    /// 失敗
    Failed(String),
}

/// 個別ステップを実行する
fn execute_step(action: &MigrationAction, base_path: &Path) -> std::result::Result<(), StepError> {
    match action {
        MigrationAction::Backup {
            source,
            destination,
        } => {
            let src = base_path.join(source);
            let dst = base_path.join(destination);
            copy_dir_recursive(&src, &dst)
                .map_err(|e| StepError::Failed(format!("バックアップに失敗: {e}")))?;
            Ok(())
        }
        MigrationAction::CreateDirectory { path } => {
            let dir_path = base_path.join(path);
            std::fs::create_dir_all(&dir_path)
                .map_err(|e| StepError::Failed(format!("ディレクトリ作成に失敗: {e}")))?;
            Ok(())
        }
        MigrationAction::MoveDirectory {
            source,
            destination,
        } => {
            let src = base_path.join(source);
            let dst = base_path.join(destination);

            if !src.exists() {
                return Err(StepError::Skipped(format!(
                    "移動元が存在しません: {source}"
                )));
            }

            // git mv を試行し、失敗した場合は fs::rename にフォールバック
            if let Some(dst_parent) = dst.parent() {
                let _ = std::fs::create_dir_all(dst_parent);
            }

            let git_result = Command::new("git")
                .args(["mv", source, destination])
                .current_dir(base_path)
                .status();

            match git_result {
                Ok(status) if status.success() => Ok(()),
                _ => {
                    std::fs::rename(&src, &dst).map_err(|e| {
                        StepError::Failed(format!("ディレクトリ移動に失敗: {e}"))
                    })?;
                    Ok(())
                }
            }
        }
        MigrationAction::MoveFile {
            source,
            destination,
        } => {
            let src = base_path.join(source);
            let dst = base_path.join(destination);

            if !src.exists() {
                return Err(StepError::Skipped(format!(
                    "移動元が存在しません: {source}"
                )));
            }

            if let Some(dst_parent) = dst.parent() {
                let _ = std::fs::create_dir_all(dst_parent);
            }

            let git_result = Command::new("git")
                .args(["mv", source, destination])
                .current_dir(base_path)
                .status();

            match git_result {
                Ok(status) if status.success() => Ok(()),
                _ => {
                    std::fs::rename(&src, &dst).map_err(|e| {
                        StepError::Failed(format!("ファイル移動に失敗: {e}"))
                    })?;
                    Ok(())
                }
            }
        }
        MigrationAction::GenerateFile { path, content } => {
            let file_path = base_path.join(path);

            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| StepError::Failed(format!("ディレクトリ作成に失敗: {e}")))?;
            }

            std::fs::write(&file_path, content)
                .map_err(|e| StepError::Failed(format!("ファイル生成に失敗: {e}")))?;
            Ok(())
        }
        MigrationAction::ReplaceInFile {
            path,
            search,
            replace,
        } => {
            let file_path = base_path.join(path);

            if !file_path.exists() {
                return Err(StepError::Skipped(format!(
                    "ファイルが存在しません: {path}"
                )));
            }

            let content = std::fs::read_to_string(&file_path)
                .map_err(|e| StepError::Failed(format!("ファイル読み取りに失敗: {e}")))?;

            let new_content = content.replace(search, replace);

            if content == new_content {
                return Err(StepError::Skipped(format!(
                    "置換対象が見つかりません: {search}"
                )));
            }

            std::fs::write(&file_path, &new_content)
                .map_err(|e| StepError::Failed(format!("ファイル書き込みに失敗: {e}")))?;
            Ok(())
        }
        MigrationAction::DeleteFile { path } => {
            let file_path = base_path.join(path);

            if !file_path.exists() {
                return Err(StepError::Skipped(format!(
                    "ファイルが存在しません: {path}"
                )));
            }

            std::fs::remove_file(&file_path)
                .map_err(|e| StepError::Failed(format!("ファイル削除に失敗: {e}")))?;
            Ok(())
        }
        MigrationAction::RunCommand {
            command,
            args,
            working_dir,
        } => {
            let work_dir = working_dir
                .as_ref()
                .map(|d| base_path.join(d))
                .unwrap_or_else(|| base_path.to_path_buf());

            let status = Command::new(command)
                .args(args)
                .current_dir(&work_dir)
                .status()
                .map_err(|e| StepError::Failed(format!("コマンド実行に失敗: {e}")))?;

            if status.success() {
                Ok(())
            } else {
                Err(StepError::Failed(format!(
                    "コマンドが終了コード {} で失敗しました",
                    status.code().unwrap_or(-1)
                )))
            }
        }
        MigrationAction::ManualAction { instruction } => {
            let out = output();
            out.warning(&format!("手動対応が必要です: {instruction}"));
            Err(StepError::Skipped("手動アクション".into()))
        }
    }
}

/// ディレクトリを再帰的にコピーする
fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !src.is_dir() {
        return Ok(());
    }

    std::fs::create_dir_all(dst)?;

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

/// バックアップを作成する
fn create_backup(path: &Path) -> Result<()> {
    let backup_name = format!(
        ".k1s0-backup-{}",
        chrono::Local::now().format("%Y%m%d-%H%M%S")
    );
    let backup_dir = path.join(&backup_name);

    // ソースファイルのみバックアップ（target, node_modules 等を除外）
    std::fs::create_dir_all(&backup_dir)
        .map_err(|e| CliError::io(format!("バックアップディレクトリの作成に失敗: {e}")))?;

    let skip_dirs = [
        "target",
        "node_modules",
        ".git",
        "vendor",
        "bin",
        "obj",
        "__pycache__",
        ".k1s0-backup",
    ];

    copy_dir_filtered(path, &backup_dir, &skip_dirs)
        .map_err(|e| CliError::io(format!("バックアップの作成に失敗: {e}")))?;

    let out = output();
    out.info(&format!("バックアップ先: {}", backup_dir.display()));

    Ok(())
}

/// スキップ対象を除外してディレクトリをコピーする
fn copy_dir_filtered(src: &Path, dst: &Path, skip_prefixes: &[&str]) -> std::io::Result<()> {
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        if skip_prefixes.iter().any(|p| name_str.starts_with(p)) {
            continue;
        }

        let src_path = entry.path();
        let dst_path = dst.join(&name);

        if src_path.is_dir() {
            std::fs::create_dir_all(&dst_path)?;
            copy_dir_filtered(&src_path, &dst_path, skip_prefixes)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

/// プランファイルを保存する
fn save_plan(plan: &MigrationPlan, path: &Path) -> Result<()> {
    let json = serde_json::to_string_pretty(plan)
        .map_err(|e| CliError::io(format!("JSON シリアライズに失敗: {e}")))?;
    std::fs::write(path, &json)
        .map_err(|e| CliError::io(format!("プランファイルの保存に失敗: {e}")))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser)]
    struct TestCli {
        #[command(subcommand)]
        command: MigrateAction,
    }

    #[test]
    fn test_parse_analyze() {
        let cli = TestCli::parse_from(["test", "analyze", "--path", "/tmp/project"]);
        assert!(matches!(cli.command, MigrateAction::Analyze(_)));
    }

    #[test]
    fn test_parse_plan() {
        let cli = TestCli::parse_from([
            "test",
            "plan",
            "--path",
            "/tmp/project",
            "--name",
            "my-service",
        ]);
        match cli.command {
            MigrateAction::Plan(args) => {
                assert_eq!(args.name, Some("my-service".to_string()));
            }
            _ => panic!("Expected Plan"),
        }
    }

    #[test]
    fn test_parse_apply_dry_run() {
        let cli = TestCli::parse_from(["test", "apply", "--dry-run", "--yes"]);
        match cli.command {
            MigrateAction::Apply(args) => {
                assert!(args.dry_run);
                assert!(args.yes);
            }
            _ => panic!("Expected Apply"),
        }
    }

    #[test]
    fn test_parse_status_json() {
        let cli = TestCli::parse_from(["test", "status", "--json"]);
        match cli.command {
            MigrateAction::Status(args) => {
                assert!(args.json);
            }
            _ => panic!("Expected Status"),
        }
    }

    #[test]
    fn test_parse_project_type_valid() {
        assert!(parse_project_type("backend-rust").is_ok());
        assert!(parse_project_type("backend-go").is_ok());
        assert!(parse_project_type("frontend-react").is_ok());
    }

    #[test]
    fn test_parse_project_type_invalid() {
        assert!(parse_project_type("invalid").is_err());
    }

    #[test]
    fn test_action_type_label() {
        assert_eq!(
            action_type_label(&MigrationAction::CreateDirectory {
                path: "test".into()
            }),
            "MKDIR "
        );
        assert_eq!(
            action_type_label(&MigrationAction::ManualAction {
                instruction: "test".into()
            }),
            "MANUAL"
        );
    }

    #[test]
    fn test_describe_action() {
        let desc = describe_action(&MigrationAction::CreateDirectory {
            path: "src/domain".into(),
        });
        assert!(desc.contains("src/domain"));
    }
}
