/// フィーチャーフラグ操作のエラー。
public enum FeatureFlagError: Error, Sendable {
    /// 指定されたフラグが見つからなかった。
    case flagNotFound(String)
}

extension FeatureFlagError: CustomStringConvertible {
    public var description: String {
        switch self {
        case .flagNotFound(let key):
            return "FEATURE_FLAG_NOT_FOUND: フラグ '\(key)' が見つかりません"
        }
    }
}
