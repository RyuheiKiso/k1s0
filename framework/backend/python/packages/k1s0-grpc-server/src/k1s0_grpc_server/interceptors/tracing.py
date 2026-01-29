"""Tracing interceptor for gRPC server."""

from __future__ import annotations

import logging
from typing import Any

import grpc

from k1s0_observability.tracing import get_tracer

logger = logging.getLogger("k1s0.grpc.interceptor.tracing")

_tracer = get_tracer("k1s0.grpc.server")


class TracingInterceptor(grpc.ServerInterceptor):
    """Adds OpenTelemetry tracing spans to gRPC method calls."""

    def intercept_service(
        self,
        continuation: Any,
        handler_call_details: grpc.HandlerCallDetails,
    ) -> Any:
        """Intercept the service call and create a tracing span."""
        method = handler_call_details.method or "unknown"
        handler = continuation(handler_call_details)
        if handler is None:
            return None

        if handler.unary_unary:
            original = handler.unary_unary

            def traced_unary_unary(request: Any, context: grpc.ServicerContext) -> Any:
                with _tracer.start_as_current_span(
                    name=f"grpc {method}",
                    attributes={"rpc.method": method, "rpc.system": "grpc"},
                ):
                    return original(request, context)

            return grpc.unary_unary_rpc_method_handler(
                traced_unary_unary,
                request_deserializer=handler.request_deserializer,
                response_serializer=handler.response_serializer,
            )

        return handler
