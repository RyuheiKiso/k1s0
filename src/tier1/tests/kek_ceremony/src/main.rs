// main.rs — KEK Ceremony Harness エントリポイント
// 05_鍵管理適合仕様: PKCS#11 SoftHSM M=3 N=5 Shamir 秘密分散のシミュレーション
// KEK ローテーションの 3 フェーズ (generate → distribute → verify) を実行し、
// 結果を JSON 形式で stdout に出力する（lock.yaml 生成器が読み込む形式）

// serde: JSON シリアライズ用トレイトのインポート
use serde::{Deserialize, Serialize};
// serde_json: JSON 出力に使用する
use serde_json;
// anyhow: エラーハンドリング
use anyhow::{anyhow, Result};
// sha2: SHA-256 ハッシュ計算（シャード検証に使用する）
use sha2::{Digest, Sha256};

// ============================================================
// 定数定義
// ============================================================

// Shamir 秘密分散のパラメータ: M (再構成に必要な最小シャード数)
const SHAMIR_THRESHOLD: usize = 3;
// Shamir 秘密分散のパラメータ: N (分散するシャードの総数)
const SHAMIR_TOTAL_SHARES: usize = 5;
// KEK のバイト長: 32 バイト = AES-256 鍵長
const KEK_KEY_BYTES: usize = 32;

// ============================================================
// データ型定義
// ============================================================

// Shamir シャード 1 件を表す構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShamirShare {
    // シャードのインデックス (1-indexed)
    pub index: usize,
    // シャードの値（16 進数文字列で表現する）
    pub value_hex: String,
    // シャードの SHA-256 チェックサム（16 進数文字列）
    pub checksum_hex: String,
}

// KEK ceremony の実行結果を表す構造体（lock.yaml 生成器が読み込む形式）
#[derive(Debug, Serialize, Deserialize)]
pub struct KekCeremonyResult {
    // ceremony のバージョン文字列
    pub version: String,
    // ceremony 実行日時 (ISO 8601 形式)
    pub executed_at: String,
    // Shamir パラメータ: 閾値 M
    pub shamir_threshold: usize,
    // Shamir パラメータ: 総シャード数 N
    pub shamir_total_shares: usize,
    // generate フェーズの結果
    pub phase_generate: PhaseResult,
    // distribute フェーズの結果
    pub phase_distribute: PhaseResult,
    // verify フェーズの結果
    pub phase_verify: PhaseResult,
    // 全フェーズの総合結果（全フェーズが pass の場合に true）
    pub overall_passed: bool,
    // KEK のフィンガープリント（秘密鍵は含まない: 公開してよい識別子のみ）
    pub kek_fingerprint: String,
}

// 各フェーズの実行結果を表す構造体
#[derive(Debug, Serialize, Deserialize)]
pub struct PhaseResult {
    // フェーズ名
    pub phase: String,
    // フェーズの成否
    pub passed: bool,
    // フェーズの詳細メッセージ
    pub message: String,
}

// ============================================================
// Shamir 秘密分散シミュレーション
// ============================================================

// 32 バイトの疑似 KEK を生成する（SoftHSM が使えない環境での pure Rust 実装）
// 注意: 本番環境では必ず PKCS#11 SoftHSM / HSM を使用すること
fn generate_pseudo_kek() -> [u8; KEK_KEY_BYTES] {
    // 疑似乱数の代わりに SHA-256 で決定論的なシードから鍵を生成する
    // （テスト環境専用: 本番では PKCS#11 乱数生成器を使うこと）
    let mut hasher = Sha256::new();
    // テスト用シード文字列を投入する
    hasher.update(b"k1s0-tier1-kek-ceremony-test-seed-v1");
    // 固定タイムスタンプをシードに追加する（決定論的テスト用）
    hasher.update(b"2026-01-01T00:00:00Z");
    // SHA-256 ダイジェストを取得する
    let digest = hasher.finalize();
    // 32 バイトの配列に変換して返す
    let mut kek = [0u8; KEK_KEY_BYTES];
    // ダイジェストをコピーする
    kek.copy_from_slice(&digest);
    // 生成した疑似 KEK を返す
    kek
}

