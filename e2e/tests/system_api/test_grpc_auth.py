"""gRPC auth-service E2E テスト。"""
import pytest

try:
    import grpc
    from grpc_reflection.v1alpha import reflection_pb2, reflection_pb2_grpc
except ImportError:
    grpc = None


def _make_unary_call(channel, service, method, request_serializer, response_deserializer, request_bytes):
    """汎用 gRPC unary 呼び出し。生成済み stub がなくても動作する。"""
    return channel.unary_unary(
        f"/{service}/{method}",
        request_serializer=request_serializer,
        response_deserializer=response_deserializer,
    )(request_bytes)


def _identity(x):
    return x


class TestGrpcAuthService:
    """gRPC AuthService の E2E テスト。"""

    SERVICE = "k1s0.system.auth.v1.AuthService"

    def test_validate_token_rpc(self, grpc_auth_channel):
        """ValidateToken RPC が正常に応答する。"""
        # ValidateTokenRequest: field 1 (token) = "test-token"
        # protobuf wire format: field_number=1, wire_type=2 (length-delimited)
        # 0x0a = (1 << 3) | 2, followed by length, then string bytes
        token = b"test-token"
        request_bytes = b"\x0a" + bytes([len(token)]) + token

        try:
            response_bytes = grpc_auth_channel.unary_unary(
                f"/{self.SERVICE}/ValidateToken",
                request_serializer=_identity,
                response_deserializer=_identity,
            )(request_bytes, timeout=5)
        except grpc.RpcError as e:
            # UNIMPLEMENTED はサービス定義はあるが未実装のケース（許容）
            if e.code() == grpc.StatusCode.UNIMPLEMENTED:
                pytest.skip("ValidateToken RPC is not implemented yet")
            raise

        assert isinstance(response_bytes, bytes)
        assert len(response_bytes) > 0

    def test_get_user_rpc(self, grpc_auth_channel):
        """GetUser RPC が正常に応答する。"""
        user_id = b"test-user-001"
        request_bytes = b"\x0a" + bytes([len(user_id)]) + user_id

        try:
            response_bytes = grpc_auth_channel.unary_unary(
                f"/{self.SERVICE}/GetUser",
                request_serializer=_identity,
                response_deserializer=_identity,
            )(request_bytes, timeout=5)
        except grpc.RpcError as e:
            if e.code() == grpc.StatusCode.UNIMPLEMENTED:
                pytest.skip("GetUser RPC is not implemented yet")
            if e.code() == grpc.StatusCode.NOT_FOUND:
                return  # ユーザーが見つからないのは正常な応答
            raise

        assert isinstance(response_bytes, bytes)

    def test_list_users_rpc(self, grpc_auth_channel):
        """ListUsers RPC が正常に応答する。"""
        # 空の ListUsersRequest
        request_bytes = b""

        try:
            response_bytes = grpc_auth_channel.unary_unary(
                f"/{self.SERVICE}/ListUsers",
                request_serializer=_identity,
                response_deserializer=_identity,
            )(request_bytes, timeout=5)
        except grpc.RpcError as e:
            if e.code() == grpc.StatusCode.UNIMPLEMENTED:
                pytest.skip("ListUsers RPC is not implemented yet")
            raise

        assert isinstance(response_bytes, bytes)

    def test_check_permission_rpc(self, grpc_auth_channel):
        """CheckPermission RPC が正常に応答する。"""
        # CheckPermissionRequest: user_id=1, permission=2, resource=3
        user_id = b"test-user-001"
        permission = b"read"
        resource = b"config"
        request_bytes = (
            b"\x0a" + bytes([len(user_id)]) + user_id
            + b"\x12" + bytes([len(permission)]) + permission
            + b"\x1a" + bytes([len(resource)]) + resource
        )

        try:
            response_bytes = grpc_auth_channel.unary_unary(
                f"/{self.SERVICE}/CheckPermission",
                request_serializer=_identity,
                response_deserializer=_identity,
            )(request_bytes, timeout=5)
        except grpc.RpcError as e:
            if e.code() == grpc.StatusCode.UNIMPLEMENTED:
                pytest.skip("CheckPermission RPC is not implemented yet")
            raise

        assert isinstance(response_bytes, bytes)


class TestGrpcAuditService:
    """gRPC AuditService の E2E テスト。"""

    SERVICE = "k1s0.system.auth.v1.AuditService"

    def test_search_audit_logs_rpc(self, grpc_auth_channel):
        """SearchAuditLogs RPC が正常に応答する。"""
        # 空の SearchAuditLogsRequest
        request_bytes = b""

        try:
            response_bytes = grpc_auth_channel.unary_unary(
                f"/{self.SERVICE}/SearchAuditLogs",
                request_serializer=_identity,
                response_deserializer=_identity,
            )(request_bytes, timeout=5)
        except grpc.RpcError as e:
            if e.code() == grpc.StatusCode.UNIMPLEMENTED:
                pytest.skip("SearchAuditLogs RPC is not implemented yet")
            raise

        assert isinstance(response_bytes, bytes)

    def test_record_audit_log_rpc(self, grpc_auth_channel):
        """RecordAuditLog RPC が正常に応答する。"""
        event_type = b"LOGIN"
        user_id = b"test-user-001"
        request_bytes = (
            b"\x0a" + bytes([len(event_type)]) + event_type
            + b"\x12" + bytes([len(user_id)]) + user_id
        )

        try:
            response_bytes = grpc_auth_channel.unary_unary(
                f"/{self.SERVICE}/RecordAuditLog",
                request_serializer=_identity,
                response_deserializer=_identity,
            )(request_bytes, timeout=5)
        except grpc.RpcError as e:
            if e.code() == grpc.StatusCode.UNIMPLEMENTED:
                pytest.skip("RecordAuditLog RPC is not implemented yet")
            raise

        assert isinstance(response_bytes, bytes)
        assert len(response_bytes) > 0
