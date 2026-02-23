/// アウトボックスストアプロトコル。
public protocol OutboxStore: Sendable {
    func save(_ message: OutboxMessage) async throws
    func fetchPending(limit: Int) async throws -> [OutboxMessage]
    func update(_ message: OutboxMessage) async throws
    func deleteDelivered(olderThanDays: Int) async throws -> Int
}

/// アウトボックスパブリッシャープロトコル。
public protocol OutboxPublisher: Sendable {
    func publish(_ message: OutboxMessage) async throws
}

/// アウトボックスプロセッサー。
public actor OutboxProcessor: Sendable {
    private let store: any OutboxStore
    private let publisher: any OutboxPublisher
    private let batchSize: Int

    public init(store: any OutboxStore, publisher: any OutboxPublisher, batchSize: Int = 100) {
        self.store = store
        self.publisher = publisher
        self.batchSize = batchSize
    }

    /// バッチ処理を実行し、処理件数を返す。
    public func processBatch() async throws -> Int {
        let messages = try await store.fetchPending(limit: batchSize)
        var processed = 0
        for var message in messages {
            message.markProcessing()
            try await store.update(message)
            do {
                try await publisher.publish(message)
                message.markDelivered()
            } catch {
                message.markFailed(error: error.localizedDescription)
            }
            try await store.update(message)
            processed += 1
        }
        return processed
    }
}
