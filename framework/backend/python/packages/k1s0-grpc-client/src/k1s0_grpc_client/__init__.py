"""k1s0-grpc-client: gRPC client utilities."""

from __future__ import annotations

from k1s0_grpc_client.channel import close_channel
from k1s0_grpc_client.factory import create_channel

__all__ = ["close_channel", "create_channel"]
