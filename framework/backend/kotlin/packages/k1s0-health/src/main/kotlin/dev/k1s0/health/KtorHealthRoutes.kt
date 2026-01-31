package dev.k1s0.health

import io.ktor.http.HttpStatusCode
import io.ktor.server.response.respond
import io.ktor.server.routing.Route
import io.ktor.server.routing.get

/**
 * Installs Kubernetes health check routes into a Ktor routing tree.
 *
 * Registers:
 * - `GET /healthz` - Liveness probe
 * - `GET /readyz` - Readiness probe
 *
 * @param readinessChecks The list of health checks for readiness evaluation.
 */
public fun Route.healthRoutes(readinessChecks: List<HealthCheck> = emptyList()) {
    val liveness = LivenessCheck()
    val readiness = ReadinessCheck(readinessChecks)

    get("/healthz") {
        val status = liveness.check()
        call.respond(HttpStatusCode.OK, status)
    }

    get("/readyz") {
        val status = readiness.check()
        val httpStatus = if (status.status == HealthStatus.Status.UP) {
            HttpStatusCode.OK
        } else {
            HttpStatusCode.ServiceUnavailable
        }
        call.respond(httpStatus, status)
    }
}
