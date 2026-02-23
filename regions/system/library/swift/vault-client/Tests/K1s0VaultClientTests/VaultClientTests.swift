import Testing
@testable import K1s0VaultClient

@Suite("VaultClient Tests")
struct VaultClientTests {
    func makeConfig() -> VaultClientConfig {
        VaultClientConfig(serverUrl: "http://localhost:8080")
    }

    func makeSecret(_ path: String) -> Secret {
        Secret(
            path: path,
            data: ["password": "s3cr3t", "username": "admin"],
            version: 1,
            createdAt: Date()
        )
    }

    @Test("シークレットを取得できること")
    func testGetSecret() async throws {
        let client = InMemoryVaultClient(config: makeConfig())
        await client.putSecret(makeSecret("system/db/primary"))
        let secret = try await client.getSecret(path: "system/db/primary")
        #expect(secret.path == "system/db/primary")
        #expect(secret.data["password"] == "s3cr3t")
    }

    @Test("存在しないシークレットでエラーが返ること")
    func testGetSecretNotFound() async throws {
        let client = InMemoryVaultClient(config: makeConfig())
        do {
            _ = try await client.getSecret(path: "missing/path")
            #expect(Bool(false), "Expected VaultError.notFound")
        } catch is VaultError {
            // expected
        }
    }

    @Test("シークレットの値を取得できること")
    func testGetSecretValue() async throws {
        let client = InMemoryVaultClient(config: makeConfig())
        await client.putSecret(makeSecret("system/db"))
        let value = try await client.getSecretValue(path: "system/db", key: "password")
        #expect(value == "s3cr3t")
    }

    @Test("存在しないキーでエラーが返ること")
    func testGetSecretValueKeyNotFound() async throws {
        let client = InMemoryVaultClient(config: makeConfig())
        await client.putSecret(makeSecret("system/db"))
        do {
            _ = try await client.getSecretValue(path: "system/db", key: "missing")
            #expect(Bool(false), "Expected VaultError.notFound")
        } catch is VaultError {
            // expected
        }
    }

    @Test("プレフィックスでシークレット一覧を取得できること")
    func testListSecrets() async throws {
        let client = InMemoryVaultClient(config: makeConfig())
        await client.putSecret(makeSecret("system/db/primary"))
        await client.putSecret(makeSecret("system/db/replica"))
        await client.putSecret(makeSecret("business/api/key"))
        let paths = try await client.listSecrets(pathPrefix: "system/")
        #expect(paths.count == 2)
        #expect(paths.allSatisfy { $0.hasPrefix("system/") })
    }

    @Test("一致しないプレフィックスで空配列を返すこと")
    func testListSecretsEmpty() async throws {
        let client = InMemoryVaultClient(config: makeConfig())
        let paths = try await client.listSecrets(pathPrefix: "nothing/")
        #expect(paths.isEmpty)
    }

    @Test("watchSecretがストリームを返すこと")
    func testWatchSecret() async throws {
        let client = InMemoryVaultClient(config: makeConfig())
        let stream = await client.watchSecret(path: "system/db")
        var events: [SecretRotatedEvent] = []
        for try await event in stream {
            events.append(event)
        }
        #expect(events.isEmpty)
    }

    @Test("VaultErrorの各バリアントが存在すること")
    func testVaultErrorVariants() {
        let notFound = VaultError.notFound(path: "test")
        let permDenied = VaultError.permissionDenied(path: "test")
        let serverErr = VaultError.serverError(message: "error")
        let timeout = VaultError.timeout
        let leaseExpired = VaultError.leaseExpired(path: "test")

        #expect(notFound is VaultError)
        #expect(permDenied is VaultError)
        #expect(serverErr is VaultError)
        #expect(timeout is VaultError)
        #expect(leaseExpired is VaultError)
    }

    @Test("VaultClientConfigのデフォルト値が正しいこと")
    func testConfigDefaults() {
        let config = VaultClientConfig(serverUrl: "http://vault:8080")
        #expect(config.cacheTtl == .seconds(600))
        #expect(config.cacheMaxCapacity == 500)
    }

    @Test("SecretRotatedEventのフィールドが正しいこと")
    func testSecretRotatedEvent() {
        let event = SecretRotatedEvent(path: "system/db", version: 2)
        #expect(event.path == "system/db")
        #expect(event.version == 2)
    }
}
