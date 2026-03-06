#[derive(Debug, Clone, Default)]
pub struct FlowKpi {
    pub total_started: i64,
    pub total_completed: i64,
    pub total_failed: i64,
    pub total_in_progress: i64,
    pub completion_rate: f64,
    pub avg_duration_seconds: f64,
    pub p50_duration_seconds: f64,
    pub p95_duration_seconds: f64,
    pub p99_duration_seconds: f64,
    pub bottleneck_step: Option<BottleneckStep>,
}

#[derive(Debug, Clone)]
pub struct BottleneckStep {
    pub event_type: String,
    pub step_index: i32,
    pub avg_duration_seconds: f64,
    pub timeout_rate: f64,
}

#[derive(Debug, Clone)]
pub struct SloStatus {
    pub target_completion_seconds: i32,
    pub target_success_rate: f64,
    pub current_success_rate: f64,
    pub is_violated: bool,
    pub burn_rate: f64,
    pub estimated_budget_exhaustion_hours: f64,
}

#[derive(Debug, Clone)]
pub struct BurnRateWindow {
    pub window: String,
    pub burn_rate: f64,
    pub error_budget_remaining: f64,
}
