// 本ファイルは ConfigMap mount 配下の JDM ファイルを監視するホットリローダ。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/09_Decision_API.md
//     - FR-T1-DECISION-004（決定表ホットリロード）
//   docs/02_構想設計/adr/ADR-RULE-001-zen-engine.md
//
// 役割:
//   tier1 Pod の起動時に指定ディレクトリ配下の `*.json` JDM ファイルを全件読み込み、
//   RuleRegistry に登録する。以後、ファイルシステム変更を検知して同じ rule_id に
//   新バージョンとして再登録する（registry の register() がバージョンを +1 する）。
//
// 受け入れ基準（FR-T1-DECISION-004）:
//   - Git push から新版評価開始まで 5 分以内
//   - 再読込中の評価は旧版 or 新版のどちらかで一貫
//   - 再読込で tier1 Pod 再起動は不要
//
// 設計上の決定:
//   - ファイル名（拡張子除外）が rule_id になる（例: "payment-approval.json" → "payment-approval"）
//   - 同 rule_id を再 register すると registry が v1 → v2 → v3 とバージョンを進める
//     （既存版は保持され、historical evaluation で参照可能、FR-T1-DECISION-002 と整合）
//   - ファイル削除は registry 上では削除しない（履歴は保持、運用方針）
//   - watcher は notify crate の RecommendedWatcher（OS native: inotify / FSEvents 等）を使う
//   - 5 分以内要件は inotify レイテンシ（数 ms）+ 反映処理（数 ms）で十分余裕がある
//
// 一貫性保証:
//   reload 中は registry の write lock を短時間取るのみ。evaluate 経路は read lock で
//   `Arc<CompiledRule>` を取得した時点で snapshot を保持するため、その評価は旧版で一貫する。
//   後続の evaluate（rule_version 未指定）は最新版を使う。
//
// 障害ハンドリング:
//   - ファイル read 失敗 / JSON parse 失敗 / JDM 不正は warn ログのみで継続
//     （単一 rule の不正で全体配信を止めない）
//   - watcher 起動失敗は致命的エラーとして上層に返す（ホットリロード機能なしで継続するか
//     fail fast するかは呼出側 main.rs の判断）

use std::path::{Path, PathBuf};
use std::sync::Arc;

use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc;

use crate::registry::{RegisterInput, RegistryError, RuleRegistry};

/// ホットリローダのエラー型。
#[derive(Debug)]
pub enum LoaderError {
    /// I/O エラー（ディレクトリ読込 / ファイル読込）。
    Io(std::io::Error),
    /// notify watcher の起動 / 動作エラー。
    Watcher(notify::Error),
    /// registry への登録時のエラー（個別ファイルでは warn ログ、致命の場合のみ propagate）。
    Registry(RegistryError),
}

impl std::fmt::Display for LoaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoaderError::Io(e) => write!(f, "io: {}", e),
            LoaderError::Watcher(e) => write!(f, "watcher: {}", e),
            LoaderError::Registry(e) => write!(f, "registry: {}", e),
        }
    }
}

impl std::error::Error for LoaderError {}

impl From<std::io::Error> for LoaderError {
    fn from(e: std::io::Error) -> Self {
        LoaderError::Io(e)
    }
}

impl From<notify::Error> for LoaderError {
    fn from(e: notify::Error) -> Self {
        LoaderError::Watcher(e)
    }
}

impl From<RegistryError> for LoaderError {
    fn from(e: RegistryError) -> Self {
        LoaderError::Registry(e)
    }
}

/// `path_to_rule_id` はファイルパスから rule_id を導出する。
/// 拡張子を取り除いたファイル名そのものを rule_id とする。
/// 例: `/etc/k1s0/decisions/payment-approval.json` → `"payment-approval"`
pub fn path_to_rule_id(path: &Path) -> Option<String> {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
}

/// `is_jdm_file` は対象パスが JDM ファイル候補か判定する。
/// 拡張子 `.json` の通常ファイルのみを対象とする（hidden / dotfile / dir は除外）。
pub fn is_jdm_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    match path.extension().and_then(|e| e.to_str()) {
        Some("json") => {
            // hidden file ("..*.json" 等の k8s ConfigMap atomic update 中間ファイル) を除外。
            path.file_name()
                .and_then(|f| f.to_str())
                .map(|s| !s.starts_with('.') && !s.starts_with(".."))
                .unwrap_or(false)
        }
        _ => false,
    }
}

