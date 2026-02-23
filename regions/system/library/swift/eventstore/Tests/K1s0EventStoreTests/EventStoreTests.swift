import Testing
@testable import K1s0EventStore
import Foundation

@Suite("EventStore Tests")
struct EventStoreTests {
    private func makeEvent(id: String, streamId: StreamId, type: String, version: Int) -> EventEnvelope {
        EventEnvelope(
            eventId: id,
            streamId: streamId,
            eventType: type,
            payload: ["data": "value"],
            version: version
        )
    }

    @Test("イベントを追記して読み込めること")
    func testAppendAndRead() async throws {
        let store = InMemoryEventStore()
        let streamId = StreamId("order-stream-1")

        let event1 = makeEvent(id: "evt-1", streamId: streamId, type: "OrderCreated", version: 0)
        let event2 = makeEvent(id: "evt-2", streamId: streamId, type: "OrderConfirmed", version: 1)

        try await store.append(events: [event1, event2], to: streamId, expectedVersion: nil)

        let events = try await store.readStream(streamId, fromVersion: 0)
        #expect(events.count == 2)
        #expect(events[0].eventId == "evt-1")
        #expect(events[1].eventId == "evt-2")
    }

    @Test("空のストリームを読み込むと空の配列が返ること")
    func testReadEmptyStream() async throws {
        let store = InMemoryEventStore()
        let streamId = StreamId("empty-stream")

        let events = try await store.readStream(streamId, fromVersion: 0)
        #expect(events.isEmpty)
    }

    @Test("バージョン競合でエラーになること")
    func testVersionConflict() async throws {
        let store = InMemoryEventStore()
        let streamId = StreamId("conflict-stream")

        let event1 = makeEvent(id: "evt-1", streamId: streamId, type: "Created", version: 0)
        try await store.append(events: [event1], to: streamId, expectedVersion: nil)

        let event2 = makeEvent(id: "evt-2", streamId: streamId, type: "Updated", version: 1)
        do {
            try await store.append(events: [event2], to: streamId, expectedVersion: 5)
            Issue.record("バージョン競合エラーがスローされるべき")
        } catch let error as EventStoreError {
            switch error {
            case .versionConflict(let expected, let actual):
                #expect(expected == 5)
                #expect(actual == 0)
            default:
                Issue.record("versionConflict エラーが期待される")
            }
        }
    }

    @Test("イベントが連番のバージョンで追記されること")
    func testSequentialVersions() async throws {
        let store = InMemoryEventStore()
        let streamId = StreamId("versioned-stream")

        let events = (0..<5).map { i in
            makeEvent(id: "evt-\(i)", streamId: streamId, type: "Event\(i)", version: i)
        }

        try await store.append(events: events, to: streamId, expectedVersion: nil)

        let retrieved = try await store.readStream(streamId, fromVersion: 0)
        #expect(retrieved.count == 5)
        for (index, event) in retrieved.enumerated() {
            #expect(event.version == index)
        }
    }

    @Test("fromVersion を指定してイベントを絞り込めること")
    func testReadFromVersion() async throws {
        let store = InMemoryEventStore()
        let streamId = StreamId("filter-stream")

        let events = (0..<5).map { i in
            makeEvent(id: "evt-\(i)", streamId: streamId, type: "Event\(i)", version: i)
        }
        try await store.append(events: events, to: streamId, expectedVersion: nil)

        let filtered = try await store.readStream(streamId, fromVersion: 3)
        #expect(filtered.count == 2)
        #expect(filtered[0].version == 3)
        #expect(filtered[1].version == 4)
    }
}
