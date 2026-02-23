#[derive(Debug, Clone)]
pub struct CircuitBreakerMetrics {
    pub failure_count: u32,
    pub success_count: u32,
    pub state: String,
}
