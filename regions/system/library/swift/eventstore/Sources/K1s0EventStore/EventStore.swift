import Foundation

/// イベントストアプロトコル。楽観的ロックによるバージョン管理をサポート。
public protocol EventStore: Sendable {
    /// イベントをストリームに追記する。
    /// - Parameters:
    ///   - events: 追記するイベント一覧。
    ///   - streamId: 対象ストリームID。
    ///   - expectedVersion: 期待するストリームの現在バージョン（楽観的ロック）。
    ///     nil の場合はバージョンチェックを行わない。
    func append(
        events: [EventEnvelope],
        to streamId: StreamId,
        expectedVersion: Int?
    ) async throws

    /// ストリームからイベントを読み込む。
    /// - Parameters:
    ///   - streamId: 対象ストリームID。
    ///   - fromVersion: 読み込み開始バージョン（inclusive）。
    /// - Returns: 指定バージョン以降のイベント一覧。
    func readStream(_ streamId: StreamId, fromVersion: Int) async throws -> [EventEnvelope]
}

/// インメモリイベントストア。楽観的ロックをサポート。
public actor InMemoryEventStore: EventStore {
    private var streams: [StreamId: [EventEnvelope]] = [:]

    public init() {}

    /// イベントをストリームに追記する。
    public func append(
        events: [EventEnvelope],
        to streamId: StreamId,
        expectedVersion: Int? = nil
    ) async throws {
        let existing = streams[streamId] ?? []
        let currentVersion = existing.last?.version ?? -1

        if let expected = expectedVersion, expected != currentVersion {
            throw EventStoreError.versionConflict(expected: expected, actual: currentVersion)
        }

        var updated = existing
        updated.append(contentsOf: events)
        streams[streamId] = updated
    }

    /// ストリームからイベントを読み込む。
    public func readStream(_ streamId: StreamId, fromVersion: Int = 0) async throws -> [EventEnvelope] {
        guard let events = streams[streamId] else {
            if fromVersion == 0 {
                return []
            }
            throw EventStoreError.streamNotFound(streamId.value)
        }
        return events.filter { $0.version >= fromVersion }
    }
}
