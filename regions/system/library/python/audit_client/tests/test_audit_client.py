"""audit_client library unit tests."""

from k1s0_audit_client import AuditEvent, BufferedAuditClient


async def test_record_event() -> None:
    client = BufferedAuditClient()
    event = AuditEvent(
        tenant_id="t1",
        actor_id="user-1",
        action="create",
        resource_type="document",
        resource_id="doc-1",
    )
    await client.record(event)
    events = await client.flush()
    assert len(events) == 1
    assert events[0].action == "create"


async def test_flush_clears_buffer() -> None:
    client = BufferedAuditClient()
    event = AuditEvent(
        tenant_id="t1",
        actor_id="user-1",
        action="delete",
        resource_type="document",
        resource_id="doc-2",
    )
    await client.record(event)
    await client.flush()
    events = await client.flush()
    assert len(events) == 0


async def test_multiple_events() -> None:
    client = BufferedAuditClient()
    for i in range(5):
        await client.record(
            AuditEvent(
                tenant_id="t1",
                actor_id="user-1",
                action=f"action-{i}",
                resource_type="item",
                resource_id=f"item-{i}",
            )
        )
    events = await client.flush()
    assert len(events) == 5


async def test_event_has_id_and_timestamp() -> None:
    event = AuditEvent(
        tenant_id="t1",
        actor_id="user-1",
        action="read",
        resource_type="doc",
        resource_id="doc-1",
    )
    assert event.id
    assert event.timestamp
