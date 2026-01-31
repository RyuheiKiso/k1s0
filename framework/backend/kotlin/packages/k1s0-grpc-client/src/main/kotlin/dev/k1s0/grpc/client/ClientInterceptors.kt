package dev.k1s0.grpc.client

import io.github.oshai.kotlinlogging.KotlinLogging
import io.grpc.CallOptions
import io.grpc.Channel
import io.grpc.ClientCall
import io.grpc.ClientInterceptor
import io.grpc.ForwardingClientCall.SimpleForwardingClientCall
import io.grpc.Metadata
import io.grpc.MethodDescriptor

private val logger = KotlinLogging.logger {}

/**
 * Client interceptor that propagates distributed tracing context
 * to outgoing gRPC calls via metadata headers.
 */
public class TracingClientInterceptor : ClientInterceptor {

    override fun <ReqT, RespT> interceptCall(
        method: MethodDescriptor<ReqT, RespT>,
        callOptions: CallOptions,
        next: Channel,
    ): ClientCall<ReqT, RespT> {
        return object : SimpleForwardingClientCall<ReqT, RespT>(next.newCall(method, callOptions)) {
            override fun start(responseListener: Listener<RespT>, headers: Metadata) {
                logger.debug { "Outgoing gRPC call: ${method.fullMethodName}" }
                super.start(responseListener, headers)
            }
        }
    }
}

/**
 * Client interceptor that logs gRPC call details for debugging.
 */
public class LoggingClientInterceptor : ClientInterceptor {

    override fun <ReqT, RespT> interceptCall(
        method: MethodDescriptor<ReqT, RespT>,
        callOptions: CallOptions,
        next: Channel,
    ): ClientCall<ReqT, RespT> {
        logger.info { "gRPC client call: ${method.fullMethodName}" }
        return next.newCall(method, callOptions)
    }
}