/// `load_one` は 1 つの JDM ファイルを読み込み、registry に register する。
///
/// `system_tenant_id` は ConfigMap 由来 rule を所属させる「システム名前空間」。
/// 環境変数 `K1S0_DECISION_SYSTEM_TENANT` 由来で、production では各テナントから
/// 直接 evaluate しても見えない（テナント越境防止 NFR-E-AC-003）。空文字も許容
/// するが、その場合は claims.tenant_id="" の dev 経路だけが evaluate できる。
///
/// 失敗時はエラーを上層に返すが、呼出側 (load_initial / on_change_event) は
/// 個別ファイルのエラーを warn ログ化して継続する想定。
pub fn load_one(
    registry: &RuleRegistry,
    path: &Path,
    system_tenant_id: &str,
    registered_by: &str,
) -> Result<String, LoaderError> {
    let rule_id = path_to_rule_id(path)
        .ok_or_else(|| LoaderError::Io(std::io::Error::other("invalid file name")))?;
    let bytes = std::fs::read(path)?;
    let outcome = registry.register(RegisterInput {
        tenant_id: system_tenant_id.to_string(),
        rule_id: rule_id.clone(),
        jdm_document: bytes,
        // commit_hash は ConfigMap 由来では K8s 側が知っている範囲外なので、
        // ファイルの mtime を文字列化して観測 ID として使う（運用簡素化）。
        commit_hash: path
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| format!("mtime-{}", d.as_secs()))
            .unwrap_or_default(),
        registered_by: registered_by.to_string(),
        registered_at_ms: 0,
    })?;
    Ok(outcome.rule_version)
}

/// `load_initial` は起動時のディレクトリ走査で全 JDM を registry に登録する。
/// 戻り値は (登録に成功した件数, 失敗ファイルのリスト)。
pub fn load_initial(
    registry: &RuleRegistry,
    dir: &Path,
    system_tenant_id: &str,
    registered_by: &str,
) -> Result<(usize, Vec<(PathBuf, String)>), LoaderError> {
    let mut ok = 0usize;
    let mut errors: Vec<(PathBuf, String)> = Vec::new();
    if !dir.exists() {
        // ディレクトリ未存在は「ホットリロード対象なし」として success 扱い（dev 経路）。
        return Ok((0, errors));
    }
    for entry in std::fs::read_dir(dir)? {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                errors.push((dir.to_path_buf(), e.to_string()));
                continue;
            }
        };
        let path = entry.path();
        if !is_jdm_file(&path) {
            continue;
        }
        match load_one(registry, &path, system_tenant_id, registered_by) {
            Ok(_) => ok += 1,
            Err(e) => errors.push((path, e.to_string())),
        }
    }
    Ok((ok, errors))
}

/// `spawn_watcher` はディレクトリ監視 watcher を起動する。
/// 監視は別 tokio task として永続化され、呼出側は `JoinHandle` を保持して abort できる。
///
/// 動作:
///   - notify::RecommendedWatcher を NonRecursive で起動
///   - イベント受信時に該当 .json ファイルを再読込 → registry.register
///   - JSON 不正 / I/O エラーは warn ログのみで処理を継続
pub fn spawn_watcher(
    registry: Arc<RuleRegistry>,
    dir: PathBuf,
    system_tenant_id: String,
    registered_by: String,
) -> Result<tokio::task::JoinHandle<()>, LoaderError> {
    // notify 4 の watcher は同期 channel に Event を送る。tokio 側で受け取るため
    // tokio::sync::mpsc の Sender を std::sync::mpsc とブリッジする。
    let (tx, mut rx) = mpsc::unbounded_channel::<notify::Result<Event>>();
    let mut watcher: RecommendedWatcher = Watcher::new(
        move |res: notify::Result<Event>| {
            // 同期 callback。tokio channel に送る（fail-soft: send 失敗は drop）。
            let _ = tx.send(res);
        },
        Config::default(),
    )?;
    watcher.watch(&dir, RecursiveMode::NonRecursive)?;
    // tokio task 内で Watcher を保持し、Drop で監視を停止させる。
    let handle = tokio::spawn(async move {
        // Watcher は move 済（drop で監視停止）。
        let _watcher = watcher;
        while let Some(ev_res) = rx.recv().await {
            match ev_res {
                Ok(event) => {
                    handle_event(&registry, &event, &system_tenant_id, &registered_by).await
                }
                Err(e) => eprintln!("tier1/decision: watcher error: {}", e),
            }
        }
    });
    Ok(handle)
}

/// `handle_event` は notify Event を処理し、対象ファイルを registry に再登録する。
async fn handle_event(
    registry: &RuleRegistry,
    event: &Event,
    system_tenant_id: &str,
    registered_by: &str,
) {
    // 関心のあるイベント種別のみ拾う（Create / Modify / Remove）。
    // ConfigMap 更新は内部的に「symlink swap → 旧 dir 削除 → 新 dir 出現」の
    // 連続イベントとして観測されるため、Create / Modify を等価に扱う。
    match event.kind {
        EventKind::Create(_) | EventKind::Modify(_) => {}
        _ => return,
    }
    for path in &event.paths {
        if !is_jdm_file(path) {
            continue;
        }
        match load_one(registry, path, system_tenant_id, registered_by) {
            Ok(version) => {
                if let Some(rule_id) = path_to_rule_id(path) {
                    eprintln!(
                        "tier1/decision: hot-reloaded rule_id={} new_version={} path={}",
                        rule_id,
                        version,
                        path.display()
                    );
                }
            }
            Err(e) => {
                eprintln!(
                    "tier1/decision: hot-reload failed path={} error={}",
                    path.display(),
                    e
                );
            }
        }
    }
}

#[cfg(test)]
#[path = "loader_tests.rs"]
mod loader_tests;
