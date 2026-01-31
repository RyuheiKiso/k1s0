package dev.k1s0.grpc.server

import io.github.oshai.kotlinlogging.KotlinLogging
import io.grpc.BindableService
import io.grpc.Server
import io.grpc.ServerBuilder
import io.grpc.ServerInterceptor

private val logger = KotlinLogging.logger {}

/**
 * Configurable gRPC server builder for k1s0 services.
 *
 * Provides a convenient wrapper around the standard gRPC [ServerBuilder]
 * with built-in support for error handling and tracing interceptors.
 *
 * @property port The port to listen on.
 */
public class GrpcServer(private val port: Int) {

    private val services = mutableListOf<BindableService>()
    private val interceptors = mutableListOf<ServerInterceptor>()

    /** Registers a gRPC service implementation. */
    public fun addService(service: BindableService): GrpcServer {
        services.add(service)
        return this
    }

    /** Registers a server interceptor. */
    public fun addInterceptor(interceptor: ServerInterceptor): GrpcServer {
        interceptors.add(interceptor)
        return this
    }

    /**
     * Builds and starts the gRPC server.
     *
     * Automatically includes [ErrorInterceptor] and [TracingInterceptor].
     *
     * @return The running [Server] instance.
     */
    public fun start(): Server {
        val builder = ServerBuilder.forPort(port)

        // Add built-in interceptors
        builder.intercept(ErrorInterceptor())
        builder.intercept(TracingInterceptor())

        // Add custom interceptors
        interceptors.forEach { builder.intercept(it) }

        // Add services
        services.forEach { builder.addService(it) }

        val server = builder.build().start()
        logger.info { "gRPC server started on port $port" }
        return server
    }
}
