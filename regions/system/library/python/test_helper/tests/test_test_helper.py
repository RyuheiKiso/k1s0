"""test-helper ライブラリのテスト。"""

import pytest

from k1s0_test_helper import (
    JwtTestHelper,
    TestClaims,
    MockServerBuilder,
    FixtureBuilder,
    AssertionHelper,
)
from k1s0_test_helper.assertion import AssertionError as TestAssertionError


class TestJwtTestHelper:
    def setup_method(self) -> None:
        self.helper = JwtTestHelper(secret="test-secret")

    def test_create_admin_token(self) -> None:
        token = self.helper.create_admin_token()
        parts = token.split(".")
        assert len(parts) == 3
        claims = self.helper.decode_claims(token)
        assert claims is not None
        assert claims.sub == "admin"
        assert claims.roles == ["admin"]

    def test_create_user_token(self) -> None:
        token = self.helper.create_user_token("user-123", ["user"])
        claims = self.helper.decode_claims(token)
        assert claims is not None
        assert claims.sub == "user-123"
        assert claims.roles == ["user"]

    def test_create_token_with_tenant(self) -> None:
        token = self.helper.create_token(
            TestClaims(sub="svc", roles=["service"], tenant_id="t-1")
        )
        claims = self.helper.decode_claims(token)
        assert claims is not None
        assert claims.tenant_id == "t-1"

    def test_decode_invalid_token(self) -> None:
        assert self.helper.decode_claims("invalid") is None


class TestMockServerBuilder:
    def test_notification_server(self) -> None:
        server = (
            MockServerBuilder.notification_server()
            .with_health_ok()
            .with_success_response("/send", '{"id":"1","status":"sent"}')
            .build()
        )
        health = server.handle("GET", "/health")
        assert health is not None
        assert health[0] == 200
        assert "ok" in health[1]

        send = server.handle("POST", "/send")
        assert send is not None
        assert send[0] == 200

        assert server.request_count == 2

    def test_unknown_route(self) -> None:
        server = MockServerBuilder.ratelimit_server().with_health_ok().build()
        assert server.handle("GET", "/nonexistent") is None

    def test_error_response(self) -> None:
        server = (
            MockServerBuilder.tenant_server()
            .with_error_response("/create", 500)
            .build()
        )
        result = server.handle("POST", "/create")
        assert result is not None
        assert result[0] == 500
        assert "error" in result[1]


class TestFixtureBuilder:
    def test_uuid(self) -> None:
        uid = FixtureBuilder.uuid()
        assert len(uid) == 36
        assert "-" in uid

    def test_email(self) -> None:
        email = FixtureBuilder.email()
        assert "@example.com" in email

    def test_name(self) -> None:
        name = FixtureBuilder.name()
        assert name.startswith("user-")

    def test_int_value_in_range(self) -> None:
        for _ in range(100):
            val = FixtureBuilder.int_value(10, 20)
            assert 10 <= val < 20

    def test_int_value_same_min_max(self) -> None:
        assert FixtureBuilder.int_value(5, 5) == 5

    def test_tenant_id(self) -> None:
        tid = FixtureBuilder.tenant_id()
        assert tid.startswith("tenant-")

    def test_uniqueness(self) -> None:
        a = FixtureBuilder.uuid()
        b = FixtureBuilder.uuid()
        assert a != b


class TestAssertionHelper:
    def test_json_contains_partial_match(self) -> None:
        AssertionHelper.assert_json_contains(
            {"id": "1", "status": "ok", "extra": "ignored"},
            {"id": "1", "status": "ok"},
        )

    def test_json_contains_nested(self) -> None:
        AssertionHelper.assert_json_contains(
            {"user": {"id": "1", "name": "test"}, "status": "ok"},
            {"user": {"id": "1"}},
        )

    def test_json_contains_mismatch(self) -> None:
        with pytest.raises(TestAssertionError, match="JSON partial match failed"):
            AssertionHelper.assert_json_contains({"id": "1"}, {"id": "2"})

    def test_event_emitted(self) -> None:
        events = [
            {"type": "created", "id": "1"},
            {"type": "updated", "id": "2"},
        ]
        AssertionHelper.assert_event_emitted(events, "created")
        AssertionHelper.assert_event_emitted(events, "updated")

    def test_event_not_emitted(self) -> None:
        with pytest.raises(TestAssertionError, match="not found"):
            AssertionHelper.assert_event_emitted([{"type": "created"}], "deleted")
