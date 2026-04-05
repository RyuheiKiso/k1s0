use async_trait::async_trait;
use std::io::Write;

/// CosignVerifier はコンテナイメージやバイナリの Cosign 署名検証を行うトレイト。
/// STATIC-CRITICAL-002: サプライチェーン攻撃対策として、バージョン登録時に署名を検証する。
#[async_trait]
pub trait CosignVerifier: Send + Sync {
    /// アーティファクト識別子（checksum_sha256 等）と署名を受け取り、署名の正当性を検証する。
    ///
    /// # Arguments
    /// - `artifact`: 検証対象のアーティファクト識別子（例: SHA256 チェックサム）
    /// - `signature`: Cosign 署名文字列（base64 エンコード）
    ///
    /// # Returns
    /// - `Ok(true)`: 署名が有効
    /// - `Ok(false)`: 署名が無効
    /// - `Err(...)`: 検証中にエラーが発生
    async fn verify(&self, artifact: &str, signature: &str) -> anyhow::Result<bool>;
}

/// StubCosignVerifier は開発・テスト用のスタブ実装。常に true を返す。
/// 本番環境では使用しないこと。
pub struct StubCosignVerifier;

#[async_trait]
impl CosignVerifier for StubCosignVerifier {
    async fn verify(&self, _artifact: &str, _signature: &str) -> anyhow::Result<bool> {
        Ok(true)
    }
}

/// SubprocessCosignVerifier は cosign CLI を通じてアーティファクト署名を検証する実装。
/// 事前に cosign バイナリが PATH 上にインストールされている必要がある。
///
/// 検証コマンド例:
/// ```
/// cosign verify-blob \
///   --key <public_key_path> \
///   --signature <signature_file> \
///   <artifact_digest>
/// ```
pub struct SubprocessCosignVerifier {
    /// Cosign 署名検証に使用する公開鍵ファイルのパス
    pub public_key_path: String,
}

impl SubprocessCosignVerifier {
    pub fn new(public_key_path: String) -> Self {
        Self { public_key_path }
    }
}

#[async_trait]
impl CosignVerifier for SubprocessCosignVerifier {
    async fn verify(&self, artifact: &str, signature: &str) -> anyhow::Result<bool> {
        // tempfile::NamedTempFile でアトミックに一時ファイルを作成し、署名を書き込む。
        // NamedTempFile はスコープ終了時に Drop で自動削除されるため、手動 remove_file が不要になる。
        let sig_file = create_signature_tempfile(signature)?;

        // cosign verify-blob に一時ファイルのパスを渡す
        let sig_path = sig_file
            .path()
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("一時ファイルのパスが UTF-8 ではありません"))?;

        let output = tokio::process::Command::new("cosign")
            .args([
                "verify-blob",
                "--key",
                &self.public_key_path,
                "--signature",
                sig_path,
                artifact,
            ])
            .output()
            .await
            .map_err(|e| anyhow::anyhow!("cosign コマンドの実行に失敗しました: {}", e))?;

        // sig_file はここで Drop され、一時ファイルが確実に削除される

        if output.status.success() {
            Ok(true)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!(
                artifact = artifact,
                stderr = %stderr,
                "Cosign 署名検証に失敗しました"
            );
            Ok(false)
        }
    }
}

/// 署名文字列を安全な一時ファイルに書き込む。
/// tempfile::Builder を使用してアトミックにファイルを作成し、TOCTOU 脆弱性を防止する。
/// NamedTempFile は Drop 時に自動的に削除されるため、手動クリーンアップが不要になる。
fn create_signature_tempfile(signature: &str) -> anyhow::Result<tempfile::NamedTempFile> {
    let mut file = tempfile::Builder::new()
        .prefix("cosign-sig-")
        .suffix(".sig")
        .tempfile()
        .map_err(|e| anyhow::anyhow!("一時ファイルの作成に失敗しました: {}", e))?;
    file.write_all(signature.as_bytes())
        .map_err(|e| anyhow::anyhow!("署名の一時ファイル書き込みに失敗しました: {}", e))?;
    Ok(file)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn stub_verifier_always_returns_true() {
        let verifier = StubCosignVerifier;
        let result = verifier.verify("sha256:abc123", "sig_base64_data").await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn stub_verifier_returns_true_for_empty_inputs() {
        let verifier = StubCosignVerifier;
        let result = verifier.verify("", "").await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}
