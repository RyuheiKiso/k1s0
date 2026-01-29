"""gRPC server factory with k1s0 interceptors."""

from __future__ import annotations

import logging
from concurrent import futures

import grpc

from k1s0_grpc_server.interceptors.error_handling import ErrorHandlingInterceptor
from k1s0_grpc_server.interceptors.tracing import TracingInterceptor

logger = logging.getLogger("k1s0.grpc.server")


def create_server(
    port: int = 50051,
    max_workers: int = 10,
    enable_tracing: bool = True,  # noqa: FBT001, FBT002
) -> grpc.Server:
    """Create a gRPC server with k1s0 standard interceptors.

    The server is pre-configured with:
    - Error handling interceptor (converts K1s0Exception to gRPC status codes)
    - Tracing interceptor (OpenTelemetry integration)

    Args:
        port: Port number for the server to listen on.
        max_workers: Maximum number of thread pool workers.
        enable_tracing: Whether to enable the tracing interceptor.

    Returns:
        A configured but not-yet-started grpc.Server.
    """
    interceptors: list[grpc.ServerInterceptor] = [ErrorHandlingInterceptor()]
    if enable_tracing:
        interceptors.append(TracingInterceptor())

    server = grpc.server(
        futures.ThreadPoolExecutor(max_workers=max_workers),
        interceptors=interceptors,
    )
    server.add_insecure_port(f"[::]:{port}")

    logger.info("gRPC server created on port %d (workers=%d)", port, max_workers)
    return server
