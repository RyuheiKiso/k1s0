import Testing
import Foundation
@testable import K1s0EventBus

@Suite("EventBus Tests")
struct EventBusTests {
    @Test("イベントをpublishしてhandlerが呼ばれること")
    func testPublish() async throws {
        let bus = InMemoryEventBus()
        let received = SendableBox<Bool>(false)
        await bus.subscribe("user.created") { _ in
            await received.set(true)
        }
        let event = Event(eventType: "user.created", payload: ["id": "123"])
        try await bus.publish(event)
        let value = await received.get()
        #expect(value)
    }

    @Test("異なるイベントタイプのhandlerは呼ばれないこと")
    func testNoMatchingHandler() async throws {
        let bus = InMemoryEventBus()
        let received = SendableBox<Bool>(false)
        await bus.subscribe("user.created") { _ in
            await received.set(true)
        }
        let event = Event(eventType: "user.deleted", payload: [:])
        try await bus.publish(event)
        let value = await received.get()
        #expect(!value)
    }

    @Test("unsubscribeでhandlerが削除されること")
    func testUnsubscribe() async throws {
        let bus = InMemoryEventBus()
        let count = SendableBox<Int>(0)
        await bus.subscribe("test") { _ in
            let current = await count.get()
            await count.set(current + 1)
        }
        try await bus.publish(Event(eventType: "test", payload: [:]))
        await bus.unsubscribe("test")
        try await bus.publish(Event(eventType: "test", payload: [:]))
        let value = await count.get()
        #expect(value == 1)
    }

    @Test("Eventにidとtimestampが自動設定されること")
    func testEventAutoFields() {
        let event = Event(eventType: "test", payload: ["key": "value"])
        #expect(!event.id.isEmpty)
        #expect(event.timestamp.timeIntervalSinceNow < 1.0)
        #expect(event.payload["key"] == "value")
    }

    @Test("複数のhandlerが同一イベントタイプに登録できること")
    func testMultipleHandlers() async throws {
        let bus = InMemoryEventBus()
        let count = SendableBox<Int>(0)
        await bus.subscribe("multi") { _ in
            let current = await count.get()
            await count.set(current + 1)
        }
        await bus.subscribe("multi") { _ in
            let current = await count.get()
            await count.set(current + 1)
        }
        try await bus.publish(Event(eventType: "multi", payload: [:]))
        let value = await count.get()
        #expect(value == 2)
    }
}

actor SendableBox<T: Sendable> {
    private var value: T
    init(_ value: T) { self.value = value }
    func get() -> T { value }
    func set(_ newValue: T) { value = newValue }
}
