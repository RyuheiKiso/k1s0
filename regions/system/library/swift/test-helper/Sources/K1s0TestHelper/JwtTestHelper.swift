import Foundation

/// テスト用 JWT トークン生成ヘルパー。
public struct JwtTestHelper: Sendable {
    private let secret: String

    public init(secret: String) {
        self.secret = secret
    }

    /// 管理者トークンを生成する。
    public func createAdminToken() throws -> String {
        return try createToken(claims: TestClaims(sub: "admin", roles: ["admin"]))
    }

    /// ユーザートークンを生成する。
    public func createUserToken(userId: String, roles: [String]) throws -> String {
        return try createToken(claims: TestClaims(sub: userId, roles: roles))
    }

    /// カスタムクレームでトークンを生成する。
    public func createToken(claims: TestClaims) throws -> String {
        let header = base64UrlEncode(Data("{\"alg\":\"HS256\",\"typ\":\"JWT\"}".utf8))
        let encoder = JSONEncoder()
        let payloadData = try encoder.encode(claims)
        let payload = base64UrlEncode(payloadData)
        let signingInput = "\(header).\(payload)"
        let signature = base64UrlEncode(Data("\(signingInput):\(secret)".utf8))
        return "\(signingInput).\(signature)"
    }

    /// トークンのペイロードをデコードしてクレームを返す。
    public func decodeClaims(token: String) -> TestClaims? {
        let parts = token.split(separator: ".")
        guard parts.count == 3 else { return nil }
        guard let payloadData = base64UrlDecode(String(parts[1])) else { return nil }
        let decoder = JSONDecoder()
        return try? decoder.decode(TestClaims.self, from: payloadData)
    }

    private func base64UrlEncode(_ data: Data) -> String {
        return data.base64EncodedString()
            .replacingOccurrences(of: "+", with: "-")
            .replacingOccurrences(of: "/", with: "_")
            .replacingOccurrences(of: "=", with: "")
    }

    private func base64UrlDecode(_ string: String) -> Data? {
        var base64 = string
            .replacingOccurrences(of: "-", with: "+")
            .replacingOccurrences(of: "_", with: "/")
        let mod = base64.count % 4
        if mod != 0 {
            base64 += String(repeating: "=", count: 4 - mod)
        }
        return Data(base64Encoded: base64)
    }
}