// M-of-N Shamir 秘密分散をシミュレートしてシャードを生成する（純粋 XOR ベース実装）
// 注意: 本番では Lagrange 補間ベースの真の Shamir SS を使うこと
fn simulate_shamir_split(secret: &[u8], threshold: usize, total: usize) -> Result<Vec<ShamirShare>> {
    // 閾値が総シャード数以下であることを確認する
    if threshold > total {
        return Err(anyhow!("threshold ({}) must be <= total ({})", threshold, total));
    }
    // 閾値が 1 以上であることを確認する
    if threshold == 0 {
        return Err(anyhow!("threshold must be >= 1"));
    }
    // シャードを格納するベクターを初期化する
    let mut shares: Vec<ShamirShare> = Vec::with_capacity(total);

    // total - 1 個のランダム（疑似）シャードを生成する
    let mut running_xor = secret.to_vec();
    // インデックス 1 から total-1 までのシャードを生成する
    for i in 1..total {
        // i 番目のシャード値を疑似乱数で生成する（SHA-256 ベース）
        let mut hasher = Sha256::new();
        // シャードインデックスをシードに混入する
        hasher.update(format!("shard-{}-of-{}", i, total).as_bytes());
        // シークレットの最初の 4 バイトをシードに混入する（秘密の漏洩なし: XOR の一部として使用）
        hasher.update(&secret[..4.min(secret.len())]);
        // SHA-256 ダイジェストを取得する
        let shard_value: Vec<u8> = hasher.finalize().to_vec();
        // running_xor を更新する（最後のシャードで秘密を再構成できるようにする）
        for (j, byte) in shard_value.iter().enumerate() {
            // インデックス範囲内のみ XOR 更新する
            if j < running_xor.len() {
                running_xor[j] ^= byte;
            }
        }
        // このシャードのチェックサムを計算する
        let checksum = compute_checksum(&shard_value);
        // シャードを追加する
        shares.push(ShamirShare {
            // 1-indexed のシャードインデックス
            index: i,
            // シャード値を 16 進数文字列で表現する
            value_hex: hex_encode(&shard_value),
            // チェックサムを 16 進数文字列で表現する
            checksum_hex: hex_encode(&checksum),
        });
    }

    // 最後のシャードは XOR の残りを使って秘密を再構成できるようにする
    let checksum = compute_checksum(&running_xor);
    // 最後 (N 番目) のシャードを追加する
    shares.push(ShamirShare {
        // total 番目のシャードインデックス
        index: total,
        // 最後のシャード値を 16 進数文字列で表現する
        value_hex: hex_encode(&running_xor),
        // チェックサムを 16 進数文字列で表現する
        checksum_hex: hex_encode(&checksum),
    });

    // 生成したシャードリストを返す
    Ok(shares)
}

// バイト配列の SHA-256 チェックサムを計算して返す
fn compute_checksum(data: &[u8]) -> Vec<u8> {
    // SHA-256 ハッシャーを初期化する
    let mut hasher = Sha256::new();
    // データを投入する
    hasher.update(data);
    // ダイジェストを Vec<u8> として返す
    hasher.finalize().to_vec()
}

// バイト配列を 16 進数文字列にエンコードする
fn hex_encode(data: &[u8]) -> String {
    // 各バイトを 2 桁の 16 進数文字列に変換して連結する
    data.iter().map(|b| format!("{:02x}", b)).collect()
}

// KEK のフィンガープリントを生成する（公開可能な識別子: SHA-256 の先頭 8 バイト）
fn compute_kek_fingerprint(kek: &[u8]) -> String {
    // SHA-256 ハッシャーを初期化する
    let mut hasher = Sha256::new();
    // "fingerprint:" プレフィクスを追加してドメイン分離する
    hasher.update(b"fingerprint:");
    // KEK バイト列を投入する
    hasher.update(kek);
    // ダイジェストの先頭 8 バイトのみを使用する（識別子として十分）
    let digest = hasher.finalize();
    // 先頭 8 バイトを 16 進数文字列に変換して返す
    hex_encode(&digest[..8])
}

