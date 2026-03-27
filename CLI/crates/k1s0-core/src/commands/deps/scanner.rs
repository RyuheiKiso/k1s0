//! サービス走査と依存関係解析。
//!
//! regions/ 配下のディレクトリ構造からサービスを検出し、
//! proto, config.yaml, ソースコード, パッケージ定義から依存関係を抽出する。

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::OnceLock;

use regex::Regex;

use super::types::{Dependency, DependencyType, ServiceInfo};

/// gRPC proto import パターンの正規表現キャッシュ
static GRPC_IMPORT_RE: OnceLock<Regex> = OnceLock::new();
/// Kafka トピック名パターンの正規表現キャッシュ
static TOPIC_RE: OnceLock<Regex> = OnceLock::new();
/// REST サービス名パターンの正規表現キャッシュ
static REST_SERVICE_RE: OnceLock<Regex> = OnceLock::new();
/// GraphQL パターンの正規表現キャッシュ
static GRAPHQL_RE: OnceLock<Regex> = OnceLock::new();
/// Cargo.toml k1s0 ライブラリ依存パターンの正規表現キャッシュ
static CARGO_LIB_RE: OnceLock<Regex> = OnceLock::new();
/// go.mod k1s0 ライブラリ依存パターンの正規表現キャッシュ
static GOMOD_LIB_RE: OnceLock<Regex> = OnceLock::new();
/// package.json @k1s0 ライブラリ依存パターンの正規表現キャッシュ
static NPM_LIB_RE: OnceLock<Regex> = OnceLock::new();
/// pubspec.yaml k1s0_ ライブラリ依存パターンの正規表現キャッシュ
static DART_LIB_RE: OnceLock<Regex> = OnceLock::new();

/// regions/ 配下のサーバーを走査してサービス情報を返す。
///
/// ディレクトリ構造:
/// - system: regions/system/server/{lang}/{name}/
/// - business: regions/business/{domain}/server/{lang}/
/// - service: regions/service/{domain}/server/{lang}/
pub fn scan_services(base_dir: &Path) -> Vec<ServiceInfo> {
    let mut services = Vec::new();
    let regions = base_dir.join("regions");
    if !regions.is_dir() {
        return services;
    }

    // system tier: regions/system/server/{lang}/{name}/
    scan_system_servers(&regions.join("system").join("server"), &mut services);

    // business tier: regions/business/{domain}/server/{lang}/
    scan_domain_servers(&regions.join("business"), "business", &mut services);

    // service tier: regions/service/{domain}/server/{lang}/
    scan_domain_servers(&regions.join("service"), "service", &mut services);

    services.sort_by(|a, b| a.name.cmp(&b.name));
    services
}

/// system tier のサーバーを走査する。
/// regions/system/server/{lang}/{name}/
fn scan_system_servers(server_dir: &Path, services: &mut Vec<ServiceInfo>) {
    if !server_dir.is_dir() {
        return;
    }
    for lang_entry in read_dir_sorted(server_dir) {
        let lang = lang_entry.file_name().to_string_lossy().to_string();
        let language = normalize_language(&lang);
        if language.is_empty() {
            continue;
        }
        for name_entry in read_dir_sorted(&lang_entry.path()) {
            let name = name_entry.file_name().to_string_lossy().to_string();
            if is_server_dir(&name_entry.path()) {
                services.push(ServiceInfo {
                    name: format!("{name}-server"),
                    tier: "system".to_string(),
                    domain: None,
                    language: language.clone(),
                    path: name_entry.path(),
                });
            }
        }
    }
}

/// business/service tier のサーバーを走査する。
/// regions/{tier}/{domain}/server/{lang}/
fn scan_domain_servers(tier_dir: &Path, tier: &str, services: &mut Vec<ServiceInfo>) {
    if !tier_dir.is_dir() {
        return;
    }
    for domain_entry in read_dir_sorted(tier_dir) {
        let domain = domain_entry.file_name().to_string_lossy().to_string();
        let server_dir = domain_entry.path().join("server");
        if !server_dir.is_dir() {
            continue;
        }
        for lang_entry in read_dir_sorted(&server_dir) {
            let lang = lang_entry.file_name().to_string_lossy().to_string();
            let language = normalize_language(&lang);
            if language.is_empty() {
                continue;
            }
            if is_server_dir(&lang_entry.path()) {
                services.push(ServiceInfo {
                    name: format!("{domain}-server"),
                    tier: tier.to_string(),
                    domain: Some(domain.clone()),
                    language: language.clone(),
                    path: lang_entry.path(),
                });
            }
        }
    }
}

