"""System API E2E テスト共通設定。"""
import os

import pytest
import requests


@pytest.fixture(scope="session")
def auth_base_url():
    return os.environ.get("AUTH_BASE_URL", "http://localhost:8080")


@pytest.fixture(scope="session")
def config_base_url():
    return os.environ.get("CONFIG_BASE_URL", "http://localhost:8082")


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
