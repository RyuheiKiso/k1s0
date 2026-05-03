//! Setup / Teardown / Fixture struct（領域 1、ADR-TEST-010 §3）。

use crate::mock_builder::MockBuilderRoot;
use crate::options::Options;
use crate::Result;

/// Setup の戻り値。利用者が test 内で SDK client init / mock builder を取得する経路。
pub struct Fixture {
    /// Setup に渡された Options（再利用 + debug 用）
    pub options: Options,
    /// 12 service の mock data builder への entry point
    pub mock_builder: MockBuilderRoot,
}

/// kind cluster 起動 + k1s0 install + SDK client の前提整備。
///
/// 採用初期で kind / helm / kubectl の async spawn を実装する。
/// リリース時点では skeleton（cluster 起動済前提で fixture struct のみ返す）。
pub async fn setup(opts: Options) -> Result<Fixture> {
    // tenant 未指定時は既定値で埋める
    let mut opts = opts;
    if opts.tenant.is_empty() {
        opts.tenant = "tenant-a".to_string();
    }
    if opts.namespace.is_empty() {
        opts.namespace = "k1s0".to_string();
    }
    if opts.kind_nodes == 0 {
        opts.kind_nodes = 2;
    }

    // 採用初期で tools/e2e/user/up.sh を spawn する形に拡張
    let tenant = opts.tenant.clone();
    Ok(Fixture {
        options: opts,
        mock_builder: MockBuilderRoot::new(tenant),
    })
}

impl Fixture {
    /// 後片付け（採用初期で tools/e2e/user/down.sh を spawn）。
    /// Drop trait での自動 teardown は将来的な拡張。
    pub async fn teardown(self) {
        // skeleton: 採用初期で down.sh spawn を実装
    }

    /// tier1 facade Pod が Ready 状態になるまで待機（採用初期で kubectl wait wrapper）
    pub async fn wait_for_tier1_facade_ready(&self) -> Result<()> {
        // 採用初期で k8s API client + Pod readiness 待機を実装
        Ok(())
    }
}
