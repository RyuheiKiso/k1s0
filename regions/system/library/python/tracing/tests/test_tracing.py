"""tracing library unit tests."""

from k1s0_tracing import Baggage, TraceContext, extract_context, inject_context


async def test_to_traceparent() -> None:
    ctx = TraceContext(
        trace_id="0af7651916cd43dd8448eb211c80319c",
        parent_id="b7ad6b7169203331",
        flags=1,
    )
    header = ctx.to_traceparent()
    assert header == "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01"


async def test_from_traceparent() -> None:
    ctx = TraceContext.from_traceparent(
        "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01"
    )
    assert ctx is not None
    assert ctx.trace_id == "0af7651916cd43dd8448eb211c80319c"
    assert ctx.parent_id == "b7ad6b7169203331"
    assert ctx.flags == 1


async def test_invalid_traceparent() -> None:
    assert TraceContext.from_traceparent("invalid") is None
    assert TraceContext.from_traceparent("01-abc-def-00") is None
    assert TraceContext.from_traceparent("") is None


async def test_baggage_set_get() -> None:
    baggage = Baggage()
    baggage.set("key1", "value1")
    baggage.set("key2", "value2")
    assert baggage.get("key1") == "value1"
    assert baggage.get("key2") == "value2"
    assert baggage.get("key3") is None


async def test_baggage_to_header() -> None:
    baggage = Baggage()
    baggage.set("key1", "value1")
    header = baggage.to_header()
    assert "key1=value1" in header


async def test_baggage_from_header() -> None:
    baggage = Baggage.from_header("key1=value1,key2=value2")
    assert baggage.get("key1") == "value1"
    assert baggage.get("key2") == "value2"


async def test_baggage_from_empty_header() -> None:
    baggage = Baggage.from_header("")
    assert baggage.is_empty


async def test_inject_context() -> None:
    headers: dict[str, str] = {}
    ctx = TraceContext(
        trace_id="0af7651916cd43dd8448eb211c80319c",
        parent_id="b7ad6b7169203331",
        flags=1,
    )
    baggage = Baggage()
    baggage.set("userId", "123")

    inject_context(headers, ctx, baggage)
    assert (
        headers["traceparent"]
        == "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01"
    )
    assert "userId=123" in headers["baggage"]


async def test_extract_context() -> None:
    headers = {
        "traceparent": "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01",
        "baggage": "userId=123",
    }
    ctx, baggage = extract_context(headers)
    assert ctx is not None
    assert ctx.trace_id == "0af7651916cd43dd8448eb211c80319c"
    assert baggage.get("userId") == "123"


async def test_extract_context_empty() -> None:
    ctx, baggage = extract_context({})
    assert ctx is None
    assert baggage.is_empty


async def test_inject_without_baggage() -> None:
    headers: dict[str, str] = {}
    ctx = TraceContext(
        trace_id="0af7651916cd43dd8448eb211c80319c",
        parent_id="b7ad6b7169203331",
    )
    inject_context(headers, ctx)
    assert "traceparent" in headers
    assert "baggage" not in headers


async def test_flags_default() -> None:
    ctx = TraceContext(
        trace_id="0af7651916cd43dd8448eb211c80319c",
        parent_id="b7ad6b7169203331",
    )
    assert ctx.flags == 1


async def test_flags_zero() -> None:
    ctx = TraceContext(
        trace_id="0af7651916cd43dd8448eb211c80319c",
        parent_id="b7ad6b7169203331",
        flags=0,
    )
    assert ctx.to_traceparent().endswith("-00")
