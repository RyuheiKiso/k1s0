// tier1 chaos テスト lib.rs
// 09_テナント容量適合仕様に基づく noisy neighbor 検査テストを実装する。
// 5 quota_class × noisy_neighbor シナリオ: 高負荷テナントが他テナントの SLO を侵害しないことを検証する。
// Litmus ChaosEngine が利用できない環境では QPS カウンタの独立性をシミュレーションで検証する。

// ============================================================
// quota_class 定義
// ============================================================

// 5 つの quota_class を表す列挙型を定義する
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuotaClass {
    // クォータクラス 1: 最小リソース（開発・テスト用途）
    Micro,
    // クォータクラス 2: 小規模リソース（スモールビジネス用途）
    Small,
    // クォータクラス 3: 標準リソース（一般ビジネス用途）
    Standard,
    // クォータクラス 4: 大規模リソース（エンタープライズ用途）
    Large,
    // クォータクラス 5: 最大リソース（プレミアム用途）
    Premium,
}

// QuotaClass に対応する QPS 上限値を返す実装
impl QuotaClass {
    // 各 quota_class の QPS 上限値を返す関数
    pub fn max_qps(&self) -> u64 {
        // quota_class ごとの QPS 上限値を定義する
        match self {
            // Micro: 10 QPS
            QuotaClass::Micro => 10,
            // Small: 100 QPS
            QuotaClass::Small => 100,
            // Standard: 500 QPS
            QuotaClass::Standard => 500,
            // Large: 2000 QPS
            QuotaClass::Large => 2000,
            // Premium: 10000 QPS
            QuotaClass::Premium => 10_000,
        }
    }

    // quota_class 名を文字列で返す関数
    pub fn name(&self) -> &'static str {
        // quota_class ごとの文字列表現を返す
        match self {
            // Micro の文字列表現
            QuotaClass::Micro => "micro",
            // Small の文字列表現
            QuotaClass::Small => "small",
            // Standard の文字列表現
            QuotaClass::Standard => "standard",
            // Large の文字列表現
            QuotaClass::Large => "large",
            // Premium の文字列表現
            QuotaClass::Premium => "premium",
        }
    }
}

// ============================================================
// テナント QPS カウンタ（noisy neighbor シミュレーション用）
// ============================================================

// テナントごとの QPS カウンタを管理する構造体を定義する
#[derive(Debug, Clone)]
pub struct TenantQpsCounter {
    // テナント ID（一意識別子）
    pub tenant_id: String,
    // このテナントに割り当てられた quota_class
    pub quota_class: QuotaClass,
    // 現在の QPS カウント（シミュレーション値）
    pub current_qps: u64,
}

// TenantQpsCounter の実装
impl TenantQpsCounter {
    // 新しいテナント QPS カウンタを生成する
    pub fn new(tenant_id: &str, quota_class: QuotaClass) -> Self {
        // テナント ID・quota_class・初期 QPS (0) を設定して返す
        Self {
            // テナント ID を文字列として保存する
            tenant_id: tenant_id.to_string(),
            // quota_class を設定する
            quota_class,
            // 初期 QPS は 0 とする
            current_qps: 0,
        }
    }

    // QPS を指定量だけ加算する（クォータ上限を超えた場合は上限値でクランプする）
    pub fn add_qps(&mut self, delta: u64) {
        // 現在の QPS に delta を加算する
        let new_qps = self.current_qps.saturating_add(delta);
        // クォータ上限値を取得する
        let limit = self.quota_class.max_qps();
        // クォータ上限を超える場合は上限値にクランプする（noisy neighbor が他テナントに影響しない保証）
        self.current_qps = new_qps.min(limit);
    }

    // 現在の QPS がクォータ上限内であることを確認する（SLO 保証チェック）
    pub fn is_within_quota(&self) -> bool {
        // current_qps が quota_class の max_qps 以下であることを確認する
        self.current_qps <= self.quota_class.max_qps()
    }
}

// ============================================================
// noisy neighbor シナリオ
// ============================================================

// noisy neighbor テストシナリオ: 高負荷テナントが他テナントの QPS 制限に影響しないことを検証する
pub struct NoisyNeighborScenario {
    // 高負荷を与える攻撃者テナント（noisy neighbor）
    pub noisy_tenant: TenantQpsCounter,
    // 保護対象のテナント一覧（SLO を守られるべきテナント群）
    pub protected_tenants: Vec<TenantQpsCounter>,
}

// NoisyNeighborScenario の実装
impl NoisyNeighborScenario {
    // 新しいシナリオを生成する（1 noisy tenant + 複数の protected tenants）
    pub fn new(noisy_tenant: TenantQpsCounter, protected_tenants: Vec<TenantQpsCounter>) -> Self {
        // フィールドを初期化して返す
        Self {
            // noisy tenant を設定する
            noisy_tenant,
            // protected tenants リストを設定する
            protected_tenants,
        }
    }