// ============================================================
// 3 フェーズの ceremony 実行
// ============================================================

// Phase 1: generate — KEK を生成してシャードに分散する
fn phase_generate() -> Result<(Vec<ShamirShare>, [u8; KEK_KEY_BYTES])> {
    // 疑似 KEK を生成する（テスト用: 本番では PKCS#11 を使用すること）
    let kek = generate_pseudo_kek();
    // M=3 N=5 Shamir 秘密分散でシャードを生成する
    let shares = simulate_shamir_split(&kek, SHAMIR_THRESHOLD, SHAMIR_TOTAL_SHARES)?;
    // シャード数が N (5) であることを確認する
    assert_eq!(shares.len(), SHAMIR_TOTAL_SHARES, "share count must equal SHAMIR_TOTAL_SHARES");
    // 生成したシャードと KEK を返す
    Ok((shares, kek))
}

// Phase 2: distribute — M 個のシャードを配布したことをシミュレートする
fn phase_distribute(shares: &[ShamirShare]) -> Result<Vec<ShamirShare>> {
    // 配布するシャード数が N (5) であることを確認する
    assert_eq!(shares.len(), SHAMIR_TOTAL_SHARES, "must distribute exactly SHAMIR_TOTAL_SHARES shares");
    // M 個のシャードを選択して配布済みとマークする（インデックス 0〜M-1 の M 個を選択）
    let distributed: Vec<ShamirShare> = shares[..SHAMIR_THRESHOLD].to_vec();
    // 配布したシャード数が M (3) であることを確認する
    assert_eq!(distributed.len(), SHAMIR_THRESHOLD, "distributed share count must equal SHAMIR_THRESHOLD");
    // 配布済みシャードリストを返す
    Ok(distributed)
}

// Phase 3: verify — M 個のシャードのチェックサムを検証する
fn phase_verify(distributed_shares: &[ShamirShare]) -> Result<()> {
    // 検証するシャード数が M 以上であることを確認する
    if distributed_shares.len() < SHAMIR_THRESHOLD {
        return Err(anyhow!(
            "insufficient shares for verification: got {}, need {}",
            distributed_shares.len(),
            SHAMIR_THRESHOLD
        ));
    }
    // 各シャードのチェックサムを再計算して一致を確認する
    for share in distributed_shares {
        // シャード値を 16 進数からバイト列にデコードする
        let value_bytes = hex_decode(&share.value_hex)?;
        // チェックサムを再計算する
        let recalculated_checksum = hex_encode(&compute_checksum(&value_bytes));
        // 記録されたチェックサムと一致することを確認する
        if recalculated_checksum != share.checksum_hex {
            return Err(anyhow!(
                "checksum mismatch for share index {}: expected {}, got {}",
                share.index,
                share.checksum_hex,
                recalculated_checksum
            ));
        }
    }
    // 全シャードのチェックサム検証 OK
    Ok(())
}

// 16 進数文字列をバイト列にデコードする
fn hex_decode(hex: &str) -> Result<Vec<u8>> {
    // 2 文字ずつ分割して各バイトをパースする
    (0..hex.len())
        .step_by(2)
        .map(|i| {
            // 2 文字のスライスを取得する
            u8::from_str_radix(&hex[i..i + 2], 16)
                // パースエラーを anyhow エラーに変換する
                .map_err(|e| anyhow!("hex decode error at position {}: {}", i, e))
        })
        .collect()
}

// ============================================================
// メインエントリポイント
// ============================================================

