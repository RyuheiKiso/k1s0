// cosign_verify.rs — コンテナイメージの cosign 署名を検証する admission webhook 前段
// spec 16 §build_provenance: cosign verify + Rekor lookup で provenance を検証する
// image admission の前段として呼ばれ、検証失敗時は admission を deny する
// cosign CLI を subprocess として呼び出すことで実装する（純 Rust sigstore-rs への移行は後続 PR で行う）

// サブプロセス実行のための標準ライブラリモジュールをインポートする
use std::process::{Command, Output};

// cosign 検証の結果を表す列挙型
#[derive(Debug, PartialEq)]
pub enum CosignVerifyResult {
    // 署名が正常に検証された（Rekor log entry も存在する）
    Verified,
    // 署名が見つからない場合（image に署名が付いていない）
    SignatureNotFound,
    // 署名の検証に失敗した場合（公開鍵が一致しない、改ざんされている等）
    VerificationFailed(String),
}

// image の cosign 署名を検証する関数
// image_ref: 検証対象のコンテナイメージ参照（例: ghcr.io/k1s0-io/k1s0:v1.0.0）
// key_path: cosign 公開鍵のファイルパス（PEM 形式 または cosign.pub）
pub fn verify_image_signature(image_ref: &str, key_path: &str) -> CosignVerifyResult {
    // cosign verify コマンドを組み立てる（keyless の場合は --key を省略して --certificate-identity 等を使う）
    let output: Output = match Command::new("cosign")
        // verify サブコマンドを指定する
        .arg("verify")
        // 公開鍵のパスを指定する（cosign.pub または PEM 形式のファイル）
        .arg("--key")
        .arg(key_path)
        // Rekor のエンドポイントを指定する（公開 Rekor transparency log を使用する）
        .arg("--rekor-url")
        .arg("https://rekor.sigstore.dev")
        // 出力を JSON 形式にする（パース容易性を高める）
        .arg("--output=json")
        // 検証対象の image reference を指定する
        .arg(image_ref)
        // stderr を子プロセスの stderr に接続する（エラーメッセージを取得するため）
        .output()
    {
        // 実行成功時は output を使用する
        Ok(o) => o,
        // 実行失敗時（cosign コマンドが見つからない、permission エラー等）はエラーを返す
        Err(e) => return CosignVerifyResult::VerificationFailed(
            // エラーメッセージに cosign コマンド実行失敗を明示する
            format!("cosign コマンドの実行に失敗した: {}", e)
        ),
    };
    // exit code で結果を判定する（0 = 成功、非 0 = 失敗）
    if output.status.success() {
        // exit code 0 の場合は署名検証成功として Verified を返す
        CosignVerifyResult::Verified
    } else {
        // stderr のバイト列を UTF-8 文字列に変換してエラー詳細を取得する
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        // "no signatures found" メッセージを含む場合は SignatureNotFound を返す
        if stderr.contains("no signatures found") || stderr.contains("no matching signatures") {
            // 署名が存在しない場合の結果を返す
            CosignVerifyResult::SignatureNotFound
        } else {
            // その他の検証失敗（鍵不一致、Rekor 接続失敗等）は VerificationFailed を返す
            CosignVerifyResult::VerificationFailed(stderr)
        }
    }
}

// cosign の署名検証テスト（unit test は subprocess なしで実行する）
#[cfg(test)]
mod tests {
    // 親モジュールの全シンボルをインポートする
    use super::*;

    // cosign コマンドが存在しない環境での fallback テスト
    // CI 環境では cosign はインストールされていないことが多いため VerificationFailed を期待する
    #[test]
    fn test_verify_returns_error_when_cosign_not_found_or_key_invalid() {
        // 存在しない公開鍵ファイルパスで検証を試みる
        let result = verify_image_signature("example.com/image:latest", "/nonexistent/cosign.pub");
        // cosign 未インストールまたは鍵ファイルが存在しない場合は VerificationFailed または SignatureNotFound を返す
        assert!(
            matches!(result, CosignVerifyResult::VerificationFailed(_) | CosignVerifyResult::SignatureNotFound),
            "cosign が見つからないか鍵が不正な場合は VerificationFailed または SignatureNotFound を返すべき"
        );
    }

    // CosignVerifyResult の PartialEq テスト（Verified は Verified と等しい）
    #[test]
    fn test_cosign_verify_result_equality() {
        // Verified 同士は等しい
        assert_eq!(CosignVerifyResult::Verified, CosignVerifyResult::Verified);
        // SignatureNotFound 同士は等しい
        assert_eq!(CosignVerifyResult::SignatureNotFound, CosignVerifyResult::SignatureNotFound);
    }
}
