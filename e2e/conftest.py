"""k1s0 E2E テスト共通設定"""
import os
import pytest
import requests


@pytest.fixture(scope="session")
def base_url():
    return os.environ.get("K1S0_BASE_URL", "http://localhost:8080")


@pytest.fixture(scope="session")
def api_client(base_url):
    session = requests.Session()
    session.base_url = base_url
    session.headers.update({"Content-Type": "application/json"})
    return session


@pytest.fixture(autouse=True)
def log_test_name(request):
    print(f"\n--- {request.node.name} ---")
