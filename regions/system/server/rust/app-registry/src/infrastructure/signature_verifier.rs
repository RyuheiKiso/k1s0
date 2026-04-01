use async_trait::async_trait;

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
        // 一時ファイルに署名を書き込んで cosign verify-blob を呼び出す
        let sig_file = tempfile_for_signature(signature).await?;

        let output = tokio::process::Command::new("cosign")
            .args([
                "verify-blob",
                "--key",
                &self.public_key_path,
                "--signature",
                &sig_file,
                artifact,
            ])
            .output()
            .await
            .map_err(|e| anyhow::anyhow!("cosign コマンドの実行に失敗しました: {}", e))?;

        // 一時ファイルを削除する（エラーは無視）
        let _ = tokio::fs::remove_file(&sig_file).await;

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

/// 署名文字列を一時ファイルに書き込み、パスを返す。
async fn tempfile_for_signature(signature: &str) -> anyhow::Result<String> {
    let path = format!("/tmp/cosign-sig-{}.sig", uuid::Uuid::new_v4());
    tokio::fs::write(&path, signature)
        .await
        .map_err(|e| anyhow::anyhow!("署名の一時ファイル書き込みに失敗しました: {}", e))?;
    Ok(path)
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
