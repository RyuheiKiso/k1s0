package dev.k1s0.health

import io.kotest.matchers.shouldBe
import kotlinx.coroutines.test.runTest
import org.junit.jupiter.api.Test

class HealthCheckTest {

    @Test
    fun `liveness check returns UP`() = runTest {
        val check = LivenessCheck()
        val result = check.check()

        result.status shouldBe HealthStatus.Status.UP
        result.name shouldBe "liveness"
    }

    @Test
    fun `readiness check returns UP when all checks pass`() = runTest {
        val checks = listOf(
            HealthCheck { HealthStatus("db", HealthStatus.Status.UP) },
            HealthCheck { HealthStatus("cache", HealthStatus.Status.UP) },
        )
        val readiness = ReadinessCheck(checks)

        val result = readiness.check()

        result.status shouldBe HealthStatus.Status.UP
    }

    @Test
    fun `readiness check returns DOWN when any check fails`() = runTest {
        val checks = listOf(
            HealthCheck { HealthStatus("db", HealthStatus.Status.UP) },
            HealthCheck { HealthStatus("cache", HealthStatus.Status.DOWN, "Connection refused") },
        )
        val readiness = ReadinessCheck(checks)

        val result = readiness.check()

        result.status shouldBe HealthStatus.Status.DOWN
    }
}
