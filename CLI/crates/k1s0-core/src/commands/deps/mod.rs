//! 依存関係マップ機能。
//!
//! regions/ 配下のサービス間依存を解析し、
//! Tier間ルール違反を検出する。

pub mod cache;
pub mod output;
pub mod rules;
pub mod scanner;
pub mod types;

use std::path::Path;

use anyhow::Result;

pub use types::*;

/// 依存関係マップを実行する。
///
/// 指定された設定に基づいてサービスを走査し、依存関係を解析し、
/// ルール違反を検出して結果を返す。
///
/// # Errors
///
/// ファイル読み込みやキャッシュ操作に失敗した場合にエラーを返す。
pub fn execute_deps(config: &DepsConfig) -> Result<DepsResult> {
    execute_deps_at(Path::new("."), config)
}

/// 指定ディレクトリを基点に依存関係マップを実行する（テスト用）。
///
/// # Errors
///
/// ファイル読み込みやキャッシュ操作に失敗した場合にエラーを返す。
pub fn execute_deps_at(base_dir: &Path, config: &DepsConfig) -> Result<DepsResult> {
    let cache_dir = base_dir.join(".k1s0");

    // キャッシュ確認
    if !config.no_cache {
        if let Some(cached) = cache::load_cache(&cache_dir) {
            let current_hashes = cache::compute_file_hashes(base_dir)?;
            if cache::is_cache_valid(&cached, &current_hashes) {
                let mut result = DepsResult {
                    services: scan_services_at(base_dir),
                    dependencies: cached.dependencies,
                    violations: cached.violations,
                };
                filter_result_by_scope(&mut result, &config.scope);
                return Ok(result);
            }
        }
    }

    // サービス走査
    let services = scan_services_at(base_dir);

    // 依存関係解析
    let mut dependencies = Vec::new();
    dependencies.extend(scanner::scan_grpc_dependencies(&services, base_dir));
    dependencies.extend(scanner::scan_kafka_dependencies(&services, base_dir));
    dependencies.extend(scanner::scan_rest_dependencies(&services, base_dir));
    dependencies.extend(scanner::scan_library_dependencies(&services, base_dir));

    // ルール違反検出
    let violations = rules::check_violations(&dependencies, &services);

    // キャッシュ保存
    if !config.no_cache {
        if let Ok(hashes) = cache::compute_file_hashes(base_dir) {
            let _ = cache::save_cache(&cache_dir, &dependencies, &violations, &hashes);
        }
    }

    let mut result = DepsResult {
        services,
        dependencies,
        violations,
    };

    filter_result_by_scope(&mut result, &config.scope);

    Ok(result)
}

/// スコープに基づいて結果をフィルタリングする。
fn filter_result_by_scope(result: &mut DepsResult, scope: &DepsScope) {
    match scope {
        DepsScope::All => {}
        DepsScope::Tier(tier) => {
            result.services.retain(|s| s.tier == *tier);
            let tier = tier.clone();
            result
                .dependencies
                .retain(|d| d.source_tier == tier || d.target_tier == tier);
            result
                .violations
                .retain(|v| v.source_tier == tier || v.target_tier == tier);
        }
        DepsScope::Services(names) => {
            result.services.retain(|s| names.contains(&s.name));
            result
                .dependencies
                .retain(|d| names.contains(&d.source) || names.contains(&d.target));
            result
                .violations
                .retain(|v| names.contains(&v.source) || names.contains(&v.target));
        }
    }
}

/// regions/ 配下のサーバーを走査してサービス情報を返す。
pub fn scan_services(base_dir: &Path) -> Vec<ServiceInfo> {
    scan_services_at(base_dir)
}

/// 指定ディレクトリを基点にサービスを走査する（テスト用）。
pub fn scan_services_at(base_dir: &Path) -> Vec<ServiceInfo> {
    scanner::scan_services(base_dir)
}