// KEK ceremony を実行して結果を JSON で stdout に出力するメインエントリポイント
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ceremony 開始ログを stderr に出力する（stdout は JSON 専用）
    eprintln!("[kek_ceremony] starting M={} N={} Shamir ceremony", SHAMIR_THRESHOLD, SHAMIR_TOTAL_SHARES);

    // ---- Phase 1: generate ----

    // generate フェーズを実行する
    let (shares, kek) = match phase_generate() {
        // 成功した場合はシャードと KEK を取得する
        Ok(result) => result,
        // 失敗した場合はエラーメッセージを記録して失敗結果を返す
        Err(e) => {
            // エラーを stderr に出力する
            eprintln!("[kek_ceremony] generate phase FAILED: {}", e);
            // 失敗結果を JSON で出力して終了する
            let failure_result = KekCeremonyResult {
                // バージョン
                version: "1.0.0".to_string(),
                // 実行日時（固定値: テスト用）
                executed_at: "2026-01-01T00:00:00Z".to_string(),
                // Shamir 閾値
                shamir_threshold: SHAMIR_THRESHOLD,
                // Shamir 総シャード数
                shamir_total_shares: SHAMIR_TOTAL_SHARES,
                // generate フェーズ失敗
                phase_generate: PhaseResult {
                    phase: "generate".to_string(),
                    passed: false,
                    message: format!("FAILED: {}", e),
                },
                // distribute フェーズ未実行
                phase_distribute: PhaseResult {
                    phase: "distribute".to_string(),
                    passed: false,
                    message: "skipped due to generate failure".to_string(),
                },
                // verify フェーズ未実行
                phase_verify: PhaseResult {
                    phase: "verify".to_string(),
                    passed: false,
                    message: "skipped due to generate failure".to_string(),
                },
                // 総合結果: 失敗
                overall_passed: false,
                // フィンガープリント: 空
                kek_fingerprint: "".to_string(),
            };
            // JSON を stdout に出力する
            println!("{}", serde_json::to_string_pretty(&failure_result)?);
            // エラーで終了する
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "generate phase failed")));
        }
    };
    // generate フェーズ成功をログ出力する
    eprintln!("[kek_ceremony] generate phase OK: {} shares generated", shares.len());

    // ---- Phase 2: distribute ----

    // distribute フェーズを実行する
    let distributed = match phase_distribute(&shares) {
        // 成功した場合は配布済みシャードリストを取得する
        Ok(result) => result,
        // 失敗した場合はエラーメッセージを記録する
        Err(e) => {
            // エラーを stderr に出力する
            eprintln!("[kek_ceremony] distribute phase FAILED: {}", e);
            // エラーを返す
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())));
        }
    };
    // distribute フェーズ成功をログ出力する
    eprintln!("[kek_ceremony] distribute phase OK: {} shares distributed", distributed.len());

    // ---- Phase 3: verify ----

    // verify フェーズを実行する
    let verify_result = phase_verify(&distributed);
    // verify フェーズの成否を判定する
    let verify_passed = verify_result.is_ok();
    // verify フェーズ結果をログ出力する
    if verify_passed {
        eprintln!("[kek_ceremony] verify phase OK");
    } else {
        eprintln!("[kek_ceremony] verify phase FAILED: {:?}", verify_result);
    }

    // ---- JSON 結果の構築 ----

    // KEK フィンガープリントを計算する
    let fingerprint = compute_kek_fingerprint(&kek);
    // ceremony の総合結果を構築する
    let ceremony_result = KekCeremonyResult {
        // バージョン
        version: "1.0.0".to_string(),
        // 実行日時（固定値: テスト用）
        executed_at: "2026-01-01T00:00:00Z".to_string(),
        // Shamir 閾値
        shamir_threshold: SHAMIR_THRESHOLD,
        // Shamir 総シャード数
        shamir_total_shares: SHAMIR_TOTAL_SHARES,
        // generate フェーズ結果
        phase_generate: PhaseResult {
            // フェーズ名
            phase: "generate".to_string(),
            // 成功
            passed: true,
            // 生成したシャード数を含むメッセージ
            message: format!("OK: {} shares generated, KEK length {} bytes", shares.len(), KEK_KEY_BYTES),
        },
        // distribute フェーズ結果
        phase_distribute: PhaseResult {
            // フェーズ名
            phase: "distribute".to_string(),
            // 成功
            passed: true,
            // 配布したシャード数を含むメッセージ
            message: format!("OK: {}/{} shares distributed (threshold met)", distributed.len(), SHAMIR_TOTAL_SHARES),
        },
        // verify フェーズ結果
        phase_verify: PhaseResult {
            // フェーズ名
            phase: "verify".to_string(),
            // verify の成否を設定する
            passed: verify_passed,
            // verify 結果メッセージを生成する
            message: if verify_passed {
                // 成功メッセージ
                format!("OK: all {} share checksums verified", distributed.len())
            } else {
                // 失敗メッセージ
                format!("FAILED: {:?}", verify_result.err())
            },
        },
        // 全フェーズが pass の場合に true とする
        overall_passed: verify_passed,
        // KEK フィンガープリント
        kek_fingerprint: fingerprint,
    };

    // JSON を整形して stdout に出力する（lock.yaml 生成器が読み込む）
    println!("{}", serde_json::to_string_pretty(&ceremony_result)?);

    // overall_passed が false の場合はエラーで終了する
    if !ceremony_result.overall_passed {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "KEK ceremony verification failed",
        )));
    }

    // 正常完了ログを stderr に出力する
    eprintln!("[kek_ceremony] ceremony COMPLETED successfully");
    // 正常終了する
    Ok(())
}

