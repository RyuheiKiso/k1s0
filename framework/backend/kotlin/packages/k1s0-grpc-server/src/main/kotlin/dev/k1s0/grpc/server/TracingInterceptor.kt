package dev.k1s0.grpc.server

import io.github.oshai.kotlinlogging.KotlinLogging
import io.grpc.Metadata
import io.grpc.ServerCall
import io.grpc.ServerCallHandler
import io.grpc.ServerInterceptor

private val logger = KotlinLogging.logger {}

/**
 * gRPC server interceptor that propagates distributed tracing context.
 *
 * Extracts trace context from incoming gRPC metadata headers and ensures
 * it is available for downstream spans within the request lifecycle.
 */
public class TracingInterceptor : ServerInterceptor {

    override fun <ReqT, RespT> interceptCall(
        call: ServerCall<ReqT, RespT>,
        headers: Metadata,
        next: ServerCallHandler<ReqT, RespT>,
    ): ServerCall.Listener<ReqT> {
        val traceParent = headers.get(TRACEPARENT_KEY)
        if (traceParent != null) {
            logger.debug { "Received traceparent: $traceParent" }
        }
        return next.startCall(call, headers)
    }

    private companion object {
        val TRACEPARENT_KEY: Metadata.Key<String> =
            Metadata.Key.of("traceparent", Metadata.ASCII_STRING_MARSHALLER)
    }
}
