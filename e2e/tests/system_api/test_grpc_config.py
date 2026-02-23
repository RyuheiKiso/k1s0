"""gRPC config-service E2E テスト。"""

import pytest

try:
    import grpc
except ImportError:
    grpc = None


def _identity(x):
    return x


class TestGrpcConfigService:
    """gRPC ConfigService の E2E テスト。"""

    SERVICE = "k1s0.system.config.v1.ConfigService"

    def test_get_config_rpc(self, grpc_config_channel):
        """GetConfig RPC が正常に応答する。"""
        # GetConfigRequest: namespace=1, key=2
        namespace = b"e2e-test"
        key = b"test-key"
        request_bytes = (
            b"\x0a" + bytes([len(namespace)]) + namespace + b"\x12" + bytes([len(key)]) + key
        )

        try:
            response_bytes = grpc_config_channel.unary_unary(
                f"/{self.SERVICE}/GetConfig",
                request_serializer=_identity,
                response_deserializer=_identity,
            )(request_bytes, timeout=5)
        except grpc.RpcError as e:
            if e.code() == grpc.StatusCode.UNIMPLEMENTED:
                pytest.skip("GetConfig RPC is not implemented yet")
            if e.code() == grpc.StatusCode.NOT_FOUND:
                return  # 設定が見つからないのは正常な応答
            raise

        assert isinstance(response_bytes, bytes)

    def test_list_configs_rpc(self, grpc_config_channel):
        """ListConfigs RPC が正常に応答する。"""
        # ListConfigsRequest: namespace=1
        namespace = b"e2e-test"
        request_bytes = b"\x0a" + bytes([len(namespace)]) + namespace

        try:
            response_bytes = grpc_config_channel.unary_unary(
                f"/{self.SERVICE}/ListConfigs",
                request_serializer=_identity,
                response_deserializer=_identity,
            )(request_bytes, timeout=5)
        except grpc.RpcError as e:
            if e.code() == grpc.StatusCode.UNIMPLEMENTED:
                pytest.skip("ListConfigs RPC is not implemented yet")
            raise

        assert isinstance(response_bytes, bytes)

    def test_get_service_config_rpc(self, grpc_config_channel):
        """GetServiceConfig RPC が正常に応答する。"""
        # GetServiceConfigRequest: service_name=1
        service_name = b"auth-server"
        request_bytes = b"\x0a" + bytes([len(service_name)]) + service_name

        try:
            response_bytes = grpc_config_channel.unary_unary(
                f"/{self.SERVICE}/GetServiceConfig",
                request_serializer=_identity,
                response_deserializer=_identity,
            )(request_bytes, timeout=5)
        except grpc.RpcError as e:
            if e.code() == grpc.StatusCode.UNIMPLEMENTED:
                pytest.skip("GetServiceConfig RPC is not implemented yet")
            raise

        assert isinstance(response_bytes, bytes)

    def test_update_config_rpc(self, grpc_config_channel):
        """UpdateConfig RPC が正常に応答する。"""
        # UpdateConfigRequest: namespace=1, key=2, value=3
        namespace = b"e2e-test"
        key = b"grpc-test-key"
        value = b"grpc-test-value"
        request_bytes = (
            b"\x0a"
            + bytes([len(namespace)])
            + namespace
            + b"\x12"
            + bytes([len(key)])
            + key
            + b"\x1a"
            + bytes([len(value)])
            + value
        )

        try:
            response_bytes = grpc_config_channel.unary_unary(
                f"/{self.SERVICE}/UpdateConfig",
                request_serializer=_identity,
                response_deserializer=_identity,
            )(request_bytes, timeout=5)
        except grpc.RpcError as e:
            if e.code() == grpc.StatusCode.UNIMPLEMENTED:
                pytest.skip("UpdateConfig RPC is not implemented yet")
            raise

        assert isinstance(response_bytes, bytes)

    def test_delete_config_rpc(self, grpc_config_channel):
        """DeleteConfig RPC が正常に応答する。"""
        # DeleteConfigRequest: namespace=1, key=2
        namespace = b"e2e-test"
        key = b"grpc-test-key"
        request_bytes = (
            b"\x0a" + bytes([len(namespace)]) + namespace + b"\x12" + bytes([len(key)]) + key
        )

        try:
            response_bytes = grpc_config_channel.unary_unary(
                f"/{self.SERVICE}/DeleteConfig",
                request_serializer=_identity,
                response_deserializer=_identity,
            )(request_bytes, timeout=5)
        except grpc.RpcError as e:
            if e.code() == grpc.StatusCode.UNIMPLEMENTED:
                pytest.skip("DeleteConfig RPC is not implemented yet")
            if e.code() == grpc.StatusCode.NOT_FOUND:
                return
            raise

        assert isinstance(response_bytes, bytes)
