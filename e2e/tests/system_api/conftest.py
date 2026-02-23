"""System API E2E テスト共通設定。"""

import os

import pytest
import requests

try:
    import grpc
except ImportError:
    grpc = None

try:
    from confluent_kafka import Consumer as _KafkaConsumer
except ImportError:
    _KafkaConsumer = None


@pytest.fixture(scope="session")
def auth_base_url():
    return os.environ.get("AUTH_BASE_URL", "http://localhost:8080")


@pytest.fixture(scope="session")
def config_base_url():
    return os.environ.get("CONFIG_BASE_URL", "http://localhost:8082")


@pytest.fixture(scope="session")
def keycloak_base_url():
    return os.environ.get("KEYCLOAK_BASE_URL", "http://localhost:8180")


@pytest.fixture(scope="session")
def auth_client(auth_base_url):
    session = requests.Session()
    session.base_url = auth_base_url
    session.headers.update({"Content-Type": "application/json"})
    return session


@pytest.fixture(scope="session")
def config_client(config_base_url):
    session = requests.Session()
    session.base_url = config_base_url
    session.headers.update({"Content-Type": "application/json"})
    return session


@pytest.fixture(scope="session")
def kong_proxy_url():
    return os.environ.get("KONG_PROXY_URL", "http://localhost:8000")


@pytest.fixture(scope="session")
def kong_admin_url():
    return os.environ.get("KONG_ADMIN_URL", "http://localhost:8001")


@pytest.fixture(scope="session")
def kong_client(kong_proxy_url):
    session = requests.Session()
    session.base_url = kong_proxy_url
    session.headers.update({"Content-Type": "application/json"})
    return session


# Keycloak E2E test client credentials (dev environment only)
KEYCLOAK_E2E_CLIENT_ID = "k1s0-e2e-test"
KEYCLOAK_E2E_CLIENT_SECRET = "dev-e2e-test-secret"
KEYCLOAK_REALM = "k1s0"


def _keycloak_token_url(base_url):
    return f"{base_url}/realms/{KEYCLOAK_REALM}/protocol/openid-connect/token"


def _keycloak_userinfo_url(base_url):
    return f"{base_url}/realms/{KEYCLOAK_REALM}/protocol/openid-connect/userinfo"


def _obtain_token(base_url, username, password):
    """Resource Owner Password Credentials でトークンを取得する。"""
    resp = requests.post(
        _keycloak_token_url(base_url),
        data={
            "grant_type": "password",
            "client_id": KEYCLOAK_E2E_CLIENT_ID,
            "client_secret": KEYCLOAK_E2E_CLIENT_SECRET,
            "username": username,
            "password": password,
            "scope": "openid profile email",
        },
        timeout=10,
    )
    resp.raise_for_status()
    return resp.json()


@pytest.fixture(scope="session")
def keycloak_admin_token(keycloak_base_url):
    """test-admin ユーザーの access_token を返す。Keycloak 未起動時は skip。"""
    try:
        return _obtain_token(keycloak_base_url, "test-admin", "admin123")
    except requests.ConnectionError:
        pytest.skip("Keycloak is not running")
    except requests.HTTPError as exc:
        pytest.skip(f"Keycloak token request failed: {exc}")


@pytest.fixture(scope="session")
def keycloak_user_token(keycloak_base_url):
    """test-user ユーザーの access_token を返す。Keycloak 未起動時は skip。"""
    try:
        return _obtain_token(keycloak_base_url, "test-user", "user123")
    except requests.ConnectionError:
        pytest.skip("Keycloak is not running")
    except requests.HTTPError as exc:
        pytest.skip(f"Keycloak token request failed: {exc}")


# --- gRPC ---


@pytest.fixture(scope="session")
def grpc_auth_channel():
    """gRPC auth-server チャネル。grpcio 未インストール or 接続不可なら skip。"""
    if grpc is None:
        pytest.skip("grpcio is not installed")
    url = os.environ.get("GRPC_AUTH_URL", "localhost:50051")
    channel = grpc.insecure_channel(url)
    try:
        grpc.channel_ready_future(channel).result(timeout=3)
    except grpc.FutureTimeoutError:
        pytest.skip(f"gRPC auth-server is not reachable at {url}")
    yield channel
    channel.close()


@pytest.fixture(scope="session")
def grpc_config_channel():
    """gRPC config-server チャネル。grpcio 未インストール or 接続不可なら skip。"""
    if grpc is None:
        pytest.skip("grpcio is not installed")
    url = os.environ.get("GRPC_CONFIG_URL", "localhost:50052")
    channel = grpc.insecure_channel(url)
    try:
        grpc.channel_ready_future(channel).result(timeout=3)
    except grpc.FutureTimeoutError:
        pytest.skip(f"gRPC config-server is not reachable at {url}")
    yield channel
    channel.close()


@pytest.fixture(scope="session")
def saga_base_url():
    return os.environ.get("SAGA_SERVER_URL", "http://localhost:8083")


@pytest.fixture(scope="session")
def saga_client(saga_base_url):
    session = requests.Session()
    session.base_url = saga_base_url
    session.headers.update({"Content-Type": "application/json"})
    return session


@pytest.fixture(scope="session")
def grpc_saga_channel():
    """gRPC saga-server チャネル。grpcio 未インストール or 接続不可なら skip。"""
    if grpc is None:
        pytest.skip("grpcio is not installed")
    url = os.environ.get("GRPC_SAGA_URL", "localhost:50053")
    channel = grpc.insecure_channel(url)
    try:
        grpc.channel_ready_future(channel).result(timeout=3)
    except grpc.FutureTimeoutError:
        pytest.skip(f"gRPC saga-server is not reachable at {url}")
    yield channel
    channel.close()


# --- Kafka ---


@pytest.fixture(scope="session")
def kafka_bootstrap_servers():
    """Kafka ブローカーアドレス。confluent-kafka 未インストール or 接続不可なら skip。"""
    if _KafkaConsumer is None:
        pytest.skip("confluent-kafka is not installed")
    servers = os.environ.get("KAFKA_BROKERS", "localhost:9092")
    consumer = _KafkaConsumer(
        {
            "bootstrap.servers": servers,
            "group.id": "e2e-health-check",
            "session.timeout.ms": 5000,
        }
    )
    try:
        metadata = consumer.list_topics(timeout=5)
        if metadata is None:
            pytest.skip(f"Kafka is not reachable at {servers}")
    except Exception:
        pytest.skip(f"Kafka is not reachable at {servers}")
    finally:
        consumer.close()
    return servers


@pytest.fixture(scope="session")
def dlq_base_url():
    return os.environ.get("DLQ_SERVER_URL", "http://localhost:8084")


@pytest.fixture(scope="session")
def dlq_client(dlq_base_url):
    session = requests.Session()
    session.base_url = dlq_base_url
    session.headers.update({"Content-Type": "application/json"})
    return session
