"""Tests for gRPC server factory."""

from __future__ import annotations

import grpc

from k1s0_grpc_server.server import create_server


class TestCreateServer:
    def test_creates_server(self) -> None:
        server = create_server(port=50099, max_workers=2)
        assert isinstance(server, grpc.Server)
        server.stop(grace=0)

    def test_creates_server_without_tracing(self) -> None:
        server = create_server(port=50098, enable_tracing=False)
        assert isinstance(server, grpc.Server)
        server.stop(grace=0)
