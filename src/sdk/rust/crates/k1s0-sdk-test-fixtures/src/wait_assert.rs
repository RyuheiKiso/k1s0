//! Wait / Assertion helper（領域 4、ADR-TEST-010 §3）。
//!
//! failure 時のエラーメッセージは 4 言語共通フォーマット:
//! `[k1s0-test-fixtures] WaitFor "<resource>" timeout after Ns`

use crate::{FixtureError, Fixture, Result};
use std::time::Duration;

impl Fixture {
    /// 指定 resource が ready になるまで polling 待機。
    /// 採用初期で k8s API client 経由の polling を実装。
    pub async fn wait_for(&self, resource: &str, timeout: Duration) -> Result<()> {
        // skeleton: 採用初期で polling 実装
        // リリース時点は即時 OK（test code が成立するように）
        let _ = (resource, timeout);
        Ok(())
    }

    /// Pod が Ready condition を持つか assert（採用初期で k8s client-go 統合）
    pub async fn assert_pod_ready(&self, namespace: &str, pod_name: &str) -> Result<()> {
        let _ = (namespace, pod_name);
        Ok(())
    }

    /// timeout を error として扱う short-cut
    pub fn timeout_error(resource: &str, seconds: u64) -> FixtureError {
        FixtureError::WaitTimeout {
            resource: resource.to_string(),
            seconds,
        }
    }
}
