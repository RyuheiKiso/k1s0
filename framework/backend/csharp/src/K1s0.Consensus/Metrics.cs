using Prometheus;

namespace K1s0.Consensus;

/// <summary>
/// Prometheus metrics for the consensus package.
/// </summary>
public static class Metrics
{
    /// <summary>
    /// Metrics for leader election.
    /// </summary>
    public static class LeaderMetrics
    {
        /// <summary>
        /// Total number of successful leader elections.
        /// </summary>
        public static readonly Counter ElectionsTotal = Prometheus.Metrics.CreateCounter(
            "k1s0_consensus_leader_elections_total",
            "Total number of successful leader elections.");

        /// <summary>
        /// Total number of lease renewals.
        /// </summary>
        public static readonly Counter RenewalsTotal = Prometheus.Metrics.CreateCounter(
            "k1s0_consensus_leader_renewals_total",
            "Total number of lease renewals.");

        /// <summary>
        /// Total number of leader losses (lease expiration or failure to renew).
        /// </summary>
        public static readonly Counter LeaderLostTotal = Prometheus.Metrics.CreateCounter(
            "k1s0_consensus_leader_lost_total",
            "Total number of leader losses.");
    }

    /// <summary>
    /// Metrics for distributed locks.
    /// </summary>
    public static class LockMetrics
    {
        /// <summary>
        /// Total number of locks acquired.
        /// </summary>
        public static readonly Counter AcquiredTotal = Prometheus.Metrics.CreateCounter(
            "k1s0_consensus_lock_acquired_total",
            "Total number of locks acquired.");

        /// <summary>
        /// Total number of locks released.
        /// </summary>
        public static readonly Counter ReleasedTotal = Prometheus.Metrics.CreateCounter(
            "k1s0_consensus_lock_released_total",
            "Total number of locks released.");

        /// <summary>
        /// Total number of lock acquisition timeouts.
        /// </summary>
        public static readonly Counter TimeoutsTotal = Prometheus.Metrics.CreateCounter(
            "k1s0_consensus_lock_timeouts_total",
            "Total number of lock acquisition timeouts.");
    }

    /// <summary>
    /// Metrics for saga orchestration.
    /// </summary>
    public static class SagaMetrics
    {
        /// <summary>
        /// Total number of sagas started.
        /// </summary>
        public static readonly Counter StartedTotal = Prometheus.Metrics.CreateCounter(
            "k1s0_consensus_saga_started_total",
            "Total number of sagas started.");

        /// <summary>
        /// Total number of sagas completed successfully.
        /// </summary>
        public static readonly Counter CompletedTotal = Prometheus.Metrics.CreateCounter(
            "k1s0_consensus_saga_completed_total",
            "Total number of sagas completed successfully.");

        /// <summary>
        /// Total number of sagas that were compensated.
        /// </summary>
        public static readonly Counter CompensatedTotal = Prometheus.Metrics.CreateCounter(
            "k1s0_consensus_saga_compensated_total",
            "Total number of sagas that were compensated.");

        /// <summary>
        /// Total number of sagas moved to dead letter.
        /// </summary>
        public static readonly Counter DeadLetterTotal = Prometheus.Metrics.CreateCounter(
            "k1s0_consensus_saga_dead_letter_total",
            "Total number of sagas moved to dead letter.");

        /// <summary>
        /// Total number of saga steps completed.
        /// </summary>
        public static readonly Counter StepsCompletedTotal = Prometheus.Metrics.CreateCounter(
            "k1s0_consensus_saga_steps_completed_total",
            "Total number of saga steps completed.");

        /// <summary>
        /// Total number of saga steps that failed.
        /// </summary>
        public static readonly Counter StepsFailedTotal = Prometheus.Metrics.CreateCounter(
            "k1s0_consensus_saga_steps_failed_total",
            "Total number of saga steps that failed.");

        /// <summary>
        /// Total number of saga timeouts.
        /// </summary>
        public static readonly Counter TimeoutsTotal = Prometheus.Metrics.CreateCounter(
            "k1s0_consensus_saga_timeouts_total",
            "Total number of saga timeouts.");
    }
}
