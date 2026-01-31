package dev.k1s0.android.observability

import java.util.UUID

/**
 * Configuration for distributed tracing in k1s0 Android applications.
 *
 * @property serviceName The name of this service for trace identification.
 * @property endpoint The OpenTelemetry collector endpoint URL.
 * @property sampleRate The sampling rate from 0.0 (none) to 1.0 (all). Defaults to 1.0.
 */
data class TracingConfig(
    val serviceName: String,
    val endpoint: String? = null,
    val sampleRate: Double = 1.0,
)

/**
 * Manages trace context for distributed tracing.
 *
 * Generates and propagates trace IDs and span IDs for correlating
 * requests across service boundaries.
 */
class TracingSetup(private val config: TracingConfig) {

    /**
     * Generates a new trace ID.
     *
     * @return A 32-character hexadecimal trace ID.
     */
    fun generateTraceId(): String {
        return UUID.randomUUID().toString().replace("-", "")
    }

    /**
     * Generates a new span ID.
     *
     * @return A 16-character hexadecimal span ID.
     */
    fun generateSpanId(): String {
        return UUID.randomUUID().toString().replace("-", "").take(16)
    }

    /**
     * Creates trace context headers for outgoing HTTP requests.
     *
     * Follows the W3C Trace Context specification for the `traceparent` header.
     *
     * @param traceId The trace ID. If null, a new one is generated.
     * @param spanId The span ID. If null, a new one is generated.
     * @return A map of header name to value for trace propagation.
     */
    fun createTraceHeaders(
        traceId: String? = null,
        spanId: String? = null,
    ): Map<String, String> {
        val tid = traceId ?: generateTraceId()
        val sid = spanId ?: generateSpanId()
        val flags = if (config.sampleRate >= 1.0) "01" else "00"

        return mapOf(
            "traceparent" to "00-$tid-$sid-$flags",
        )
    }

    /** The configured service name. */
    val serviceName: String get() = config.serviceName
}
