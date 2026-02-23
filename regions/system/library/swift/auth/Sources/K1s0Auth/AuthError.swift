/// 認証・認可エラー。
public enum AuthError: Error, Sendable {
    case tokenExpired
    case invalidToken(String)
    case jwksFetchFailed(String)
    case missingToken
    case invalidAuthHeader
    case permissionDenied
    case tierAccessDenied
}

extension AuthError: CustomStringConvertible {
    public var description: String {
        switch self {
        case .tokenExpired:
            return "TOKEN_EXPIRED: トークンの有効期限が切れています"
        case .invalidToken(let reason):
            return "INVALID_TOKEN: \(reason)"
        case .jwksFetchFailed(let reason):
            return "JWKS_FETCH_FAILED: \(reason)"
        case .missingToken:
            return "MISSING_TOKEN: トークンが見つかりません"
        case .invalidAuthHeader:
            return "INVALID_AUTH_HEADER: Authorizationヘッダーが不正です"
        case .permissionDenied:
            return "PERMISSION_DENIED: 権限がありません"
        case .tierAccessDenied:
            return "TIER_ACCESS_DENIED: Tierへのアクセスが拒否されました"
        }
    }
}
