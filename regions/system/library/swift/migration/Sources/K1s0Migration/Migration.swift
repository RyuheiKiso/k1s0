import Foundation

// MARK: - Protocol

public protocol MigrationRunner: Sendable {
    func runUp() async throws -> MigrationReport
    func runDown(steps: Int) async throws -> MigrationReport
    func status() async throws -> [MigrationStatus]
    func pending() async throws -> [PendingMigration]
}

// MARK: - Config

public struct MigrationConfig: Sendable {
    public let migrationsDir: URL
    public let databaseUrl: String
    public let tableName: String

    public init(
        migrationsDir: URL,
        databaseUrl: String,
        tableName: String = "_migrations"
    ) {
        self.migrationsDir = migrationsDir
        self.databaseUrl = databaseUrl
        self.tableName = tableName
    }
}

// MARK: - Report

public struct MigrationReport: Sendable {
    public let appliedCount: Int
    public let elapsed: Duration
    public let errors: [any Error & Sendable]

    public init(appliedCount: Int, elapsed: Duration, errors: [any Error & Sendable] = []) {
        self.appliedCount = appliedCount
        self.elapsed = elapsed
        self.errors = errors
    }
}

// MARK: - Status

public struct MigrationStatus: Sendable {
    public let version: String
    public let name: String
    public let appliedAt: Date?
    public let checksum: String

    public init(version: String, name: String, appliedAt: Date?, checksum: String) {
        self.version = version
        self.name = name
        self.appliedAt = appliedAt
        self.checksum = checksum
    }
}

// MARK: - PendingMigration

public struct PendingMigration: Sendable {
    public let version: String
    public let name: String

    public init(version: String, name: String) {
        self.version = version
        self.name = name
    }
}

// MARK: - Error

public enum MigrationError: Error, Sendable {
    case connectionFailed(underlying: any Error & Sendable)
    case migrationFailed(version: String, underlying: any Error & Sendable)
    case checksumMismatch(version: String, expected: String, actual: String)
    case directoryNotFound(path: String)
    case rollbackNotSupported(version: String)
}

// MARK: - Direction

public enum MigrationDirection: Sendable {
    case up
    case down
}

// MARK: - File Parser

public struct MigrationFileParser: Sendable {
    public struct ParsedFile: Sendable {
        public let version: String
        public let name: String
        public let direction: MigrationDirection
    }

    public static func parseFilename(_ filename: String) -> ParsedFile? {
        guard filename.hasSuffix(".sql") else { return nil }
        let stem = String(filename.dropLast(4))

        let direction: MigrationDirection
        let rest: String

        if stem.hasSuffix(".up") {
            direction = .up
            rest = String(stem.dropLast(3))
        } else if stem.hasSuffix(".down") {
            direction = .down
            rest = String(stem.dropLast(5))
        } else {
            return nil
        }

        guard let idx = rest.firstIndex(of: "_"),
              idx > rest.startIndex,
              rest.index(after: idx) < rest.endIndex else {
            return nil
        }

        let version = String(rest[rest.startIndex..<idx])
        let name = String(rest[rest.index(after: idx)...])

        return ParsedFile(version: version, name: name, direction: direction)
    }

    public static func computeChecksum(_ content: String) -> String {
        let data = Array(content.utf8)
        var hash = [UInt8](repeating: 0, count: 32)
        // Simple SHA-256 alternative using built-in approach
        // Use a basic hash for portability without CryptoKit
        var h0: UInt64 = 0x6a09e667bb67ae85
        var h1: UInt64 = 0x3c6ef372a54ff53a
        for byte in data {
            h0 = h0 &* 31 &+ UInt64(byte)
            h1 = h1 &* 37 &+ UInt64(byte)
        }
        return String(format: "%016x%016x", h0, h1)
    }
}

// MARK: - InMemory Runner

public actor InMemoryMigrationRunner: MigrationRunner {
    private struct Entry: Sendable {
        let version: String
        let name: String
        let content: String
    }

    private let config: MigrationConfig
    private let upMigrations: [Entry]
    private let downMigrations: [String: Entry]
    private var applied: [MigrationStatus] = []

    public init(
        config: MigrationConfig,
        ups: [(version: String, name: String, content: String)],
        downs: [(version: String, name: String, content: String)]
    ) {
        self.config = config
        self.upMigrations = ups
            .map { Entry(version: $0.version, name: $0.name, content: $0.content) }
            .sorted { $0.version < $1.version }
        self.downMigrations = Dictionary(
            uniqueKeysWithValues: downs.map {
                ($0.version, Entry(version: $0.version, name: $0.name, content: $0.content))
            }
        )
    }

    public func runUp() async throws -> MigrationReport {
        let start = ContinuousClock().now
        let appliedVersions = Set(applied.map(\.version))
        var count = 0

        for mf in upMigrations {
            if appliedVersions.contains(mf.version) { continue }
            let cs = MigrationFileParser.computeChecksum(mf.content)
            applied.append(MigrationStatus(
                version: mf.version,
                name: mf.name,
                appliedAt: Date(),
                checksum: cs
            ))
            count += 1
        }

        let elapsed = ContinuousClock().now - start
        return MigrationReport(appliedCount: count, elapsed: elapsed)
    }

    public func runDown(steps: Int) async throws -> MigrationReport {
        let start = ContinuousClock().now
        var count = 0

        for _ in 0..<steps {
            if applied.isEmpty { break }
            applied.removeLast()
            count += 1
        }

        let elapsed = ContinuousClock().now - start
        return MigrationReport(appliedCount: count, elapsed: elapsed)
    }

    public func status() async throws -> [MigrationStatus] {
        let appliedMap = Dictionary(uniqueKeysWithValues: applied.map { ($0.version, $0) })
        return upMigrations.map { mf in
            let cs = MigrationFileParser.computeChecksum(mf.content)
            let appliedStatus = appliedMap[mf.version]
            return MigrationStatus(
                version: mf.version,
                name: mf.name,
                appliedAt: appliedStatus?.appliedAt,
                checksum: cs
            )
        }
    }

    public func pending() async throws -> [PendingMigration] {
        let appliedVersions = Set(applied.map(\.version))
        return upMigrations
            .filter { !appliedVersions.contains($0.version) }
            .map { PendingMigration(version: $0.version, name: $0.name) }
    }
}
