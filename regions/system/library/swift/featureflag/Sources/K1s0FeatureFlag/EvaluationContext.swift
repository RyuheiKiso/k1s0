/// フィーチャーフラグ評価のコンテキスト。
public struct EvaluationContext: Sendable {
    /// ユーザーID（オプション）。
    public let userId: String?
    /// テナントID（オプション）。
    public let tenantId: String?

    public init(userId: String? = nil, tenantId: String? = nil) {
        self.userId = userId
        self.tenantId = tenantId
    }
}

/// フィーチャーフラグ評価の結果。
public struct EvaluationResult: Sendable {
    /// フラグが有効かどうか。
    public let enabled: Bool
    /// 選択されたバリアント（バリアントがある場合）。
    public let variant: FlagVariant?
    /// 評価の理由。
    public let reason: String

    public init(enabled: Bool, variant: FlagVariant? = nil, reason: String) {
        self.enabled = enabled
        self.variant = variant
        self.reason = reason
    }
}
