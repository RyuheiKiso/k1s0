import Testing
import Foundation
@testable import K1s0Migration

func createRunner() -> InMemoryMigrationRunner {
    let config = MigrationConfig(
        migrationsDir: URL(fileURLWithPath: "."),
        databaseUrl: "memory://"
    )
    let ups: [(version: String, name: String, content: String)] = [
        ("20240101000001", "create_users", "CREATE TABLE users (id INT);"),
        ("20240101000002", "add_email", "ALTER TABLE users ADD COLUMN email TEXT;"),
        ("20240201000001", "create_orders", "CREATE TABLE orders (id INT);"),
    ]
    let downs: [(version: String, name: String, content: String)] = [
        ("20240101000001", "create_users", "DROP TABLE users;"),
        ("20240101000002", "add_email", "ALTER TABLE users DROP COLUMN email;"),
        ("20240201000001", "create_orders", "DROP TABLE orders;"),
    ]
    return InMemoryMigrationRunner(config: config, ups: ups, downs: downs)
}

@Suite("MigrationFileParser")
struct MigrationFileParserTests {
    @Test("Parse up migration filename")
    func parseUpMigration() {
        let result = MigrationFileParser.parseFilename("20240101000001_create_users.up.sql")
        #expect(result != nil)
        #expect(result?.version == "20240101000001")
        #expect(result?.name == "create_users")
        #expect(result?.direction == .up)
    }

    @Test("Parse down migration filename")
    func parseDownMigration() {
        let result = MigrationFileParser.parseFilename("20240101000001_create_users.down.sql")
        #expect(result != nil)
        #expect(result?.version == "20240101000001")
        #expect(result?.name == "create_users")
        #expect(result?.direction == .down)
    }

    @Test("Invalid filenames return nil")
    func parseInvalidFilenames() {
        #expect(MigrationFileParser.parseFilename("invalid.sql") == nil)
        #expect(MigrationFileParser.parseFilename("no_direction.sql") == nil)
        #expect(MigrationFileParser.parseFilename("_.up.sql") == nil)
    }

    @Test("Checksum is deterministic")
    func checksumDeterministic() {
        let content = "CREATE TABLE users (id SERIAL PRIMARY KEY);"
        let c1 = MigrationFileParser.computeChecksum(content)
        let c2 = MigrationFileParser.computeChecksum(content)
        #expect(c1 == c2)
    }

    @Test("Checksum differs for different content")
    func checksumDiffers() {
        let c1 = MigrationFileParser.computeChecksum("CREATE TABLE users;")
        let c2 = MigrationFileParser.computeChecksum("CREATE TABLE orders;")
        #expect(c1 != c2)
    }
}

@Suite("MigrationConfig")
struct MigrationConfigTests {
    @Test("Default table name")
    func defaultTableName() {
        let config = MigrationConfig(
            migrationsDir: URL(fileURLWithPath: "."),
            databaseUrl: "memory://"
        )
        #expect(config.tableName == "_migrations")
    }

    @Test("Custom table name")
    func customTableName() {
        let config = MigrationConfig(
            migrationsDir: URL(fileURLWithPath: "."),
            databaseUrl: "memory://",
            tableName: "custom"
        )
        #expect(config.tableName == "custom")
    }
}

@Suite("InMemoryMigrationRunner")
struct InMemoryMigrationRunnerTests {
    @Test("RunUp applies all migrations")
    func runUpAppliesAll() async throws {
        let runner = createRunner()
        let report = try await runner.runUp()
        #expect(report.appliedCount == 3)
        #expect(report.errors.isEmpty)
    }

    @Test("RunUp is idempotent")
    func runUpIdempotent() async throws {
        let runner = createRunner()
        _ = try await runner.runUp()
        let report = try await runner.runUp()
        #expect(report.appliedCount == 0)
    }

    @Test("RunDown rolls back one step")
    func runDownOneStep() async throws {
        let runner = createRunner()
        _ = try await runner.runUp()
        let report = try await runner.runDown(steps: 1)
        #expect(report.appliedCount == 1)

        let p = try await runner.pending()
        #expect(p.count == 1)
        #expect(p[0].version == "20240201000001")
    }

    @Test("RunDown rolls back multiple steps")
    func runDownMultipleSteps() async throws {
        let runner = createRunner()
        _ = try await runner.runUp()
        let report = try await runner.runDown(steps: 2)
        #expect(report.appliedCount == 2)

        let p = try await runner.pending()
        #expect(p.count == 2)
    }

    @Test("RunDown handles more than applied")
    func runDownMoreThanApplied() async throws {
        let runner = createRunner()
        _ = try await runner.runUp()
        let report = try await runner.runDown(steps: 10)
        #expect(report.appliedCount == 3)
    }

    @Test("Status shows all pending initially")
    func statusAllPending() async throws {
        let runner = createRunner()
        let statuses = try await runner.status()
        #expect(statuses.count == 3)
        for s in statuses {
            #expect(s.appliedAt == nil)
        }
    }

    @Test("Status shows all applied after runUp")
    func statusAfterRunUp() async throws {
        let runner = createRunner()
        _ = try await runner.runUp()
        let statuses = try await runner.status()
        #expect(statuses.count == 3)
        for s in statuses {
            #expect(s.appliedAt != nil)
        }
    }

    @Test("Pending returns all unapplied")
    func pendingReturnsAll() async throws {
        let runner = createRunner()
        let p = try await runner.pending()
        #expect(p.count == 3)
        #expect(p[0].version == "20240101000001")
        #expect(p[1].version == "20240101000002")
        #expect(p[2].version == "20240201000001")
    }

    @Test("Pending empty after full apply")
    func pendingEmptyAfterApply() async throws {
        let runner = createRunner()
        _ = try await runner.runUp()
        let p = try await runner.pending()
        #expect(p.isEmpty)
    }
}
