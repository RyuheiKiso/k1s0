"""notification_client library unit tests."""

from k1s0_notification_client import (
    InMemoryNotificationClient,
    NotificationChannel,
    NotificationRequest,
)


async def test_send_email() -> None:
    client = InMemoryNotificationClient()
    req = NotificationRequest(
        channel=NotificationChannel.EMAIL,
        recipient="user@example.com",
        body="Hello",
        subject="Test",
    )
    resp = await client.send(req)
    assert resp.status == "sent"
    assert resp.id == req.id


async def test_sent_list() -> None:
    client = InMemoryNotificationClient()
    req = NotificationRequest(
        channel=NotificationChannel.SMS,
        recipient="+1234567890",
        body="OTP: 123456",
    )
    await client.send(req)
    assert len(client.sent) == 1
    assert client.sent[0].channel == NotificationChannel.SMS


async def test_multiple_sends() -> None:
    client = InMemoryNotificationClient()
    for ch in NotificationChannel:
        await client.send(
            NotificationRequest(channel=ch, recipient="test", body="msg")
        )
    assert len(client.sent) == 4


async def test_sent_returns_copy() -> None:
    client = InMemoryNotificationClient()
    await client.send(
        NotificationRequest(
            channel=NotificationChannel.PUSH,
            recipient="device-1",
            body="ping",
        )
    )
    sent1 = client.sent
    sent2 = client.sent
    assert sent1 is not sent2
    assert sent1 == sent2


async def test_request_has_id() -> None:
    req = NotificationRequest(
        channel=NotificationChannel.EMAIL,
        recipient="test@test.com",
        body="hello",
    )
    assert req.id