/// ディレクトリ内のエントリをソートして返す。
/// シンボリックリンクはリポジトリ外を指す可能性があるため除外する。
fn read_dir_sorted(dir: &Path) -> Vec<fs::DirEntry> {
    let mut entries: Vec<fs::DirEntry> = fs::read_dir(dir)
        .ok()
        .into_iter()
        .flat_map(std::iter::Iterator::flatten)
        // シンボリックリンクを除外してからディレクトリかどうかを判定する
        .filter(|e| !e.path().is_symlink() && e.path().is_dir())
        .collect();
    entries.sort_by_key(std::fs::DirEntry::file_name);
    entries
}

/// 言語ディレクトリ名を正規化する。
fn normalize_language(lang: &str) -> String {
    match lang {
        "rust" => "rust".to_string(),
        "go" => "go".to_string(),
        "ts" | "typescript" => "typescript".to_string(),
        "dart" | "flutter" => "dart".to_string(),
        _ => String::new(),
    }
}

/// サーバーディレクトリかどうかを判定する。
/// Cargo.toml, go.mod, package.json, pubspec.yaml のいずれかが存在するか。
fn is_server_dir(path: &Path) -> bool {
    path.join("Cargo.toml").exists()
        || path.join("go.mod").exists()
        || path.join("package.json").exists()
        || path.join("pubspec.yaml").exists()
}

// ============================================================================
// gRPC 依存解析
// ============================================================================

