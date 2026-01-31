package dev.k1s0.health

import kotlinx.serialization.Serializable

/**
 * Interface for health check probes used by Kubernetes liveness and readiness checks.
 */
public fun interface HealthCheck {
    /**
     * Performs the health check.
     *
     * @return The [HealthStatus] indicating the component's health.
     */
    public suspend fun check(): HealthStatus
}

/**
 * Health status of a single component or the overall service.
 *
 * @property name The name of the component being checked.
 * @property status Whether the component is healthy.
 * @property message Optional detail message.
 */
@Serializable
public data class HealthStatus(
    val name: String,
    val status: Status,
    val message: String? = null,
) {
    /** Health status values. */
    public enum class Status { UP, DOWN }
}

/** A liveness check that always returns UP (the process is running). */
public class LivenessCheck : HealthCheck {
    override suspend fun check(): HealthStatus =
        HealthStatus(name = "liveness", status = HealthStatus.Status.UP)
}

/**
 * A readiness check that aggregates multiple [HealthCheck] instances.
 *
 * Returns DOWN if any of the checks fail.
 */
public class ReadinessCheck(
    private val checks: List<HealthCheck>,
) : HealthCheck {
    override suspend fun check(): HealthStatus {
        val results = checks.map { it.check() }
        val allUp = results.all { it.status == HealthStatus.Status.UP }
        return HealthStatus(
            name = "readiness",
            status = if (allUp) HealthStatus.Status.UP else HealthStatus.Status.DOWN,
            message = if (allUp) null else results.filter { it.status == HealthStatus.Status.DOWN }
                .joinToString(", ") { "${it.name}: ${it.message ?: "DOWN"}" },
        )
    }
}