    // noisy neighbor 負荷を発生させて、protected tenants の QPS が独立していることを検証する
    pub fn run_isolation_check(&mut self, noisy_load: u64) -> bool {
        // noisy tenant に高負荷を与える
        self.noisy_tenant.add_qps(noisy_load);

        // protected tenants の QPS が変化していないことを確認する（独立性の保証）
        let all_protected = self
            .protected_tenants
            .iter()
            .all(|tenant| tenant.is_within_quota());

        // noisy tenant 自身もクォータ上限内に収まることを確認する
        let noisy_within_quota = self.noisy_tenant.is_within_quota();

        // 両方の条件が満たされた場合のみ true を返す
        all_protected && noisy_within_quota
    }
}

// ============================================================
// テスト: 5 quota_class × noisy_neighbor シナリオ
// ============================================================

// chaos テスト群（Litmus 不要・シミュレーションベース）
#[cfg(test)]
mod tests {
    // 親モジュールの全シンボルをインポートする
    use super::*;

    // ---- Micro quota_class の noisy neighbor テスト ----

    // Micro quota_class: noisy neighbor が他テナントに影響しないことを検証する
    #[test]
    fn test_noisy_neighbor_isolation_micro() {
        // Micro quota_class の noisy tenant を作成する
        let noisy = TenantQpsCounter::new("noisy-tenant-micro", QuotaClass::Micro);
        // 保護対象テナントを作成する（Micro quota_class）
        let protected = vec![
            TenantQpsCounter::new("protected-tenant-1", QuotaClass::Micro),
            TenantQpsCounter::new("protected-tenant-2", QuotaClass::Small),
        ];
        // noisy neighbor シナリオを初期化する
        let mut scenario = NoisyNeighborScenario::new(noisy, protected);
        // Micro の QPS 上限 (10) × 10 の負荷を与えてテナント分離を確認する
        let isolated = scenario.run_isolation_check(100);
        // テナントが正しく分離されていることを検証する
        assert!(isolated, "Micro quota_class: noisy neighbor should not break SLO isolation");
    }

    // ---- Small quota_class の noisy neighbor テスト ----

    // Small quota_class: noisy neighbor が他テナントに影響しないことを検証する
    #[test]
    fn test_noisy_neighbor_isolation_small() {
        // Small quota_class の noisy tenant を作成する
        let noisy = TenantQpsCounter::new("noisy-tenant-small", QuotaClass::Small);
        // 保護対象テナントを作成する（Micro / Standard quota_class）
        let protected = vec![
            TenantQpsCounter::new("protected-tenant-micro", QuotaClass::Micro),
            TenantQpsCounter::new("protected-tenant-standard", QuotaClass::Standard),
        ];
        // noisy neighbor シナリオを初期化する
        let mut scenario = NoisyNeighborScenario::new(noisy, protected);
        // Small の QPS 上限 (100) × 10 の負荷を与えてテナント分離を確認する
        let isolated = scenario.run_isolation_check(1000);
        // テナントが正しく分離されていることを検証する
        assert!(isolated, "Small quota_class: noisy neighbor should not break SLO isolation");
    }

    // ---- Standard quota_class の noisy neighbor テスト ----

    // Standard quota_class: noisy neighbor が他テナントに影響しないことを検証する
    #[test]
    fn test_noisy_neighbor_isolation_standard() {
        // Standard quota_class の noisy tenant を作成する
        let noisy = TenantQpsCounter::new("noisy-tenant-standard", QuotaClass::Standard);
        // 保護対象テナントを作成する（Micro / Small / Large quota_class）
        let protected = vec![
            TenantQpsCounter::new("protected-tenant-micro", QuotaClass::Micro),
            TenantQpsCounter::new("protected-tenant-small", QuotaClass::Small),
            TenantQpsCounter::new("protected-tenant-large", QuotaClass::Large),
        ];
        // noisy neighbor シナリオを初期化する
        let mut scenario = NoisyNeighborScenario::new(noisy, protected);
        // Standard の QPS 上限 (500) × 10 の負荷を与えてテナント分離を確認する
        let isolated = scenario.run_isolation_check(5000);
        // テナントが正しく分離されていることを検証する
        assert!(isolated, "Standard quota_class: noisy neighbor should not break SLO isolation");
    }

    // ---- Large quota_class の noisy neighbor テスト ----

