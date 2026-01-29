"""Error handling interceptor that converts K1s0Exception to gRPC status codes."""

from __future__ import annotations

import logging
from typing import Any

import grpc

from k1s0_error.exception import (
    ConflictException,
    ForbiddenException,
    K1s0Exception,
    NotFoundException,
    UnauthorizedException,
    ValidationException,
)

logger = logging.getLogger("k1s0.grpc.interceptor.error")

_EXCEPTION_TO_GRPC_CODE: dict[type[K1s0Exception], grpc.StatusCode] = {
    NotFoundException: grpc.StatusCode.NOT_FOUND,
    ValidationException: grpc.StatusCode.INVALID_ARGUMENT,
    ConflictException: grpc.StatusCode.ALREADY_EXISTS,
    UnauthorizedException: grpc.StatusCode.UNAUTHENTICATED,
    ForbiddenException: grpc.StatusCode.PERMISSION_DENIED,
}


def _map_exception_to_status(exc: K1s0Exception) -> grpc.StatusCode:
    """Map a K1s0Exception to the appropriate gRPC status code."""
    return _EXCEPTION_TO_GRPC_CODE.get(type(exc), grpc.StatusCode.INTERNAL)


class ErrorHandlingInterceptor(grpc.ServerInterceptor):
    """Intercepts unhandled K1s0Exceptions and converts them to gRPC status codes."""

    def intercept_service(
        self,
        continuation: Any,
        handler_call_details: grpc.HandlerCallDetails,
    ) -> Any:
        """Intercept the service call."""
        handler = continuation(handler_call_details)
        if handler is None:
            return None

        # Wrap unary-unary handlers
        if handler.unary_unary:
            original = handler.unary_unary

            def wrapped_unary_unary(request: Any, context: grpc.ServicerContext) -> Any:
                try:
                    return original(request, context)
                except K1s0Exception as exc:
                    code = _map_exception_to_status(exc)
                    context.set_code(code)
                    context.set_details(exc.detail)
                    logger.warning(
                        "gRPC error: %s -> %s: %s",
                        exc.error_code,
                        code.name,
                        exc.detail,
                    )
                    return None
                except Exception:
                    context.set_code(grpc.StatusCode.INTERNAL)
                    context.set_details("Internal server error")
                    logger.exception("Unhandled exception in gRPC handler")
                    return None

            return grpc.unary_unary_rpc_method_handler(
                wrapped_unary_unary,
                request_deserializer=handler.request_deserializer,
                response_serializer=handler.response_serializer,
            )

        return handler
