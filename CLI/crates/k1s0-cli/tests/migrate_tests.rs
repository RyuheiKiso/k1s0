//! `k1s0 migrate` コマンドの統合テスト
//!
//! テストフィクスチャに対してアナライザー関数を直接呼び出し、
//! プロジェクト検出・スコア計算・プラン生成の正確性を検証する。

use std::path::{Path, PathBuf};

use k1s0_generator::analyzer::{
    self, AnalysisResult, DetectedProjectType,
};

// ---------------------------------------------------------------------------
// ヘルパー
// ---------------------------------------------------------------------------

/// テストフィクスチャのベースパスを返す
fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("migrate")
}

/// 指定フィクスチャに対してフル分析を実行する
fn run_analysis(fixture_name: &str) -> AnalysisResult {
    let path = fixtures_dir().join(fixture_name);
    assert!(path.is_dir(), "fixture not found: {}", path.display());

    let project_type = analyzer::detect_project_type(&path);
    assert_ne!(
        project_type,
        DetectedProjectType::Unknown,
        "project type should be detected for fixture: {fixture_name}"
    );

    let structure = analyzer::analyze_structure(&path, &project_type);
    let violations = analyzer::scan_violations(&path, &project_type);
    let dependencies = analyzer::analyze_dependencies(&path, &project_type);
    let scores = analyzer::calculate_scores(&structure, &violations, &dependencies);

    AnalysisResult {
        project_type,
        structure,
        violations,
        dependencies,
        scores,
    }
}

// ---------------------------------------------------------------------------
// 1. rust-axum-basic: backend-rust として検出、スコア < 60
// ---------------------------------------------------------------------------

#[test]
fn analyze_rust_axum_basic_detects_backend_rust() {
    let result = run_analysis("rust-axum-basic");
    assert_eq!(result.project_type, DetectedProjectType::BackendRust);
}

#[test]
fn analyze_rust_axum_basic_score_below_60() {
    let result = run_analysis("rust-axum-basic");
    assert!(
        result.scores.overall < 60,
        "rust-axum-basic overall score should be < 60, got {}",
        result.scores.overall
    );
}

// ---------------------------------------------------------------------------
// 2. rust-clean-arch: rust-axum-basic より高いスコア
// ---------------------------------------------------------------------------

#[test]
fn analyze_rust_clean_arch_has_higher_score_than_basic() {
    let basic = run_analysis("rust-axum-basic");
    let clean = run_analysis("rust-clean-arch");

    assert_eq!(clean.project_type, DetectedProjectType::BackendRust);
    assert!(
        clean.scores.overall > basic.scores.overall,
        "rust-clean-arch overall ({}) should be > rust-axum-basic overall ({})",
        clean.scores.overall,
        basic.scores.overall
    );
}

// ---------------------------------------------------------------------------
// 3. go-gin-basic: backend-go として検出
// ---------------------------------------------------------------------------

#[test]
fn analyze_go_gin_basic_detects_backend_go() {
    let result = run_analysis("go-gin-basic");
    assert_eq!(result.project_type, DetectedProjectType::BackendGo);
}

// ---------------------------------------------------------------------------
// 4. python-fastapi-basic: backend-python として検出
// ---------------------------------------------------------------------------

#[test]
fn analyze_python_fastapi_basic_detects_backend_python() {
    let result = run_analysis("python-fastapi-basic");
    assert_eq!(result.project_type, DetectedProjectType::BackendPython);
}

// ---------------------------------------------------------------------------
// 5. プラン生成: 5 フェーズであること
// ---------------------------------------------------------------------------

#[test]
fn plan_generation_has_five_phases() {
    let result = run_analysis("rust-axum-basic");
    let plan = analyzer::generate_migration_plan(&result, "my-service");

    assert_eq!(
        plan.phases.len(),
        5,
        "migration plan should have exactly 5 phases, got {}",
        plan.phases.len()
    );

    // フェーズ番号が 1..=5 であること
    for (i, phase) in plan.phases.iter().enumerate() {
        assert_eq!(
            phase.number,
            i + 1,
            "phase number should be sequential starting from 1"
        );
    }

    // プラン名が feature 名を含むこと
    assert!(
        plan.name.contains("my-service"),
        "plan name should contain the feature name, got: {}",
        plan.name
    );

    // プロジェクトタイプが保持されていること
    assert_eq!(plan.project_type, DetectedProjectType::BackendRust);

    // 総ステップ数が 0 より大きいこと
    assert!(
        plan.total_steps() > 0,
        "plan should have at least one step"
    );

    // scores_before が設定されていること
    assert!(plan.scores_before.overall <= 100);
}

// ---------------------------------------------------------------------------
// 6. --dry-run: ファイルが書き込まれないこと
// ---------------------------------------------------------------------------

#[test]
fn plan_dry_run_does_not_write_file() {
    let tmp = tempfile::tempdir().expect("failed to create temp dir");

    // フィクスチャの内容を一時ディレクトリにコピー
    let fixture = fixtures_dir().join("rust-axum-basic");
    copy_dir_recursive(&fixture, tmp.path());

    let output_path = tmp.path().join("migration-plan.json");

    // ファイルが存在しないことを確認
    assert!(
        !output_path.exists(),
        "migration-plan.json should not exist before plan generation"
    );

    // 分析とプラン生成を実行（ファイル書き込みは行わない）
    let project_type = analyzer::detect_project_type(tmp.path());
    let structure = analyzer::analyze_structure(tmp.path(), &project_type);
    let violations = analyzer::scan_violations(tmp.path(), &project_type);
    let dependencies = analyzer::analyze_dependencies(tmp.path(), &project_type);
    let scores = analyzer::calculate_scores(&structure, &violations, &dependencies);

    let result = AnalysisResult {
        project_type,
        structure,
        violations,
        dependencies,
        scores,
    };

    // プラン生成自体はメモリ上のみ
    let plan = analyzer::generate_migration_plan(&result, "dry-run-test");
    assert!(!plan.phases.is_empty());

    // dry-run なのでファイルは書かれない
    assert!(
        !output_path.exists(),
        "migration-plan.json should not exist after dry-run plan generation"
    );
}

// ---------------------------------------------------------------------------
// ヘルパー: ディレクトリの再帰コピー
// ---------------------------------------------------------------------------

fn copy_dir_recursive(src: &Path, dst: &Path) {
    for entry in std::fs::read_dir(src).expect("failed to read dir") {
        let entry = entry.expect("failed to read entry");
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            std::fs::create_dir_all(&dst_path).expect("failed to create dir");
            copy_dir_recursive(&src_path, &dst_path);
        } else {
            std::fs::copy(&src_path, &dst_path).expect("failed to copy file");
        }
    }
}
