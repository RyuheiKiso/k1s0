package consensus

import "github.com/prometheus/client_golang/prometheus"

var (
	// Leader election metrics.
	metricsLeaderElections = prometheus.NewCounterVec(
		prometheus.CounterOpts{
			Name: "k1s0_consensus_leader_elections_total",
			Help: "Total number of leader election events.",
		},
		[]string{"lease_key", "outcome"},
	)

	metricsLeaderRenewals = prometheus.NewCounterVec(
		prometheus.CounterOpts{
			Name: "k1s0_consensus_leader_renewals_total",
			Help: "Total number of successful leader lease renewals.",
		},
		[]string{"lease_key"},
	)

	// Lock metrics.
	metricsLockAcquisitions = prometheus.NewCounterVec(
		prometheus.CounterOpts{
			Name: "k1s0_consensus_lock_acquisitions_total",
			Help: "Total number of lock acquisition attempts.",
		},
		[]string{"key", "backend", "outcome"},
	)

	// Fencing metrics.
	metricsFenceTokenViolations = prometheus.NewCounter(
		prometheus.CounterOpts{
			Name: "k1s0_consensus_fence_token_violations_total",
			Help: "Total number of fence token violations detected.",
		},
	)

	// Saga metrics.
	metricsSagaExecutions = prometheus.NewCounterVec(
		prometheus.CounterOpts{
			Name: "k1s0_consensus_saga_executions_total",
			Help: "Total number of saga execution events.",
		},
		[]string{"saga_name", "outcome"},
	)

	metricsSagaStepDuration = prometheus.NewHistogramVec(
		prometheus.HistogramOpts{
			Name:    "k1s0_consensus_saga_duration_seconds",
			Help:    "Duration of saga executions in seconds.",
			Buckets: prometheus.DefBuckets,
		},
		[]string{"saga_name", "step"},
	)
)

func init() {
	prometheus.MustRegister(
		metricsLeaderElections,
		metricsLeaderRenewals,
		metricsLockAcquisitions,
		metricsFenceTokenViolations,
		metricsSagaExecutions,
		metricsSagaStepDuration,
	)
}
