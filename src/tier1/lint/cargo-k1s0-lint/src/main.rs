// main.rs — k1s0 tier1 cargo-k1s0-lint: Rust 独自 lint binary
// tier1/CLAUDE.md §lint ツール「cargo-deny [bans] + clippy + cargo public-api」を補完する。
// banned API チェック・facade 経由強制チェック・依存方向チェックを実装する。
// 違反なし: exit 0 / 違反あり: exit 1 を返す。

// std::env: コマンドライン引数の取得に使用する
use std::env;
// std::fs: ファイル読み込みに使用する
use std::fs;
// std::path::Path: ファイルパスの操作に使用する
use std::path::Path;
// std::process: exit コードの設定に使用する
use std::process;

// BANNED_APIS: src/CLAUDE.md §全軸共通コーディング禁止事項に基づく Rust 禁止 API リスト
// wall-clock TTL 計算禁止・production/development 区別禁止を enforce する
const BANNED_APIS: &[&str] = &[
    // wall-clock TTL 計算禁止: SystemTime::now() の TTL 計算利用を禁止する
    "SystemTime::now()",
    // wall-clock TTL 計算禁止: Instant::now() の TTL deadline 計算を禁止する（相対時間計算は OK）
    "Utc::now()",
    // production/development 区別禁止: cfg(debug_assertions) でセキュリティ弱体化禁止
    "#[cfg(debug_assertions)]",
    // production/development 区別禁止: cfg(test) でセキュリティ弱体化禁止
    // NOTE: テスト自体の #[cfg(test)] は許可するが、セキュリティロジック分岐は禁止
    "cfg(feature = \"unsafe_dev\")",
    // 生 key bytes を公開 API に露出禁止: pub fn を通じた [u8] / Vec<u8> の key bytes 返却禁止
    // NOTE: 実際の enforce は公開シグネチャ検査で行う（ここはシンボリックなエントリ）
];

// FACADE_REQUIRED_PATTERNS: tier2/tier3 から OSS crate を直接 import することを禁止するパターン
// src/CLAUDE.md §依存方向の制約「tier2 → tier1 OSS crate の直接 public API 露出禁止（facade 経由必須）」
const FACADE_REQUIRED_PATTERNS: &[&str] = &[
    // Npgsql 直接 import 禁止（IDbClient facade 経由必須）
    "use Npgsql::",
    // Confluent.Kafka 直接 import 禁止（IMessagingProducer facade 経由必須）
    "use confluent_kafka::",
    // Temporalio 直接 import 禁止（IWorkflowClient facade 経由必須）
    "use temporalio::",
];

// LintViolation: lint 違反を表す構造体
struct LintViolation {
    // ファイルパス: 違反が検出されたファイル
    file: String,
    // 行番号: 違反が検出された行番号（1-based）
    line: usize,
    // 違反メッセージ: 違反内容の説明
    message: String,
}

// check_banned_apis: 指定ファイルの禁止 API チェックを実行する
// path: チェック対象ファイルのパス
// violations: 違反を追記するベクタ（可変参照）
fn check_banned_apis(path: &Path, violations: &mut Vec<LintViolation>) {
    // ファイルを読み込む（読み込み失敗時はスキップする）
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        // ファイル読み込み失敗時はスキップする（バイナリファイル等）
        Err(_) => return,
    };
    // 各行を走査して禁止 API パターンを検出する
    for (line_idx, line) in content.lines().enumerate() {
        // 禁止 API リストの各エントリと照合する
        for banned in BANNED_APIS {
            // 行に禁止パターンが含まれる場合は違反として記録する
            if line.contains(banned) {
                // 違反を追加する（行番号は 1-based にする）
                violations.push(LintViolation {
                    file: path.display().to_string(),
                    line: line_idx + 1,
                    message: format!("banned API detected: `{banned}` — src/CLAUDE.md §wall-clock TTL 禁止 / production-development 区別禁止"),
                });
            }
        }
    }
}