/// protoファイルのimport文からサービス間依存を検出する。
///
/// パターン: `import "k1s0/{tier}/{domain}/v{n}/{file}.proto"`
/// → ターゲット: `{domain}-server` ({tier})
/// Scan gRPC dependencies from imported proto files.
///
/// # Panics
///
/// Panics only if a hard-coded regular expression becomes invalid.
pub fn scan_grpc_dependencies(services: &[ServiceInfo], _base_dir: &Path) -> Vec<Dependency> {
    let mut deps = Vec::new();
    // OnceLock で正規表現を一度だけコンパイルしてキャッシュする
    // 静的正規表現のコンパイル失敗はプログラミングエラーのため expect で即時パニックする
    let re = GRPC_IMPORT_RE.get_or_init(|| {
        Regex::new(r#"import\s+"k1s0/(\w+)/(\w[\w-]*)/v\d+/"#).expect("static regex")
    });

    // サービス名→tier のマップを構築
    let service_tier_map: HashMap<String, String> = services
        .iter()
        .map(|s| (s.name.clone(), s.tier.clone()))
        .collect();

    for service in services {
        let proto_files = find_files_with_extension(&service.path, "proto");
        for proto_file in &proto_files {
            if let Ok(content) = fs::read_to_string(proto_file) {
                for cap in re.captures_iter(&content) {
                    let target_tier = &cap[1];
                    let target_domain = &cap[2];

                    // 共通型定義（common）は依存としてカウントしない
                    if target_domain == "common" {
                        continue;
                    }

                    let target_name = format!("{target_domain}-server");

                    // 自分自身への依存はスキップ
                    if target_name == service.name {
                        continue;
                    }

                    let location = proto_file.to_string_lossy().to_string();
                    // 既存の依存にlocationを追加するか、新規作成
                    if let Some(existing) = deps.iter_mut().find(|d: &&mut Dependency| {
                        d.source == service.name
                            && d.target == target_name
                            && d.dep_type == DependencyType::Grpc
                    }) {
                        if !existing.locations.contains(&location) {
                            existing.locations.push(location);
                        }
                    } else {
                        deps.push(Dependency {
                            source: service.name.clone(),
                            source_tier: service.tier.clone(),
                            target: target_name.clone(),
                            target_tier: service_tier_map
                                .get(&target_name)
                                .cloned()
                                .unwrap_or_else(|| target_tier.to_string()),
                            dep_type: DependencyType::Grpc,
                            locations: vec![location],
                            detail: None,
                        });
                    }
                }
            }
        }
    }

    deps
}

// ============================================================================
// Kafka 依存解析
// ============================================================================

/// config.yamlのkafka.topicsセクションからpublish/subscribeを解析する。
///
/// トピック命名: `k1s0.{tier}.{domain}.{event}.v{n}`
///
/// # Panics
///
/// トピック名照合用の静的正規表現の初期化に失敗した場合（正規表現構文エラー）にパニックする。
/// 正規表現はコンパイル時に検証済みのため、通常はパニックしない。
pub fn scan_kafka_dependencies(services: &[ServiceInfo], _base_dir: &Path) -> Vec<Dependency> {
    let mut deps = Vec::new();
    // トピック名→公開元サービスのマップ
    let mut topic_publishers: HashMap<String, Vec<String>> = HashMap::new();
    // トピック名→購読先サービスのマップ
    let mut topic_subscribers: HashMap<String, Vec<String>> = HashMap::new();

    let service_tier_map: HashMap<String, String> = services
        .iter()
        .map(|s| (s.name.clone(), s.tier.clone()))
        .collect();

    for service in services {
        let config_path = service.path.join("config").join("config.yaml");
        if !config_path.exists() {
            // config/ ディレクトリ直下でなければルート直下も確認
            let alt_config = service.path.join("config.yaml");
            if alt_config.exists() {
                parse_kafka_config(
                    &alt_config,
                    &service.name,
                    &mut topic_publishers,
                    &mut topic_subscribers,
                );
            }
            continue;
        }
        parse_kafka_config(
            &config_path,
            &service.name,
            &mut topic_publishers,
            &mut topic_subscribers,
        );
    }

    // パブリッシャーとサブスクライバーをマッチさせて依存関係を構築
    // OnceLock で正規表現を一度だけコンパイルしてキャッシュする
    // 静的正規表現のコンパイル失敗はプログラミングエラーのため expect で即時パニックする
    let topic_re = Some(
        TOPIC_RE.get_or_init(|| Regex::new(r"k1s0\.(\w+)\.(\w[\w-]*)\.").expect("static regex")),
    );

    for (topic, subscribers) in &topic_subscribers {
        for subscriber in subscribers {
            // パブリッシャーが明示的にいる場合
            if let Some(publishers) = topic_publishers.get(topic) {
                for publisher in publishers {
                    if publisher == subscriber {
                        continue;
                    }
                    deps.push(Dependency {
                        source: subscriber.clone(),
                        source_tier: service_tier_map
                            .get(subscriber)
                            .cloned()
                            .unwrap_or_default(),
                        target: publisher.clone(),
                        target_tier: service_tier_map.get(publisher).cloned().unwrap_or_default(),
                        dep_type: DependencyType::Kafka,
                        locations: vec![format!("kafka topic: {topic}")],
                        detail: Some(topic.clone()),
                    });
                }
            } else if let Some(re) = topic_re {
                // パブリッシャーがいない場合、トピック名からターゲットを推測
                if let Some(cap) = re.captures(topic) {
                    let target_tier = &cap[1];
                    let target_domain = &cap[2];
                    let target_name = format!("{target_domain}-server");
                    if target_name != *subscriber {
                        deps.push(Dependency {
                            source: subscriber.clone(),
                            source_tier: service_tier_map
                                .get(subscriber)
                                .cloned()
                                .unwrap_or_default(),
                            target: target_name,
                            target_tier: target_tier.to_string(),
                            dep_type: DependencyType::Kafka,
                            locations: vec![format!("kafka topic: {topic}")],
                            detail: Some(topic.clone()),
                        });
                    }
                }
            }
        }
    }

    deps
}

/// config.yamlからkafkaトピック設定を解析する。
fn parse_kafka_config(
    config_path: &Path,
    service_name: &str,
    publishers: &mut HashMap<String, Vec<String>>,
    subscribers: &mut HashMap<String, Vec<String>>,
) {
    let Ok(content) = fs::read_to_string(config_path) else {
        return;
    };

    let value: serde_yaml::Value = match serde_yaml::from_str(&content) {
        Ok(v) => v,
        Err(_) => return,
    };

    if let Some(kafka) = value.get("kafka").and_then(|k| k.get("topics")) {
        // publish トピック
        if let Some(publish) = kafka.get("publish") {
            extract_topics(publish, service_name, publishers);
        }
        // subscribe トピック
        if let Some(subscribe) = kafka.get("subscribe") {
            extract_topics(subscribe, service_name, subscribers);
        }
    }
}

/// YAMLの配列またはマップからトピック名を抽出する。
fn extract_topics(
    value: &serde_yaml::Value,
    service_name: &str,
    map: &mut HashMap<String, Vec<String>>,
) {
    match value {
        serde_yaml::Value::Sequence(seq) => {
            for item in seq {
                if let Some(topic) = item.as_str() {
                    map.entry(topic.to_string())
                        .or_default()
                        .push(service_name.to_string());
                } else if let Some(topic) = item.get("name").and_then(|n| n.as_str()) {
                    map.entry(topic.to_string())
                        .or_default()
                        .push(service_name.to_string());
                }
            }
        }
        serde_yaml::Value::Mapping(mapping) => {
            for (key, _) in mapping {
                if let Some(topic) = key.as_str() {
                    map.entry(topic.to_string())
                        .or_default()
                        .push(service_name.to_string());
                }
            }
        }
        _ => {}
    }
}

// ============================================================================
// REST/GraphQL 依存解析
// ============================================================================

/// ソースコード内のHTTPクライアントURLパターンからサービス間依存を検出する。
///
/// パターン: `{service-name}.k1s0-{tier}`
/// Scan REST and GraphQL dependencies from source files.
///
/// # Panics
///
/// Panics only if a hard-coded regular expression becomes invalid.
pub fn scan_rest_dependencies(services: &[ServiceInfo], _base_dir: &Path) -> Vec<Dependency> {
    let mut deps = Vec::new();
    // OnceLock で正規表現を一度だけコンパイルしてキャッシュする
    // 静的正規表現のコンパイル失敗はプログラミングエラーのため expect で即時パニックする
    let re = REST_SERVICE_RE.get_or_init(|| {
        Regex::new(r"([\w-]+)\.k1s0-(system|business|service)").expect("static regex")
    });
    let graphql_re =
        GRAPHQL_RE.get_or_init(|| Regex::new(r"(?i)graphql|/graphql").expect("static regex"));

    let service_tier_map: HashMap<String, String> = services
        .iter()
        .map(|s| (s.name.clone(), s.tier.clone()))
        .collect();

    let extensions = ["rs", "go", "ts", "dart"];

    for service in services {
        let source_files = find_files_with_extensions(&service.path, &extensions);
        for source_file in &source_files {
            if let Ok(content) = fs::read_to_string(source_file) {
                for cap in re.captures_iter(&content) {
                    let target_service = &cap[1];
                    let target_tier = &cap[2];
                    let target_name = format!("{target_service}-server");

                    // 自分自身への依存はスキップ
                    if target_name == service.name {
                        continue;
                    }

                    // GraphQLかRESTかを判定
                    let dep_type = if graphql_re.is_match(&content) {
                        DependencyType::GraphQL
                    } else {
                        DependencyType::Rest
                    };

                    let location = source_file.to_string_lossy().to_string();

                    if let Some(existing) = deps.iter_mut().find(|d: &&mut Dependency| {
                        d.source == service.name
                            && d.target == target_name
                            && d.dep_type == dep_type
                    }) {
                        if !existing.locations.contains(&location) {
                            existing.locations.push(location);
                        }
                    } else {
                        deps.push(Dependency {
                            source: service.name.clone(),
                            source_tier: service.tier.clone(),
                            target: target_name.clone(),
                            target_tier: service_tier_map
                                .get(&target_name)
                                .cloned()
                                .unwrap_or_else(|| target_tier.to_string()),
                            dep_type,
                            locations: vec![location],
                            detail: None,
                        });
                    }
                }
            }
        }
    }

    deps
}

// ============================================================================
// ライブラリ依存解析
// ============================================================================

/// パッケージ定義ファイルからk1s0系ライブラリ依存を検出する。
///
/// - Cargo.toml: `k1s0-{lib}` パス依存
/// - go.mod: `k1s0/regions/system/library/go/`
/// - package.json: `@k1s0/`
/// - pubspec.yaml: `k1s0_`
///
/// Scan shared-library dependencies from package manifests.
///
/// # Panics
///
/// Panics only if a hard-coded regular expression becomes invalid.
pub fn scan_library_dependencies(services: &[ServiceInfo], _base_dir: &Path) -> Vec<Dependency> {
    let mut deps = Vec::new();

    // 静的正規表現のコンパイル失敗はプログラミングエラーのため expect で即時パニックする
    let cargo_re = CARGO_LIB_RE.get_or_init(|| Regex::new(r"k1s0-([\w-]+)").expect("static regex"));
    let gomod_re = GOMOD_LIB_RE.get_or_init(|| {
        Regex::new(r"k1s0/regions/system/library/go/([\w-]+)").expect("static regex")
    });
    let npm_re = NPM_LIB_RE.get_or_init(|| Regex::new(r"@k1s0/([\w-]+)").expect("static regex"));
    let dart_re = DART_LIB_RE.get_or_init(|| Regex::new(r"k1s0_([\w]+)").expect("static regex"));

    for service in services {
        // Cargo.toml
        let cargo_toml = service.path.join("Cargo.toml");
        if cargo_toml.exists() {
            if let Ok(content) = fs::read_to_string(&cargo_toml) {
                let location = cargo_toml.to_string_lossy().to_string();
                for cap in cargo_re.captures_iter(&content) {
                    let lib_name = &cap[1];
                    add_library_dep(&mut deps, &service.name, &service.tier, lib_name, &location);
                }
            }
        }

        // go.mod
        let gomod = service.path.join("go.mod");
        if gomod.exists() {
            if let Ok(content) = fs::read_to_string(&gomod) {
                let location = gomod.to_string_lossy().to_string();
                for cap in gomod_re.captures_iter(&content) {
                    let lib_name = &cap[1];
                    add_library_dep(&mut deps, &service.name, &service.tier, lib_name, &location);
                }
            }
        }

        // package.json
        let package_json = service.path.join("package.json");
        if package_json.exists() {
            if let Ok(content) = fs::read_to_string(&package_json) {
                let location = package_json.to_string_lossy().to_string();
                for cap in npm_re.captures_iter(&content) {
                    let lib_name = &cap[1];
                    add_library_dep(&mut deps, &service.name, &service.tier, lib_name, &location);
                }
            }
        }

        // pubspec.yaml
        let pubspec = service.path.join("pubspec.yaml");
        if pubspec.exists() {
            if let Ok(content) = fs::read_to_string(&pubspec) {
                let location = pubspec.to_string_lossy().to_string();
                for cap in dart_re.captures_iter(&content) {
                    let lib_name = &cap[1];
                    add_library_dep(&mut deps, &service.name, &service.tier, lib_name, &location);
                }
            }
        }
    }

    deps
}

/// ライブラリ依存を追加する。
fn add_library_dep(
    deps: &mut Vec<Dependency>,
    source: &str,
    source_tier: &str,
    lib_name: &str,
    location: &str,
) {
    let target = format!("k1s0-{lib_name}");

    if let Some(existing) = deps.iter_mut().find(|d: &&mut Dependency| {
        d.source == source && d.target == target && d.dep_type == DependencyType::Library
    }) {
        if !existing.locations.contains(&location.to_string()) {
            existing.locations.push(location.to_string());
        }
    } else {
        deps.push(Dependency {
            source: source.to_string(),
            source_tier: source_tier.to_string(),
            target,
            target_tier: "system".to_string(), // ライブラリは常にsystem tier
            dep_type: DependencyType::Library,
            locations: vec![location.to_string()],
            detail: Some(lib_name.to_string()),
        });
    }
}

// ============================================================================
// ファイル検索ユーティリティ
// ============================================================================

/// 指定拡張子のファイルを再帰的に検索する。
fn find_files_with_extension(dir: &Path, ext: &str) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    find_files_recursive(dir, &[ext], &mut files);
    files
}

