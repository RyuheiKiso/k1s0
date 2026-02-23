"""webhook_client library unit tests."""

from k1s0_webhook_client import WebhookPayload, generate_signature, verify_signature


def test_generate_signature_deterministic() -> None:
    sig1 = generate_signature("secret", b"body")
    sig2 = generate_signature("secret", b"body")
    assert sig1 == sig2


def test_generate_signature_different_secrets() -> None:
    sig1 = generate_signature("secret1", b"body")
    sig2 = generate_signature("secret2", b"body")
    assert sig1 != sig2


def test_verify_signature_success() -> None:
    sig = generate_signature("my-secret", b'{"event":"test"}')
    assert verify_signature("my-secret", b'{"event":"test"}', sig) is True


def test_verify_signature_failure() -> None:
    sig = generate_signature("my-secret", b'{"event":"test"}')
    assert verify_signature("my-secret", b"tampered", sig) is False


def test_verify_signature_wrong_secret() -> None:
    sig = generate_signature("correct-secret", b"body")
    assert verify_signature("wrong-secret", b"body", sig) is False


def test_webhook_payload_defaults() -> None:
    payload = WebhookPayload(event_type="user.created", timestamp="2024-01-01T00:00:00Z")
    assert payload.data == {}
    assert payload.event_type == "user.created"