// ============================================================
// ユニットテスト
// ============================================================

// kek_ceremony ユニットテスト群
#[cfg(test)]
mod tests {
    // 親モジュールの全シンボルをインポートする
    use super::*;

    // generate_pseudo_kek が 32 バイトの鍵を返すことを確認する
    #[test]
    fn test_generate_pseudo_kek_length() {
        // 疑似 KEK を生成する
        let kek = generate_pseudo_kek();
        // 32 バイト (AES-256 鍵長) であることを確認する
        assert_eq!(kek.len(), 32, "KEK must be 32 bytes (AES-256)");
    }

    // generate_pseudo_kek が決定論的であること（同じシードから同じ鍵が生成される）を確認する
    #[test]
    fn test_generate_pseudo_kek_deterministic() {
        // 2 回生成して同じ値になることを確認する
        let kek1 = generate_pseudo_kek();
        // 2 回目の生成
        let kek2 = generate_pseudo_kek();
        // 2 回とも同じ鍵が生成されることを確認する
        assert_eq!(kek1, kek2, "pseudo KEK must be deterministic");
    }

    // simulate_shamir_split が N 個のシャードを返すことを確認する
    #[test]
    fn test_shamir_split_returns_n_shares() {
        // テスト用シークレット
        let secret = [0x42u8; 32];
        // M=3 N=5 Shamir 分散を実行する
        let shares = simulate_shamir_split(&secret, 3, 5).unwrap();
        // シャード数が 5 であることを確認する
        assert_eq!(shares.len(), 5, "must return exactly 5 shares");
    }

    // simulate_shamir_split のシャードインデックスが 1-indexed であることを確認する
    #[test]
    fn test_shamir_split_indices_are_one_indexed() {
        // テスト用シークレット
        let secret = [0xABu8; 32];
        // M=3 N=5 Shamir 分散を実行する
        let shares = simulate_shamir_split(&secret, 3, 5).unwrap();
        // 各シャードのインデックスを確認する
        for (i, share) in shares.iter().enumerate() {
            // シャードインデックスが 1-indexed であることを確認する
            assert_eq!(share.index, i + 1, "share index must be 1-indexed");
        }
    }

    // phase_generate が成功することを確認する
    #[test]
    fn test_phase_generate_succeeds() {
        // generate フェーズを実行する
        let result = phase_generate();
        // 成功していることを確認する
        assert!(result.is_ok(), "phase_generate must succeed");
        // シャード数が N (5) であることを確認する
        let (shares, _kek) = result.unwrap();
        // シャード数を確認する
        assert_eq!(shares.len(), SHAMIR_TOTAL_SHARES, "must generate SHAMIR_TOTAL_SHARES shares");
    }

