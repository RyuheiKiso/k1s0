import Foundation

/// 設定ファイル読み込みと検証。
public enum ConfigLoader: Sendable {
    /// JSON設定ファイルを読み込む。
    public static func load(from url: URL) throws -> Config {
        let data: Data
        do {
            data = try Data(contentsOf: url)
        } catch {
            throw ConfigError.readFile(url.path)
        }
        do {
            return try JSONDecoder().decode(Config.self, from: data)
        } catch {
            throw ConfigError.parseJSON(error.localizedDescription)
        }
    }

    /// 設定を検証する。
    public static func validate(_ config: Config) throws {
        guard !config.app.name.isEmpty else {
            throw ConfigError.validation("app.name は必須です")
        }
        guard !config.app.version.isEmpty else {
            throw ConfigError.validation("app.version は必須です")
        }
        let validTiers = ["system", "business", "service"]
        guard validTiers.contains(config.app.tier) else {
            throw ConfigError.validation("app.tier は \(validTiers) のいずれかである必要があります")
        }
        let validEnvs = ["dev", "staging", "prod"]
        guard validEnvs.contains(config.app.environment) else {
            throw ConfigError.validation("app.environment は \(validEnvs) のいずれかである必要があります")
        }
        guard !config.server.host.isEmpty else {
            throw ConfigError.validation("server.host は必須です")
        }
        guard config.server.port > 0 else {
            throw ConfigError.validation("server.port は 0 より大きい必要があります")
        }
        guard !config.auth.jwt.issuer.isEmpty else {
            throw ConfigError.validation("auth.jwt.issuer は必須です")
        }
        guard !config.auth.jwt.audience.isEmpty else {
            throw ConfigError.validation("auth.jwt.audience は必須です")
        }
    }
}