// check_facade_required: 指定ファイルの facade 経由強制チェックを実行する
// path: チェック対象ファイルのパス
// violations: 違反を追記するベクタ（可変参照）
fn check_facade_required(path: &Path, violations: &mut Vec<LintViolation>) {
    // ファイルを読み込む（読み込み失敗時はスキップする）
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        // ファイル読み込み失敗時はスキップする
        Err(_) => return,
    };
    // 各行を走査して facade 経由強制パターンを検出する
    for (line_idx, line) in content.lines().enumerate() {
        // facade 必須パターンリストの各エントリと照合する
        for pattern in FACADE_REQUIRED_PATTERNS {
            // 行に直接 import パターンが含まれる場合は違反として記録する
            if line.contains(pattern) {
                // 違反を追加する（行番号は 1-based にする）
                violations.push(LintViolation {
                    file: path.display().to_string(),
                    line: line_idx + 1,
                    message: format!("direct OSS import detected: `{pattern}` — facade 経由でアクセスすること（tier1/CLAUDE.md §公開 API の型制約）"),
                });
            }
        }
    }
}

// walk_rs_files: ディレクトリを再帰的に走査して .rs ファイルを収集する
// dir: 走査開始ディレクトリ
// files: 収集したファイルパスを追記するベクタ（可変参照）
fn walk_rs_files(dir: &Path, files: &mut Vec<std::path::PathBuf>) {
    // ディレクトリの読み込みに失敗した場合はスキップする
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        // 読み込み失敗時はスキップする
        Err(_) => return,
    };
    // 各エントリを処理する
    for entry in entries.flatten() {
        // エントリのパスを取得する
        let path = entry.path();
        // ディレクトリの場合は再帰的に走査する
        if path.is_dir() {
            // target/ ディレクトリはビルド成果物なのでスキップする
            if path.file_name().map_or(false, |n| n == "target") {
                continue;
            }
            // 再帰的に走査する
            walk_rs_files(&path, files);
        } else if path.extension().map_or(false, |e| e == "rs") {
            // .rs ファイルの場合は収集リストに追加する
            files.push(path);
        }
    }
}

// main: cargo k1s0-lint のエントリポイント
// cargo k1s0-lint [target_dir] の形式で呼び出される
fn main() {
    // コマンドライン引数を取得する
    let args: Vec<String> = env::args().collect();
    // cargo サブコマンドとして呼び出された場合は最初の引数（"k1s0-lint"）をスキップする
    // 第 2 引数以降をターゲットディレクトリとして使用する
    let target_dir = if args.len() >= 3 && args[1] == "k1s0-lint" {
        // cargo k1s0-lint <dir> として呼び出された場合
        args[2].clone()
    } else if args.len() >= 2 && args[1] != "k1s0-lint" {
        // 直接実行で <dir> を指定した場合
        args[1].clone()
    } else {
        // 引数なしの場合はカレントディレクトリを走査する
        ".".to_string()
    };
    // ターゲットディレクトリを Path に変換する
    let target_path = Path::new(&target_dir);
    // ターゲットディレクトリが存在しない場合はエラーを表示して終了する
    if !target_path.exists() {
        eprintln!("error: target directory `{}` does not exist", target_dir);
        // 設定エラーは exit 2 で返す
        process::exit(2);
    }
    // .rs ファイルを収集する
    let mut rs_files = Vec::new();
    // ディレクトリを再帰的に走査する
    walk_rs_files(target_path, &mut rs_files);
    // 違反リストを初期化する
    let mut violations: Vec<LintViolation> = Vec::new();
    // 各 .rs ファイルに対してチェックを実行する
    for file in &rs_files {
        // 禁止 API チェックを実行する
        check_banned_apis(file, &mut violations);
        // facade 経由強制チェックを実行する
        check_facade_required(file, &mut violations);
    }
    // 違反がない場合は成功メッセージを表示して exit 0 で終了する
    if violations.is_empty() {
        println!("cargo-k1s0-lint: {} files checked, no violations found.", rs_files.len());
        // 違反なし: exit 0
        process::exit(0);
    }
    // 違反がある場合は各違反を stderr に出力する
    eprintln!("cargo-k1s0-lint: {} violation(s) found:", violations.len());
    // 各違反の詳細を出力する
    for v in &violations {
        eprintln!("  {}:{}: {}", v.file, v.line, v.message);
    }
    // 違反あり: exit 1
    process::exit(1);
}
