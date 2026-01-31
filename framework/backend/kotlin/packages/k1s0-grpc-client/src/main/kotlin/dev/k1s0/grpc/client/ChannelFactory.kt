package dev.k1s0.grpc.client

import io.github.oshai.kotlinlogging.KotlinLogging
import io.grpc.ManagedChannel
import io.grpc.ManagedChannelBuilder
import java.util.concurrent.TimeUnit

private val logger = KotlinLogging.logger {}

/**
 * Factory for creating gRPC [ManagedChannel] instances.
 *
 * Provides pre-configured channels with sensible defaults for
 * inter-service communication within k1s0 microservices.
 */
public object ChannelFactory {

    /**
     * Creates a new gRPC channel to the specified target.
     *
     * @param host The target hostname.
     * @param port The target port.
     * @param usePlaintext Whether to use plaintext (no TLS). Defaults to false.
     * @param idleTimeoutSeconds Idle timeout in seconds before the channel is closed.
     * @return A configured [ManagedChannel].
     */
    public fun create(
        host: String,
        port: Int,
        usePlaintext: Boolean = false,
        idleTimeoutSeconds: Long = 300,
    ): ManagedChannel {
        val builder = ManagedChannelBuilder.forAddress(host, port)
            .idleTimeout(idleTimeoutSeconds, TimeUnit.SECONDS)

        if (usePlaintext) {
            builder.usePlaintext()
        }

        logger.info { "Creating gRPC channel to $host:$port (plaintext=$usePlaintext)" }
        return builder.build()
    }

    /**
     * Gracefully shuts down a channel.
     *
     * @param channel The channel to shut down.
     * @param timeoutSeconds Maximum time to wait for shutdown.
     */
    public fun shutdown(channel: ManagedChannel, timeoutSeconds: Long = 5) {
        if (!channel.isShutdown) {
            channel.shutdown()
            if (!channel.awaitTermination(timeoutSeconds, TimeUnit.SECONDS)) {
                logger.warn { "Channel did not terminate in time, forcing shutdown" }
                channel.shutdownNow()
            }
        }
    }
}
