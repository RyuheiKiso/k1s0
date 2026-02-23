/// フィーチャーフラグクライアントプロトコル。
public protocol FeatureFlagClient: Sendable {
    /// フラグを評価する。
    func evaluate(_ flagKey: String, context: EvaluationContext) async throws -> EvaluationResult
    /// フラグが有効かどうかを確認する。
    func isEnabled(_ flagKey: String, context: EvaluationContext) async throws -> Bool
    /// フラグの定義を取得する。
    func getFlag(_ flagKey: String) async throws -> FeatureFlag
}

/// インメモリフィーチャーフラグクライアント。テストやローカル開発向け。
public actor InMemoryFeatureFlagClient: FeatureFlagClient {
    private var flags: [String: FeatureFlag] = [:]

    public init() {}

    /// フラグを設定する。
    public func setFlag(_ flag: FeatureFlag) {
        flags[flag.flagKey] = flag
    }

    /// フラグを評価する。
    public func evaluate(_ flagKey: String, context: EvaluationContext) async throws -> EvaluationResult {
        guard let flag = flags[flagKey] else {
            throw FeatureFlagError.flagNotFound(flagKey)
        }

        guard flag.enabled else {
            return EvaluationResult(enabled: false, variant: nil, reason: "FLAG_DISABLED")
        }

        if flag.variants.isEmpty {
            return EvaluationResult(enabled: true, variant: nil, reason: "FLAG_ENABLED")
        }

        let totalWeight = flag.variants.reduce(0) { $0 + $1.weight }
        let seed = context.userId ?? context.tenantId ?? ""
        let hash = abs(seed.hashValue) % max(totalWeight, 1)

        var cumulative = 0
        for variant in flag.variants {
            cumulative += variant.weight
            if hash < cumulative {
                return EvaluationResult(enabled: true, variant: variant, reason: "VARIANT_ASSIGNED")
            }
        }

        let defaultVariant = flag.variants.first
        return EvaluationResult(enabled: true, variant: defaultVariant, reason: "DEFAULT_VARIANT")
    }

    /// フラグが有効かどうかを確認する。
    public func isEnabled(_ flagKey: String, context: EvaluationContext) async throws -> Bool {
        let result = try await evaluate(flagKey, context: context)
        return result.enabled
    }

    /// フラグの定義を取得する。
    public func getFlag(_ flagKey: String) async throws -> FeatureFlag {
        guard let flag = flags[flagKey] else {
            throw FeatureFlagError.flagNotFound(flagKey)
        }
        return flag
    }
}
