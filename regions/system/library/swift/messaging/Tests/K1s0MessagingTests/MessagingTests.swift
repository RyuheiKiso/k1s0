import Testing
@testable import K1s0Messaging

@Suite("Messaging Tests")
struct MessagingTests {
    @Test("EventMetadata が生成されること")
    func testEventMetadataCreation() {
        let meta = EventMetadata(eventType: "order.created", source: "order-service")
        #expect(!meta.eventId.isEmpty)
        #expect(meta.eventType == "order.created")
        #expect(meta.schemaVersion == 1)
        #expect(meta.traceId == nil)
    }

    @Test("TraceId 付き EventMetadata が生成されること")
    func testEventMetadataWithTraceId() {
        let meta = EventMetadata(eventType: "order.created", source: "order-service")
            .withTraceId("trace-123")
        #expect(meta.traceId == "trace-123")
    }

    @Test("JSON エンベロープが生成されること")
    func testJsonEnvelope() throws {
        let envelope = try EventEnvelope.json(
            topic: "k1s0.service.orders.order-created.v1",
            key: "order-1",
            payload: ["id": "order-1"]
        )
        #expect(envelope.topic == "k1s0.service.orders.order-created.v1")
        #expect(!envelope.payload.isEmpty)
    }

    @Test("ConsumedMessage が JSON デシリアライズできること")
    func testConsumedMessageDeserialize() throws {
        struct Payload: Codable { let id: String }
        let data = try JSONEncoder().encode(Payload(id: "msg-1"))
        let msg = ConsumedMessage(topic: "test", partition: 0, offset: 0, key: nil, payload: data)
        let decoded = try msg.deserializeJSON(as: Payload.self)
        #expect(decoded.id == "msg-1")
    }

    @Test("NoOpEventProducer が正常に動作すること")
    func testNoOpProducer() async throws {
        let producer = NoOpEventProducer()
        let envelope = try EventEnvelope.json(topic: "test", key: "key", payload: ["x": 1])
        try await producer.publish(envelope)
    }

    @Test("MessagingError の説明が含まれること")
    func testMessagingErrorDescription() {
        let error = MessagingError.publishError("connection refused")
        #expect(error.description.contains("PUBLISH_ERROR"))
    }
}
