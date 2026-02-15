"""E2E test configuration and shared fixtures."""

from typing import Generator

import pytest
import requests


@pytest.fixture(scope="session")
def base_url() -> str:
    """Base URL for the API under test."""
    return "http://localhost:8080"


@pytest.fixture(scope="session")
def api_session(base_url: str) -> Generator[requests.Session, None, None]:
    """Authenticated HTTP session for API requests."""
    session = requests.Session()
    session.headers.update(
        {
            "Content-Type": "application/json",
            "Accept": "application/json",
        }
    )
    yield session
    session.close()


@pytest.fixture(scope="session")
def keycloak_token(base_url: str) -> str:
    """Obtain an access token from Keycloak for E2E tests."""
    token_url = "http://localhost:8180/realms/k1s0/protocol/openid-connect/token"
    response = requests.post(
        token_url,
        data={
            "grant_type": "client_credentials",
            "client_id": "e2e-test",
            "client_secret": "e2e-test-secret",
        },
    )
    response.raise_for_status()
    return response.json()["access_token"]


@pytest.fixture(scope="session")
def auth_session(
    api_session: requests.Session, keycloak_token: str
) -> Generator[requests.Session, None, None]:
    """Authenticated HTTP session with Keycloak token."""
    api_session.headers.update({"Authorization": f"Bearer {keycloak_token}"})
    yield api_session
