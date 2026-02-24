import Foundation
import Crypto
import _CryptoExtras

#if canImport(FoundationNetworking)
import FoundationNetworking
#endif

// MARK: - JWKS レスポンスモデル

/// JWKS エンドポイントのレスポンス。
struct JWKSResponse: Codable, Sendable {
    let keys: [JWK]
}

/// JSON Web Key (RSA 公開鍵)。
struct JWK: Codable, Sendable {
    let kty: String
    let kid: String?
    let use: String?
    let alg: String?
    let n: String
    let e: String
}

/// JWT ヘッダー。
struct JWTHeader: Codable, Sendable {
    let alg: String
    let kid: String?
    let typ: String?
}

// MARK: - JWKS データ取得プロトコル (テスト可能性のため)

/// JWKS データ取得の抽象化。
public protocol JWKSDataFetcher: Sendable {
    func fetchJWKSData(from url: URL) async throws -> Data
}

/// URLSession を使用したデフォルトの JWKS フェッチャー。
public struct DefaultJWKSFetcher: JWKSDataFetcher, Sendable {
    public init() {}

    public func fetchJWKSData(from url: URL) async throws -> Data {
        let (data, response) = try await URLSession.shared.data(from: url)
        if let httpResponse = response as? HTTPURLResponse,
           !(200...299).contains(httpResponse.statusCode) {
            throw AuthError.jwksFetchFailed("JWKS エンドポイントが HTTP \(httpResponse.statusCode) を返しました")
        }
        return data
    }
}

// MARK: - JwksVerifier

/// JWKS エンドポイントを使用した JWT 検証。
public actor JwksVerifier: Sendable {
    private let jwksURL: URL
    private let issuer: String
    private let audience: String
    private let cacheTTL: TimeInterval
    private var cachedKeys: [String: _RSA.Signing.PublicKey]?
    private var cacheTimestamp: Date?
    private let fetcher: JWKSDataFetcher

    public init(
        jwksURL: URL,
        issuer: String,
        audience: String,
        cacheTTL: TimeInterval = 3600,
        fetcher: JWKSDataFetcher = DefaultJWKSFetcher()
    ) {
        self.jwksURL = jwksURL
        self.issuer = issuer
        self.audience = audience
        self.cacheTTL = cacheTTL
        self.fetcher = fetcher
    }

    /// トークンを検証し Claims を返す。
    public func verify(token: String) async throws -> Claims {
        let parts = token.split(separator: ".").map(String.init)
        guard parts.count == 3 else {
            throw AuthError.invalidToken("JWTの形式が不正です")
        }

        // ヘッダーのデコード
        guard let headerData = Data(base64URLEncoded: parts[0]) else {
            throw AuthError.invalidToken("ヘッダーのBase64デコードに失敗しました")
        }
        let header = try JSONDecoder().decode(JWTHeader.self, from: headerData)

        // RS256 のみサポート
        guard header.alg == "RS256" else {
            throw AuthError.invalidToken("サポートされていないアルゴリズム: \(header.alg)")
        }

        // ペイロードのデコード
        guard let payloadData = Data(base64URLEncoded: parts[1]) else {
            throw AuthError.invalidToken("ペイロードのBase64デコードに失敗しました")
        }
        let claims = try JSONDecoder().decode(Claims.self, from: payloadData)

        // 有効期限チェック
        let now = Date().timeIntervalSince1970
        guard claims.exp > now else {
            throw AuthError.tokenExpired
        }

        // issuer チェック
        guard claims.iss == issuer else {
            throw AuthError.invalidToken("issuer が一致しません: expected=\(issuer), actual=\(claims.iss)")
        }

        // JWKS キーの取得
        let keys = try await fetchKeys()

        // kid に対応するキーを取得
        let publicKey: _RSA.Signing.PublicKey
        if let kid = header.kid {
            guard let key = keys[kid] else {
                throw AuthError.invalidToken("kid '\(kid)' に対応するキーが見つかりません")
            }
            publicKey = key
        } else {
            // kid が無い場合、最初のキーを使用
            guard let firstKey = keys.values.first else {
                throw AuthError.jwksFetchFailed("JWKS にキーが含まれていません")
            }
            publicKey = firstKey
        }

        // 署名の検証
        guard let signatureData = Data(base64URLEncoded: parts[2]) else {
            throw AuthError.invalidToken("署名のBase64デコードに失敗しました")
        }

        let signingInput = Data("\(parts[0]).\(parts[1])".utf8)
        let digest = SHA256.hash(data: signingInput)
        let signature = _RSA.Signing.RSASignature(rawRepresentation: signatureData)

        guard publicKey.isValidSignature(signature, for: digest, padding: .insecurePKCS1v1_5) else {
            throw AuthError.invalidToken("RSA署名の検証に失敗しました")
        }

        return claims
    }

    /// キャッシュを無効化する。
    public func invalidateCache() {
        cachedKeys = nil
        cacheTimestamp = nil
    }

    // MARK: - Private

    /// JWKS キーを取得する (キャッシュ付き)。
    private func fetchKeys() async throws -> [String: _RSA.Signing.PublicKey] {
        // キャッシュが有効な場合はそれを返す
        if let cached = cachedKeys,
           let timestamp = cacheTimestamp,
           Date().timeIntervalSince(timestamp) < cacheTTL {
            return cached
        }

        let data: Data
        do {
            data = try await fetcher.fetchJWKSData(from: jwksURL)
        } catch let error as AuthError {
            throw error
        } catch {
            throw AuthError.jwksFetchFailed("JWKS エンドポイントへの接続に失敗しました: \(error.localizedDescription)")
        }

        let jwks: JWKSResponse
        do {
            jwks = try JSONDecoder().decode(JWKSResponse.self, from: data)
        } catch {
            throw AuthError.jwksFetchFailed("JWKS レスポンスのパースに失敗しました: \(error.localizedDescription)")
        }

        var keys: [String: _RSA.Signing.PublicKey] = [:]
        for jwk in jwks.keys {
            // RSA キーのみ処理
            guard jwk.kty == "RSA" else { continue }

            // 署名検証用のキーのみ (use == "sig" or use == nil)
            if let use = jwk.use, use != "sig" { continue }

            guard let modulusData = Data(base64URLEncoded: jwk.n),
                  let exponentData = Data(base64URLEncoded: jwk.e) else {
                continue
            }

            do {
                let rsaKey = try _RSA.Signing.PublicKey(n: modulusData, e: exponentData)
                let keyId = jwk.kid ?? UUID().uuidString
                keys[keyId] = rsaKey
            } catch {
                // 無効なキーはスキップ
                continue
            }
        }

        // キャッシュに保存
        cachedKeys = keys
        cacheTimestamp = Date()

        return keys
    }
}

// MARK: - Data Base64URL Extension

extension Data {
    init?(base64URLEncoded string: String) {
        var base64 = string
            .replacingOccurrences(of: "-", with: "+")
            .replacingOccurrences(of: "_", with: "/")
        let remainder = base64.count % 4
        if remainder != 0 {
            base64 += String(repeating: "=", count: 4 - remainder)
        }
        self.init(base64Encoded: base64)
    }
}
