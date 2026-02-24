import Foundation
import Argon2Swift

/// Argon2id parameters matching other k1s0 libraries.
private let argon2Memory = 19456
private let argon2Iterations = 2
private let argon2Parallelism = 1
private let argon2HashLength = 32

/// Hash a password with Argon2id.
///
/// Returns a PHC-format string:
/// `$argon2id$v=19$m=19456,t=2,p=1$<salt_base64>$<hash_base64>`
public func hashPassword(_ password: String) -> String {
    let salt = Salt.newSalt()
    let result = try! Argon2Swift.hashPasswordString(
        password: password,
        salt: salt,
        iterations: argon2Iterations,
        memory: argon2Memory,
        parallelism: argon2Parallelism,
        length: argon2HashLength,
        type: .id,
        version: .V13
    )
    return result.encodedString()
}

/// Verify a password against an Argon2id hash string.
public func verifyPassword(_ password: String, hash: String) -> Bool {
    do {
        return try Argon2Swift.verifyHashString(
            password: password,
            hash: hash,
            type: .id
        )
    } catch {
        return false
    }
}