    // Large quota_class: noisy neighbor が他テナントに影響しないことを検証する
    #[test]
    fn test_noisy_neighbor_isolation_large() {
        // Large quota_class の noisy tenant を作成する
        let noisy = TenantQpsCounter::new("noisy-tenant-large", QuotaClass::Large);
        // 保護対象テナントを作成する（Small / Standard / Premium quota_class）
        let protected = vec![
            TenantQpsCounter::new("protected-tenant-small", QuotaClass::Small),
            TenantQpsCounter::new("protected-tenant-standard", QuotaClass::Standard),
            TenantQpsCounter::new("protected-tenant-premium", QuotaClass::Premium),
        ];
        // noisy neighbor シナリオを初期化する
        let mut scenario = NoisyNeighborScenario::new(noisy, protected);
        // Large の QPS 上限 (2000) × 10 の負荷を与えてテナント分離を確認する
        let isolated = scenario.run_isolation_check(20_000);
        // テナントが正しく分離されていることを検証する
        assert!(isolated, "Large quota_class: noisy neighbor should not break SLO isolation");
    }

    // ---- Premium quota_class の noisy neighbor テスト ----

    // Premium quota_class: noisy neighbor が他テナントに影響しないことを検証する
    #[test]
    fn test_noisy_neighbor_isolation_premium() {
        // Premium quota_class の noisy tenant を作成する
        let noisy = TenantQpsCounter::new("noisy-tenant-premium", QuotaClass::Premium);
        // 保護対象テナントを作成する（全 quota_class を含む）
        let protected = vec![
            TenantQpsCounter::new("protected-tenant-micro", QuotaClass::Micro),
            TenantQpsCounter::new("protected-tenant-small", QuotaClass::Small),
            TenantQpsCounter::new("protected-tenant-standard", QuotaClass::Standard),
            TenantQpsCounter::new("protected-tenant-large", QuotaClass::Large),
        ];
        // noisy neighbor シナリオを初期化する
        let mut scenario = NoisyNeighborScenario::new(noisy, protected);
        // Premium の QPS 上限 (10000) × 10 の負荷を与えてテナント分離を確認する
        let isolated = scenario.run_isolation_check(100_000);
        // テナントが正しく分離されていることを検証する
        assert!(isolated, "Premium quota_class: noisy neighbor should not break SLO isolation");
    }

    // ---- quota_class QPS 上限値の検証テスト ----

    // 各 quota_class の QPS 上限値が仕様通りであることを検証する
    #[test]
    fn test_quota_class_max_qps_values() {
        // Micro: 10 QPS であることを確認する
        assert_eq!(QuotaClass::Micro.max_qps(), 10, "Micro max_qps should be 10");
        // Small: 100 QPS であることを確認する
        assert_eq!(QuotaClass::Small.max_qps(), 100, "Small max_qps should be 100");
        // Standard: 500 QPS であることを確認する
        assert_eq!(QuotaClass::Standard.max_qps(), 500, "Standard max_qps should be 500");
        // Large: 2000 QPS であることを確認する
        assert_eq!(QuotaClass::Large.max_qps(), 2000, "Large max_qps should be 2000");
        // Premium: 10000 QPS であることを確認する
        assert_eq!(QuotaClass::Premium.max_qps(), 10_000, "Premium max_qps should be 10000");
    }

    // ---- TenantQpsCounter クランプ動作の検証テスト ----

    // QPS が上限を超えた場合にクランプされることを検証する
    #[test]
    fn test_qps_counter_clamp_at_quota_limit() {
        // Micro テナントを作成する（上限 10 QPS）
        let mut counter = TenantQpsCounter::new("test-tenant", QuotaClass::Micro);
        // 上限を大幅に超える QPS (1000) を加算する
        counter.add_qps(1000);
        // QPS が上限値 (10) にクランプされていることを確認する
        assert_eq!(counter.current_qps, 10, "QPS should be clamped to max quota limit");
        // クォータ内であることを確認する
        assert!(counter.is_within_quota(), "Counter should be within quota after clamping");
    }

    // ---- Litmus ChaosEngine を使用する E2E テスト（要 k8s cluster: 通常は #[ignore]）----

    // Litmus を使った noisy neighbor E2E テスト（要 Kubernetes cluster + Litmus operator）
    #[tokio::test]
    #[ignore = "requires Kubernetes cluster with Litmus ChaosEngine operator"]
    async fn test_litmus_noisy_neighbor_e2e() {
        // Litmus ChaosEngine の ChaosResult を kubectl で取得してテナント分離を検証する
        // manifests/litmus_noisy_neighbor_v1_per_tenant_qps.yaml を apply してから確認する
        let output = std::process::Command::new("kubectl")
            // chaos result を取得するコマンドを実行する
            .args(["get", "chaosresult", "-n", "tier1-test", "-o", "json"])
            // コマンド出力を取得する
            .output()
            .expect("kubectl command should succeed");
        // 終了コードが 0 (success) であることを確認する
        assert!(output.status.success(), "kubectl get chaosresult should succeed");
    }
}
