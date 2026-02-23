import Testing
@testable import K1s0Config

@Suite("Config Tests")
struct ConfigTests {
    @Test("JSON設定をデコードできること")
    func testDecodeConfig() throws {
        let json = """
        {
            "app": {"name": "test-service", "version": "1.0.0", "tier": "system", "environment": "dev"},
            "server": {"host": "0.0.0.0", "port": 8080},
            "auth": {"jwt": {"issuer": "https://keycloak.example.com", "audience": "k1s0-api"}}
        }
        """
        let data = json.data(using: .utf8)!
        let config = try JSONDecoder().decode(Config.self, from: data)
        #expect(config.app.name == "test-service")
        #expect(config.server.port == 8080)
    }

    @Test("app.name が空の場合バリデーションエラーになること")
    func testValidationFailsOnEmptyName() throws {
        let config = makeConfig(name: "")
        #expect(throws: ConfigError.self) {
            try ConfigLoader.validate(config)
        }
    }

    @Test("有効な設定はバリデーションを通ること")
    func testValidationPassesWithValidConfig() throws {
        let config = makeConfig()
        try ConfigLoader.validate(config)
    }

    @Test("不正なTierでバリデーションエラーになること")
    func testValidationFailsOnInvalidTier() {
        let config = makeConfig(tier: "invalid")
        #expect(throws: ConfigError.self) {
            try ConfigLoader.validate(config)
        }
    }

    @Test("ConfigErrorの説明が含まれること")
    func testConfigErrorDescription() {
        let error = ConfigError.validation("name is required")
        #expect(error.description.contains("VALIDATION_ERROR"))
    }

    // MARK: - ヘルパー

    private func makeConfig(name: String = "svc", version: String = "1.0.0", tier: String = "system", env: String = "dev") -> Config {
        Config(
            app: AppConfig(name: name, version: version, tier: tier, environment: env),
            server: ServerConfig(host: "localhost", port: 8080, readTimeout: nil, writeTimeout: nil, shutdownTimeout: nil),
            database: nil,
            kafka: nil,
            auth: AuthConfig(jwt: AuthConfig.JwtConfig(issuer: "https://iss.example.com", audience: "k1s0-api", publicKeyPath: nil))
        )
    }
}
