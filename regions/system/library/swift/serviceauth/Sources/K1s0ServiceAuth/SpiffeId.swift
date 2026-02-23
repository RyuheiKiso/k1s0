/// SPIFFE ワークロードアイデンティティ。
///
/// フォーマット: `spiffe://{trust_domain}/ns/{namespace}/sa/{service_account}`
public struct SpiffeId: Sendable {
    public let trustDomain: String
    public let namespace: String
    public let serviceAccount: String

    /// SPIFFE URI を解析する。
    public static func parse(_ uri: String) throws -> SpiffeId {
        guard uri.hasPrefix("spiffe://") else {
            throw ServiceAuthError.spiffeValidationFailed("spiffe:// スキームが必要です")
        }
        let path = String(uri.dropFirst("spiffe://".count))
        let components = path.split(separator: "/", maxSplits: 4).map(String.init)
        guard components.count == 5,
              components[1] == "ns",
              components[3] == "sa" else {
            throw ServiceAuthError.spiffeValidationFailed("不正なSPIFFE URI形式: \(uri)")
        }
        return SpiffeId(
            trustDomain: components[0],
            namespace: components[2],
            serviceAccount: components[4]
        )
    }

    /// SPIFFE URI を返す。
    public var toURI: String {
        "spiffe://\(trustDomain)/ns/\(namespace)/sa/\(serviceAccount)"
    }

    /// ターゲットTierへのアクセスを許可するか判定する。
    ///
    /// - system → system/business/service 全てアクセス可
    /// - business → business/service アクセス可
    /// - service → service のみ
    public func allowsTierAccess(to targetTier: String) -> Bool {
        switch namespace {
        case "system":
            return true
        case "business":
            return targetTier == "business" || targetTier == "service"
        case "service":
            return targetTier == "service"
        default:
            return false
        }
    }
}