/// 複数の拡張子でファイルを再帰的に検索する。
fn find_files_with_extensions(dir: &Path, exts: &[&str]) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    find_files_recursive(dir, exts, &mut files);
    files
}

/// 再帰的にファイルを検索する。
fn find_files_recursive(dir: &Path, exts: &[&str], files: &mut Vec<std::path::PathBuf>) {
    if !dir.is_dir() {
        return;
    }
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // target/, node_modules/ などはスキップ
                let dir_name = path.file_name().unwrap_or_default().to_string_lossy();
                if dir_name == "target"
                    || dir_name == "node_modules"
                    || dir_name == ".git"
                    || dir_name == "vendor"
                {
                    continue;
                }
                find_files_recursive(&path, exts, files);
            } else if let Some(file_ext) = path.extension() {
                let ext_str = file_ext.to_string_lossy();
                if exts.iter().any(|e| *e == ext_str.as_ref()) {
                    files.push(path);
                }
            }
        }
    }
}

// テストコードでは unwrap() の使用を許可する
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ========================================================================
    // scan_services テスト
    // ========================================================================

    #[test]
    fn test_scan_services_empty_dir() {
        let tmp = TempDir::new().unwrap();
        let services = scan_services(tmp.path());
        assert!(services.is_empty());
    }

    #[test]
    fn test_scan_services_no_regions() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("src")).unwrap();
        let services = scan_services(tmp.path());
        assert!(services.is_empty());
    }

    #[test]
    fn test_scan_services_system_rust() {
        let tmp = TempDir::new().unwrap();
        let auth_dir = tmp.path().join("regions/system/server/rust/auth");
        fs::create_dir_all(&auth_dir).unwrap();
        fs::write(auth_dir.join("Cargo.toml"), "[package]\nname = \"auth\"").unwrap();

        let services = scan_services(tmp.path());
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].name, "auth-server");
        assert_eq!(services[0].tier, "system");
        assert_eq!(services[0].language, "rust");
        assert!(services[0].domain.is_none());
    }

    #[test]
    fn test_scan_services_system_go() {
        let tmp = TempDir::new().unwrap();
        let bff_dir = tmp.path().join("regions/system/server/go/bff-proxy");
        fs::create_dir_all(&bff_dir).unwrap();
        fs::write(bff_dir.join("go.mod"), "module bff-proxy").unwrap();

        let services = scan_services(tmp.path());
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].name, "bff-proxy-server");
        assert_eq!(services[0].tier, "system");
        assert_eq!(services[0].language, "go");
    }

    #[test]
    fn test_scan_services_business_tier() {
        let tmp = TempDir::new().unwrap();
        let tm_dir = tmp
            .path()
            .join("regions/business/taskmanagement/server/rust");
        fs::create_dir_all(&tm_dir).unwrap();
        fs::write(tm_dir.join("Cargo.toml"), "[package]").unwrap();

        let services = scan_services(tmp.path());
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].name, "taskmanagement-server");
        assert_eq!(services[0].tier, "business");
        assert_eq!(services[0].domain, Some("taskmanagement".to_string()));
        assert_eq!(services[0].language, "rust");
    }

    #[test]
    fn test_scan_services_service_tier() {
        let tmp = TempDir::new().unwrap();
        let task_dir = tmp.path().join("regions/service/task/server/go");
        fs::create_dir_all(&task_dir).unwrap();
        fs::write(task_dir.join("go.mod"), "module task-server").unwrap();

        let services = scan_services(tmp.path());
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].name, "task-server");
        assert_eq!(services[0].tier, "service");
        assert_eq!(services[0].domain, Some("task".to_string()));
    }

    #[test]
    fn test_scan_services_multiple() {
        let tmp = TempDir::new().unwrap();

        // system
        let auth_dir = tmp.path().join("regions/system/server/rust/auth");
        fs::create_dir_all(&auth_dir).unwrap();
        fs::write(auth_dir.join("Cargo.toml"), "[package]").unwrap();

        // business
        let tm_dir = tmp
            .path()
            .join("regions/business/taskmanagement/server/rust");
        fs::create_dir_all(&tm_dir).unwrap();
        fs::write(tm_dir.join("Cargo.toml"), "[package]").unwrap();

        // service
        let task_dir = tmp.path().join("regions/service/task/server/go");
        fs::create_dir_all(&task_dir).unwrap();
        fs::write(task_dir.join("go.mod"), "module task").unwrap();

        let services = scan_services(tmp.path());
        assert_eq!(services.len(), 3);
        // ソート済み
        assert_eq!(services[0].name, "auth-server");
        assert_eq!(services[1].name, "task-server");
        assert_eq!(services[2].name, "taskmanagement-server");
    }

    #[test]
    fn test_scan_services_ignores_non_server_dirs() {
        let tmp = TempDir::new().unwrap();
        // サーバーディレクトリだがプロジェクトファイルなし
        let empty_dir = tmp.path().join("regions/system/server/rust/empty");
        fs::create_dir_all(&empty_dir).unwrap();

        let services = scan_services(tmp.path());
        assert!(services.is_empty());
    }

    // ========================================================================
    // gRPC 依存解析テスト
    // ========================================================================

    #[test]
    fn test_scan_grpc_dependencies_with_import() {
        let tmp = TempDir::new().unwrap();

        let auth_dir = tmp.path().join("regions/system/server/rust/auth");
        fs::create_dir_all(auth_dir.join("src/proto")).unwrap();
        fs::write(auth_dir.join("Cargo.toml"), "[package]").unwrap();

        let config_dir = tmp.path().join("regions/system/server/rust/config");
        fs::create_dir_all(config_dir.join("src")).unwrap();
        fs::write(config_dir.join("Cargo.toml"), "[package]").unwrap();
        fs::write(
            config_dir.join("src/service.proto"),
            r#"
syntax = "proto3";
import "k1s0/system/auth/v1/auth.proto";
package k1s0.system.config.v1;
"#,
        )
        .unwrap();

        let services = scan_services(tmp.path());
        let deps = scan_grpc_dependencies(&services, tmp.path());
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].source, "config-server");
        assert_eq!(deps[0].target, "auth-server");
        assert_eq!(deps[0].dep_type, DependencyType::Grpc);
    }

    #[test]
    fn test_scan_grpc_dependencies_common_excluded() {
        let tmp = TempDir::new().unwrap();

        let auth_dir = tmp.path().join("regions/system/server/rust/auth");
        fs::create_dir_all(auth_dir.join("src")).unwrap();
        fs::write(auth_dir.join("Cargo.toml"), "[package]").unwrap();
        fs::write(
            auth_dir.join("src/service.proto"),
            r#"
syntax = "proto3";
import "k1s0/system/common/v1/types.proto";
import "k1s0/system/config/v1/config.proto";
"#,
        )
        .unwrap();

        let config_dir = tmp.path().join("regions/system/server/rust/config");
        fs::create_dir_all(&config_dir).unwrap();
        fs::write(config_dir.join("Cargo.toml"), "[package]").unwrap();

        let services = scan_services(tmp.path());
        let deps = scan_grpc_dependencies(&services, tmp.path());
        // commonは除外されるのでconfig-serverへの依存のみ
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].target, "config-server");
    }

    #[test]
    fn test_scan_grpc_dependencies_no_self_reference() {
        let tmp = TempDir::new().unwrap();

        let auth_dir = tmp.path().join("regions/system/server/rust/auth");
        fs::create_dir_all(auth_dir.join("src")).unwrap();
        fs::write(auth_dir.join("Cargo.toml"), "[package]").unwrap();
        fs::write(
            auth_dir.join("src/service.proto"),
            r#"import "k1s0/system/auth/v1/auth.proto";"#,
        )
        .unwrap();

        let services = scan_services(tmp.path());
        let deps = scan_grpc_dependencies(&services, tmp.path());
        assert!(deps.is_empty(), "自分自身への依存は検出されないこと");
    }

    // ========================================================================
    // Kafka 依存解析テスト
    // ========================================================================

    #[test]
    fn test_scan_kafka_dependencies() {
        let tmp = TempDir::new().unwrap();

        // パブリッシャー
        let auth_dir = tmp.path().join("regions/system/server/rust/auth");
        fs::create_dir_all(auth_dir.join("config")).unwrap();
        fs::write(auth_dir.join("Cargo.toml"), "[package]").unwrap();
        fs::write(
            auth_dir.join("config/config.yaml"),
            r#"
kafka:
  topics:
    publish:
      - "k1s0.system.auth.user-created.v1"
"#,
        )
        .unwrap();

        // サブスクライバー
        let notif_dir = tmp.path().join("regions/system/server/rust/notification");
        fs::create_dir_all(notif_dir.join("config")).unwrap();
        fs::write(notif_dir.join("Cargo.toml"), "[package]").unwrap();
        fs::write(
            notif_dir.join("config/config.yaml"),
            r#"
kafka:
  topics:
    subscribe:
      - "k1s0.system.auth.user-created.v1"
"#,
        )
        .unwrap();

        let services = scan_services(tmp.path());
        let deps = scan_kafka_dependencies(&services, tmp.path());
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].source, "notification-server");
        assert_eq!(deps[0].target, "auth-server");
        assert_eq!(deps[0].dep_type, DependencyType::Kafka);
        assert!(deps[0].detail.as_ref().unwrap().contains("user-created"));
    }

    // ========================================================================
    // REST 依存解析テスト
    // ========================================================================

    #[test]
    fn test_scan_rest_dependencies() {
        let tmp = TempDir::new().unwrap();

        let task_dir = tmp.path().join("regions/service/task/server/rust");
        fs::create_dir_all(task_dir.join("src")).unwrap();
        fs::write(task_dir.join("Cargo.toml"), "[package]").unwrap();
        fs::write(
            task_dir.join("src/client.rs"),
            r#"let url = "http://auth.k1s0-system:8080/api/v1/verify";"#,
        )
        .unwrap();

        let auth_dir = tmp.path().join("regions/system/server/rust/auth");
        fs::create_dir_all(auth_dir.join("src")).unwrap();
        fs::write(auth_dir.join("Cargo.toml"), "[package]").unwrap();

        let services = scan_services(tmp.path());
        let deps = scan_rest_dependencies(&services, tmp.path());
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].source, "task-server");
        assert_eq!(deps[0].target, "auth-server");
        assert_eq!(deps[0].dep_type, DependencyType::Rest);
    }

    // ========================================================================
    // ライブラリ依存解析テスト
    // ========================================================================

    #[test]
    fn test_scan_library_dependencies_cargo() {
        let tmp = TempDir::new().unwrap();

        let auth_dir = tmp.path().join("regions/system/server/rust/auth");
        fs::create_dir_all(&auth_dir).unwrap();
        fs::write(
            auth_dir.join("Cargo.toml"),
            r#"
[package]
name = "auth"

[dependencies]
k1s0-observability = { path = "../../library/rust/observability" }
k1s0-messaging = { path = "../../library/rust/messaging" }
"#,
        )
        .unwrap();

        let services = scan_services(tmp.path());
        let deps = scan_library_dependencies(&services, tmp.path());
        assert_eq!(deps.len(), 2);
        assert!(deps.iter().any(|d| d.target == "k1s0-observability"));
        assert!(deps.iter().any(|d| d.target == "k1s0-messaging"));
    }

    #[test]
    fn test_scan_library_dependencies_gomod() {
        let tmp = TempDir::new().unwrap();

        let bff_dir = tmp.path().join("regions/system/server/go/bff-proxy");
        fs::create_dir_all(&bff_dir).unwrap();
        fs::write(
            bff_dir.join("go.mod"),
            r"
module bff-proxy

require (
    k1s0/regions/system/library/go/observability v0.0.0
)
",
        )
        .unwrap();

        let services = scan_services(tmp.path());
        let deps = scan_library_dependencies(&services, tmp.path());
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].target, "k1s0-observability");
    }

    // ========================================================================
    // ユーティリティテスト
    // ========================================================================

    #[test]
    fn test_normalize_language() {
        assert_eq!(normalize_language("rust"), "rust");
        assert_eq!(normalize_language("go"), "go");
        assert_eq!(normalize_language("ts"), "typescript");
        assert_eq!(normalize_language("typescript"), "typescript");
        assert_eq!(normalize_language("dart"), "dart");
        assert_eq!(normalize_language("flutter"), "dart");
        assert_eq!(normalize_language("python"), "");
    }

    #[test]
    fn test_is_server_dir() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("test-server");
        fs::create_dir_all(&dir).unwrap();

        // プロジェクトファイルなし
        assert!(!is_server_dir(&dir));

        // Cargo.toml あり
        fs::write(dir.join("Cargo.toml"), "[package]").unwrap();
        assert!(is_server_dir(&dir));
    }
}