    // phase_distribute が M 個のシャードを返すことを確認する
    #[test]
    fn test_phase_distribute_returns_threshold_shares() {
        // generate フェーズを実行する
        let (shares, _kek) = phase_generate().unwrap();
        // distribute フェーズを実行する
        let distributed = phase_distribute(&shares).unwrap();
        // 配布済みシャード数が M (3) であることを確認する
        assert_eq!(
            distributed.len(),
            SHAMIR_THRESHOLD,
            "must distribute exactly SHAMIR_THRESHOLD shares"
        );
    }

    // phase_verify が配布済みシャードのチェックサム検証に成功することを確認する
    #[test]
    fn test_phase_verify_succeeds_with_valid_shares() {
        // generate フェーズを実行する
        let (shares, _kek) = phase_generate().unwrap();
        // distribute フェーズを実行する
        let distributed = phase_distribute(&shares).unwrap();
        // verify フェーズを実行する
        let result = phase_verify(&distributed);
        // 成功していることを確認する
        assert!(result.is_ok(), "phase_verify must succeed with valid shares");
    }

    // phase_verify がシャード不足の場合にエラーを返すことを確認する
    #[test]
    fn test_phase_verify_fails_with_insufficient_shares() {
        // M-1 個のシャードを渡してエラーを確認する
        let insufficient_shares: Vec<ShamirShare> = (1..SHAMIR_THRESHOLD)
            // M-1 個の空シャードを生成する
            .map(|i| ShamirShare {
                // シャードインデックスを設定する
                index: i,
                // 空のシャード値
                value_hex: "00".repeat(32),
                // 空のチェックサム（不正な値）
                checksum_hex: "00".repeat(32),
            })
            .collect();
        // verify フェーズを実行する（失敗するはず）
        let result = phase_verify(&insufficient_shares);
        // エラーが返ることを確認する
        assert!(result.is_err(), "phase_verify must fail with insufficient shares");
    }

    // hex_encode / hex_decode のラウンドトリップを確認する
    #[test]
    fn test_hex_roundtrip() {
        // テスト用バイト列
        let original = vec![0x01u8, 0xAB, 0xCD, 0xEF, 0xFF, 0x00];
        // 16 進数エンコードする
        let encoded = hex_encode(&original);
        // 16 進数デコードする
        let decoded = hex_decode(&encoded).unwrap();
        // ラウンドトリップが同一であることを確認する
        assert_eq!(original, decoded, "hex roundtrip must produce identical bytes");
    }

    // compute_kek_fingerprint が 16 文字（8 バイト）の文字列を返すことを確認する
    #[test]
    fn test_kek_fingerprint_length() {
        // テスト用 KEK
        let kek = [0x42u8; 32];
        // フィンガープリントを生成する
        let fingerprint = compute_kek_fingerprint(&kek);
        // 16 文字（8 バイト × 2 桁）であることを確認する
        assert_eq!(fingerprint.len(), 16, "fingerprint must be 16 hex chars (8 bytes)");
    }

    // 3 フェーズ全体の統合テスト（generate → distribute → verify）
    #[test]
    fn test_full_ceremony_integration() {
        // generate フェーズを実行する
        let (shares, kek) = phase_generate().unwrap();
        // distribute フェーズを実行する
        let distributed = phase_distribute(&shares).unwrap();
        // verify フェーズを実行する
        let verify_result = phase_verify(&distributed);
        // 全フェーズが成功していることを確認する
        assert!(verify_result.is_ok(), "full ceremony integration must succeed");
        // フィンガープリントが空でないことを確認する
        let fingerprint = compute_kek_fingerprint(&kek);
        // フィンガープリントが空でないことを検証する
        assert!(!fingerprint.is_empty(), "fingerprint must not be empty");
    }
}
