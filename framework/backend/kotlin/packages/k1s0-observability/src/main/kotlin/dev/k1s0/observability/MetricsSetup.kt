package dev.k1s0.observability

import io.github.oshai.kotlinlogging.KotlinLogging
import io.opentelemetry.api.metrics.Meter
import io.opentelemetry.sdk.OpenTelemetrySdk
import io.opentelemetry.sdk.metrics.SdkMeterProvider

private val logger = KotlinLogging.logger {}

/**
 * Configures metrics collection using OpenTelemetry.
 *
 * Provides a meter instance for recording counters, histograms,
 * and gauges that are exported to an OTLP-compatible backend.
 */
public object MetricsSetup {

    private var meterProvider: SdkMeterProvider? = null

    /**
     * Initializes the metrics subsystem.
     *
     * @param serviceName The name of the service for metric attribution.
     */
    public fun initialize(serviceName: String) {
        meterProvider = SdkMeterProvider.builder()
            .setResource(
                io.opentelemetry.sdk.resources.Resource.builder()
                    .put("service.name", serviceName)
                    .build(),
            )
            .build()

        logger.info { "Metrics initialized for service=$serviceName" }
    }

    /** Returns a [Meter] for recording metrics. */
    public fun meter(instrumentationName: String): Meter {
        val sdk = meterProvider?.let {
            OpenTelemetrySdk.builder().setMeterProvider(it).build()
        } ?: OpenTelemetrySdk.builder().build()
        return sdk.getMeter(instrumentationName)
    }
}
