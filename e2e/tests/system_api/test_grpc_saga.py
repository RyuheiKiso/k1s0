"""gRPC saga-service E2E テスト。"""
import pytest

try:
    import grpc
except ImportError:
    grpc = None


def _identity(x):
    return x


class TestGrpcSagaService:
    """gRPC SagaService の E2E テスト。"""

    SERVICE = "k1s0.system.saga.v1.SagaService"

    def test_start_saga_rpc(self, grpc_saga_channel):
        """StartSaga RPC が正常に応答する。"""
        # StartSagaRequest: workflow_name=1
        workflow_name = b"test-workflow"
        request_bytes = b"\x0a" + bytes([len(workflow_name)]) + workflow_name

        try:
            response_bytes = grpc_saga_channel.unary_unary(
                f"/{self.SERVICE}/StartSaga",
                request_serializer=_identity,
                response_deserializer=_identity,
            )(request_bytes, timeout=5)
        except grpc.RpcError as e:
            if e.code() == grpc.StatusCode.UNIMPLEMENTED:
                pytest.skip("StartSaga RPC is not implemented yet")
            raise

        assert isinstance(response_bytes, bytes)
        assert len(response_bytes) > 0

    def test_get_saga_rpc(self, grpc_saga_channel):
        """GetSaga RPC が正常に応答する。"""
        # GetSagaRequest: saga_id=1
        saga_id = b"00000000-0000-0000-0000-000000000001"
        request_bytes = b"\x0a" + bytes([len(saga_id)]) + saga_id

        try:
            response_bytes = grpc_saga_channel.unary_unary(
                f"/{self.SERVICE}/GetSaga",
                request_serializer=_identity,
                response_deserializer=_identity,
            )(request_bytes, timeout=5)
        except grpc.RpcError as e:
            if e.code() == grpc.StatusCode.UNIMPLEMENTED:
                pytest.skip("GetSaga RPC is not implemented yet")
            if e.code() == grpc.StatusCode.NOT_FOUND:
                return  # Saga が見つからないのは正常な応答
            raise

        assert isinstance(response_bytes, bytes)

    def test_list_sagas_rpc(self, grpc_saga_channel):
        """ListSagas RPC が正常に応答する。"""
        # 空の ListSagasRequest
        request_bytes = b""

        try:
            response_bytes = grpc_saga_channel.unary_unary(
                f"/{self.SERVICE}/ListSagas",
                request_serializer=_identity,
                response_deserializer=_identity,
            )(request_bytes, timeout=5)
        except grpc.RpcError as e:
            if e.code() == grpc.StatusCode.UNIMPLEMENTED:
                pytest.skip("ListSagas RPC is not implemented yet")
            raise

        assert isinstance(response_bytes, bytes)

    def test_cancel_saga_rpc(self, grpc_saga_channel):
        """CancelSaga RPC が正常に応答する。"""
        # CancelSagaRequest: saga_id=1
        saga_id = b"00000000-0000-0000-0000-000000000001"
        request_bytes = b"\x0a" + bytes([len(saga_id)]) + saga_id

        try:
            response_bytes = grpc_saga_channel.unary_unary(
                f"/{self.SERVICE}/CancelSaga",
                request_serializer=_identity,
                response_deserializer=_identity,
            )(request_bytes, timeout=5)
        except grpc.RpcError as e:
            if e.code() == grpc.StatusCode.UNIMPLEMENTED:
                pytest.skip("CancelSaga RPC is not implemented yet")
            if e.code() == grpc.StatusCode.NOT_FOUND:
                return  # 対象 Saga が存在しないのは正常な応答
            raise

        assert isinstance(response_bytes, bytes)


class TestGrpcWorkflowService:
    """gRPC SagaService の WorkflowService 相当 RPC の E2E テスト。"""

    SERVICE = "k1s0.system.saga.v1.SagaService"

    def test_register_workflow_rpc(self, grpc_saga_channel):
        """RegisterWorkflow RPC が正常に応答する。"""
        # RegisterWorkflowRequest: workflow_yaml=1
        workflow_yaml = b"name: test\nsteps: []"
        request_bytes = b"\x0a" + bytes([len(workflow_yaml)]) + workflow_yaml

        try:
            response_bytes = grpc_saga_channel.unary_unary(
                f"/{self.SERVICE}/RegisterWorkflow",
                request_serializer=_identity,
                response_deserializer=_identity,
            )(request_bytes, timeout=5)
        except grpc.RpcError as e:
            if e.code() == grpc.StatusCode.UNIMPLEMENTED:
                pytest.skip("RegisterWorkflow RPC is not implemented yet")
            raise

        assert isinstance(response_bytes, bytes)

    def test_list_workflows_rpc(self, grpc_saga_channel):
        """ListWorkflows RPC が正常に応答する。"""
        # 空の ListWorkflowsRequest
        request_bytes = b""

        try:
            response_bytes = grpc_saga_channel.unary_unary(
                f"/{self.SERVICE}/ListWorkflows",
                request_serializer=_identity,
                response_deserializer=_identity,
            )(request_bytes, timeout=5)
        except grpc.RpcError as e:
            if e.code() == grpc.StatusCode.UNIMPLEMENTED:
                pytest.skip("ListWorkflows RPC is not implemented yet")
            raise

        assert isinstance(response_bytes, bytes)
