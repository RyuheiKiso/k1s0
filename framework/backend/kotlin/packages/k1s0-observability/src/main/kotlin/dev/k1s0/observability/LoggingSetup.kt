package dev.k1s0.observability

import io.github.oshai.kotlinlogging.KotlinLogging

private val logger = KotlinLogging.logger {}

/**
 * Configures structured logging for k1s0 services.
 *
 * Uses Logback as the SLF4J implementation with JSON-formatted output
 * suitable for log aggregation systems.
 */
public object LoggingSetup {

    /**
     * Initializes logging with the given service name and environment.
     *
     * @param serviceName The name of the service for log context.
     * @param env The environment (dev, stg, prod).
     */
    public fun initialize(serviceName: String, env: String = "default") {
        System.setProperty("SERVICE_NAME", serviceName)
        System.setProperty("SERVICE_ENV", env)
        logger.info { "Logging initialized for service=$serviceName env=$env" }
    }
}
