/// フラグのバリアント（変種）。A/Bテストや段階的ロールアウトに使用。
public struct FlagVariant: Sendable, Equatable {
    /// バリアント名。
    public let name: String
    /// バリアントの値。
    public let value: String
    /// バリアントの重み（割合）。
    public let weight: Int

    public init(name: String, value: String, weight: Int) {
        self.name = name
        self.value = value
        self.weight = weight
    }
}

/// フィーチャーフラグの定義。
public struct FeatureFlag: Sendable {
    /// フラグの一意識別子。
    public let id: String
    /// フラグのキー（参照用）。
    public let flagKey: String
    /// フラグの説明。
    public let description: String
    /// フラグが有効かどうか。
    public let enabled: Bool
    /// フラグのバリアント一覧。
    public let variants: [FlagVariant]

    public init(
        id: String,
        flagKey: String,
        description: String,
        enabled: Bool,
        variants: [FlagVariant] = []
    ) {
        self.id = id
        self.flagKey = flagKey
        self.description = description
        self.enabled = enabled
        self.variants = variants
    }
}
