"""gRPC channel utilities."""

from __future__ import annotations

import logging

import grpc

logger = logging.getLogger("k1s0.grpc.client")


def close_channel(channel: grpc.Channel) -> None:
    """Gracefully close a gRPC channel.

    Args:
        channel: The gRPC channel to close.
    """
    channel.close()
    logger.info("gRPC channel closed")


def check_channel_connectivity(channel: grpc.Channel) -> grpc.ChannelConnectivity:
    """Check the current connectivity state of a gRPC channel.

    Args:
        channel: The gRPC channel to check.

    Returns:
        The current ChannelConnectivity state.
    """
    # Subscribe and immediately cancel to get state
    future = grpc.channel_ready_future(channel)
    try:
        future.result(timeout=0.1)
        return grpc.ChannelConnectivity.READY
    except grpc.FutureTimeoutError:
        return grpc.ChannelConnectivity.CONNECTING
    except Exception:
        return grpc.ChannelConnectivity.TRANSIENT_FAILURE
