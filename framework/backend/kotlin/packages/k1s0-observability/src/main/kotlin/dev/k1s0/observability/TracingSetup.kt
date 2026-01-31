package dev.k1s0.observability

import io.github.oshai.kotlinlogging.KotlinLogging
import io.opentelemetry.api.OpenTelemetry
import io.opentelemetry.api.trace.Tracer
import io.opentelemetry.sdk.OpenTelemetrySdk
import io.opentelemetry.sdk.trace.SdkTracerProvider

private val logger = KotlinLogging.logger {}

/**
 * Configures distributed tracing using OpenTelemetry.
 *
 * Provides a tracer instance that can be used to create spans
 * for distributed trace correlation across services.
 */
public object TracingSetup {

    private var openTelemetry: OpenTelemetry = OpenTelemetry.noop()

    /**
     * Initializes the tracing subsystem.
     *
     * @param serviceName The name of the service for trace attribution.
     * @param otlpEndpoint The OTLP collector endpoint. If null, uses a noop exporter.
     * @return The configured [OpenTelemetry] instance.
     */
    public fun initialize(
        serviceName: String,
        otlpEndpoint: String? = null,
    ): OpenTelemetry {
        val tracerProvider = SdkTracerProvider.builder()
            .setResource(
                io.opentelemetry.sdk.resources.Resource.builder()
                    .put("service.name", serviceName)
                    .build(),
            )
            .build()

        openTelemetry = OpenTelemetrySdk.builder()
            .setTracerProvider(tracerProvider)
            .build()

        logger.info { "Tracing initialized for service=$serviceName endpoint=$otlpEndpoint" }
        return openTelemetry
    }

    /** Returns a [Tracer] for creating spans. */
    public fun tracer(instrumentationName: String): Tracer =
        openTelemetry.getTracer(instrumentationName)
}
