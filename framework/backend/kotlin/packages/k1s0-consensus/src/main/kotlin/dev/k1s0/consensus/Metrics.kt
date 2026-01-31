package dev.k1s0.consensus

import io.github.oshai.kotlinlogging.KotlinLogging
import io.micrometer.core.instrument.Counter
import io.micrometer.core.instrument.MeterRegistry
import io.micrometer.core.instrument.Timer
import java.util.concurrent.ConcurrentHashMap

private val logger = KotlinLogging.logger {}

/**
 * Metrics for leader election operations.
 */
public class LeaderMetrics(private val registry: MeterRegistry) {

    private val electionsTotal: Counter = Counter.builder("k1s0.consensus.leader.elections.total")
        .description("Total number of leader elections won")
        .register(registry)

    private val renewalsTotal: Counter = Counter.builder("k1s0.consensus.leader.renewals.total")
        .description("Total number of lease renewals")
        .register(registry)

    private val lossesTotal: Counter = Counter.builder("k1s0.consensus.leader.losses.total")
        .description("Total number of leadership losses")
        .register(registry)

    private val releasesTotal: Counter = Counter.builder("k1s0.consensus.leader.releases.total")
        .description("Total number of voluntary leadership releases")
        .register(registry)

    public fun leaderElected() {
        electionsTotal.increment()
    }

    public fun leaseRenewed() {
        renewalsTotal.increment()
    }

    public fun leaderLost() {
        lossesTotal.increment()
    }

    public fun leaderReleased() {
        releasesTotal.increment()
    }
}

/**
 * Metrics for distributed lock operations.
 */
public class LockMetrics(private val registry: MeterRegistry) {

    private val acquiredCounters = ConcurrentHashMap<String, Counter>()
    private val releasedCounters = ConcurrentHashMap<String, Counter>()
    private val timeoutCounters = ConcurrentHashMap<String, Counter>()
    private val extendedCounters = ConcurrentHashMap<String, Counter>()

    public fun lockAcquired(lockName: String) {
        acquiredCounters.computeIfAbsent(lockName) {
            Counter.builder("k1s0.consensus.lock.acquired.total")
                .tag("lock_name", lockName)
                .description("Total number of lock acquisitions")
                .register(registry)
        }.increment()
    }

    public fun lockReleased(lockName: String) {
        releasedCounters.computeIfAbsent(lockName) {
            Counter.builder("k1s0.consensus.lock.released.total")
                .tag("lock_name", lockName)
                .description("Total number of lock releases")
                .register(registry)
        }.increment()
    }

    public fun lockTimeout(lockName: String) {
        timeoutCounters.computeIfAbsent(lockName) {
            Counter.builder("k1s0.consensus.lock.timeout.total")
                .tag("lock_name", lockName)
                .description("Total number of lock acquisition timeouts")
                .register(registry)
        }.increment()
    }

    public fun lockExtended(lockName: String) {
        extendedCounters.computeIfAbsent(lockName) {
            Counter.builder("k1s0.consensus.lock.extended.total")
                .tag("lock_name", lockName)
                .description("Total number of lock extensions")
                .register(registry)
        }.increment()
    }
}

/**
 * Metrics for saga operations.
 */
public class SagaMetrics(private val registry: MeterRegistry) {

    private val startedCounters = ConcurrentHashMap<String, Counter>()
    private val completedCounters = ConcurrentHashMap<String, Counter>()
    private val stepFailedCounters = ConcurrentHashMap<String, Counter>()
    private val compensationFailedCounters = ConcurrentHashMap<String, Counter>()
    private val deadLetteredCounters = ConcurrentHashMap<String, Counter>()

    public fun sagaStarted(sagaName: String) {
        startedCounters.computeIfAbsent(sagaName) {
            Counter.builder("k1s0.consensus.saga.started.total")
                .tag("saga_name", sagaName)
                .description("Total number of sagas started")
                .register(registry)
        }.increment()
    }

    public fun sagaCompleted(sagaName: String) {
        completedCounters.computeIfAbsent(sagaName) {
            Counter.builder("k1s0.consensus.saga.completed.total")
                .tag("saga_name", sagaName)
                .description("Total number of sagas completed successfully")
                .register(registry)
        }.increment()
    }

    public fun sagaStepFailed(sagaName: String, stepName: String) {
        val key = "$sagaName:$stepName"
        stepFailedCounters.computeIfAbsent(key) {
            Counter.builder("k1s0.consensus.saga.step.failed.total")
                .tag("saga_name", sagaName)
                .tag("step_name", stepName)
                .description("Total number of saga step failures")
                .register(registry)
        }.increment()
    }

    public fun sagaCompensationFailed(sagaName: String, stepName: String) {
        val key = "$sagaName:$stepName"
        compensationFailedCounters.computeIfAbsent(key) {
            Counter.builder("k1s0.consensus.saga.compensation.failed.total")
                .tag("saga_name", sagaName)
                .tag("step_name", stepName)
                .description("Total number of saga compensation failures")
                .register(registry)
        }.increment()
    }

    public fun sagaDeadLettered(sagaName: String) {
        deadLetteredCounters.computeIfAbsent(sagaName) {
            Counter.builder("k1s0.consensus.saga.dead_letter.total")
                .tag("saga_name", sagaName)
                .description("Total number of sagas moved to dead letter queue")
                .register(registry)
        }.increment()
    }
}
