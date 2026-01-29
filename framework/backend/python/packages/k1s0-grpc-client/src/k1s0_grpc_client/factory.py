"""gRPC channel factory."""

from __future__ import annotations

import logging

import grpc

logger = logging.getLogger("k1s0.grpc.client")


def create_channel(
    target: str,
    *,
    secure: bool = False,
    options: list[tuple[str, str | int]] | None = None,
) -> grpc.Channel:
    """Create a gRPC channel to the specified target.

    Args:
        target: Target address in "host:port" format.
        secure: Whether to use TLS. Defaults to insecure.
        options: Optional gRPC channel options.

    Returns:
        A gRPC Channel instance.
    """
    channel_options = options or []

    if secure:
        credentials = grpc.ssl_channel_credentials()
        channel = grpc.secure_channel(target, credentials, options=channel_options)
    else:
        channel = grpc.insecure_channel(target, options=channel_options)

    logger.info("Created gRPC channel to %s (secure=%s)", target, secure)
    return channel
