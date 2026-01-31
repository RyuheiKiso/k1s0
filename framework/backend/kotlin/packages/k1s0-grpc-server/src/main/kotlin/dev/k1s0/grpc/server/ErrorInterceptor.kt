package dev.k1s0.grpc.server

import dev.k1s0.error.K1s0Exception
import io.github.oshai.kotlinlogging.KotlinLogging
import io.grpc.ForwardingServerCallListener.SimpleForwardingServerCallListener
import io.grpc.Metadata
import io.grpc.ServerCall
import io.grpc.ServerCallHandler
import io.grpc.ServerInterceptor
import io.grpc.Status

private val logger = KotlinLogging.logger {}

/**
 * gRPC server interceptor that converts [K1s0Exception] instances
 * into appropriate gRPC [Status] responses.
 *
 * Unhandled exceptions are logged and returned as INTERNAL errors.
 */
public class ErrorInterceptor : ServerInterceptor {

    override fun <ReqT, RespT> interceptCall(
        call: ServerCall<ReqT, RespT>,
        headers: Metadata,
        next: ServerCallHandler<ReqT, RespT>,
    ): ServerCall.Listener<ReqT> {
        val listener = next.startCall(call, headers)
        return object : SimpleForwardingServerCallListener<ReqT>(listener) {
            override fun onHalfClose() {
                try {
                    super.onHalfClose()
                } catch (ex: K1s0Exception) {
                    logger.warn { "K1s0Exception in gRPC call: ${ex.serviceErrorCode} - ${ex.detail}" }
                    val grpcCode = Status.Code.valueOf(ex.errorCode.grpcStatus)
                    val status = Status.fromCode(grpcCode)
                        .withDescription(ex.detail)
                        .withCause(ex)
                    call.close(status, Metadata())
                } catch (ex: Exception) {
                    logger.error(ex) { "Unhandled exception in gRPC call" }
                    call.close(
                        Status.INTERNAL.withDescription("Internal server error").withCause(ex),
                        Metadata(),
                    )
                }
            }
        }
    }
}
