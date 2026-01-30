"""gRPC authentication interceptor."""

from __future__ import annotations

from typing import Any, Callable

import grpc
import grpc.aio

from k1s0_auth.errors import AuthError, TokenExpiredError, TokenInvalidError
from k1s0_auth.jwt.verifier import JwtVerifier

_METADATA_KEY = "authorization"


class GrpcAuthInterceptor(grpc.aio.ServerInterceptor):  # type: ignore[misc]
    """gRPC server interceptor that validates Bearer tokens from metadata.

    Args:
        verifier: The JWT verifier instance.
        skip_methods: Fully-qualified method names to skip (e.g. health checks).
    """

    def __init__(
        self,
        verifier: JwtVerifier,
        skip_methods: list[str] | None = None,
    ) -> None:
        self._verifier = verifier
        self._skip_methods = set(skip_methods or [])

    async def intercept_service(
        self,
        continuation: Callable[..., Any],
        handler_call_details: grpc.HandlerCallDetails,
    ) -> Any:
        """Intercept and authenticate incoming gRPC calls."""
        method = handler_call_details.method or ""
        if method in self._skip_methods:
            return await continuation(handler_call_details)

        metadata = dict(handler_call_details.invocation_metadata or [])
        auth_value = metadata.get(_METADATA_KEY, "")

        if not auth_value.startswith("Bearer "):
            return self._abort_handler(
                grpc.StatusCode.UNAUTHENTICATED,
                "Missing Bearer token in metadata",
            )

        token = auth_value[7:]
        try:
            claims = await self._verifier.verify(token)
        except TokenExpiredError as exc:
            return self._abort_handler(grpc.StatusCode.UNAUTHENTICATED, str(exc))
        except TokenInvalidError as exc:
            return self._abort_handler(grpc.StatusCode.UNAUTHENTICATED, str(exc))
        except AuthError as exc:
            return self._abort_handler(grpc.StatusCode.UNAUTHENTICATED, str(exc))

        # Store claims in context for downstream handlers
        handler_call_details.invocation_metadata.append(  # type: ignore[union-attr]
            ("x-auth-claims-sub", claims.sub),
        )

        return await continuation(handler_call_details)

    @staticmethod
    def _abort_handler(code: grpc.StatusCode, details: str) -> grpc.aio.AbortError:
        """Create an abort error for unauthenticated requests."""

        async def _abort(
            request: Any,
            context: grpc.aio.ServicerContext[Any, Any],
        ) -> None:
            await context.abort(code, details)

        return grpc.unary_unary_rpc_method_handler(_abort)  # type: ignore[return-value]
