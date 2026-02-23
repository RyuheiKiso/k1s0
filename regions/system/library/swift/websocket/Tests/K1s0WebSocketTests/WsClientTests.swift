import Testing
import Foundation
@testable import K1s0WebSocket

@Suite("WsClient Tests")
struct WsClientTests {
    @Test("接続と切断ができること")
    func testConnectAndDisconnect() async throws {
        let client = InMemoryWsClient()
        #expect(await client.state == .disconnected)

        try await client.connect()
        #expect(await client.state == .connected)

        try await client.disconnect()
        #expect(await client.state == .disconnected)
    }

    @Test("メッセージの送受信ができること")
    func testSendAndReceive() async throws {
        let client = InMemoryWsClient()
        try await client.connect()

        await client.injectMessage(.text("hello"))
        let msg = try await client.receive()
        #expect(msg.type == .text)
        #expect(msg.textValue == "hello")

        try await client.send(.text("world"))
        let sent = await client.getSentMessages()
        #expect(sent.count == 1)
    }

    @Test("未接続で送信するとエラーになること")
    func testSendWhileDisconnected() async throws {
        let client = InMemoryWsClient()
        do {
            try await client.send(.text("hello"))
            #expect(Bool(false), "Should have thrown")
        } catch is WsError {
            // expected
        }
    }

    @Test("未接続で受信するとエラーになること")
    func testReceiveWhileDisconnected() async throws {
        let client = InMemoryWsClient()
        do {
            _ = try await client.receive()
            #expect(Bool(false), "Should have thrown")
        } catch is WsError {
            // expected
        }
    }

    @Test("バッファが空で受信するとエラーになること")
    func testReceiveEmptyBuffer() async throws {
        let client = InMemoryWsClient()
        try await client.connect()
        do {
            _ = try await client.receive()
            #expect(Bool(false), "Should have thrown")
        } catch is WsError {
            // expected
        }
    }

    @Test("二重接続でエラーになること")
    func testDoubleConnect() async throws {
        let client = InMemoryWsClient()
        try await client.connect()
        do {
            try await client.connect()
            #expect(Bool(false), "Should have thrown")
        } catch is WsError {
            // expected
        }
    }

    @Test("テキストメッセージのファクトリメソッド")
    func testTextFactory() {
        let msg = WsMessage.text("hello")
        #expect(msg.type == .text)
        #expect(msg.textValue == "hello")
    }

    @Test("バイナリメッセージのファクトリメソッド")
    func testBinaryFactory() {
        let data = Data([0x01, 0x02, 0x03])
        let msg = WsMessage.binary(data)
        #expect(msg.type == .binary)
        #expect(msg.payload == data)
        #expect(msg.textValue == nil)
    }

    @Test("WsConfigのデフォルト値")
    func testConfigDefaults() {
        let config = WsConfig.defaults
        #expect(config.url == "ws://localhost")
        #expect(config.reconnect == true)
        #expect(config.maxReconnectAttempts == 5)
        #expect(config.pingInterval == nil)
    }

    @Test("ConnectionStateの全バリアント")
    func testConnectionStates() {
        let states: [ConnectionState] = [.disconnected, .connecting, .connected, .reconnecting, .closing]
        #expect(states.count == 5)
    }

    @Test("WsErrorの各バリアント")
    func testWsErrors() {
        let err1 = WsError.notConnected
        if case .notConnected = err1 { } else {
            #expect(Bool(false), "Should be notConnected")
        }

        let err2 = WsError.connectionFailed(reason: "timeout")
        if case .connectionFailed(let reason) = err2 {
            #expect(reason == "timeout")
        }
    }
}
