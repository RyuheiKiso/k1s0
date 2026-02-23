import Foundation
import Crypto

public func hashPassword(_ password: String) -> String {
    let digest = SHA256.hash(data: Data(password.utf8))
    return Data(digest).base64EncodedString()
}

public func verifyPassword(_ password: String, hash: String) -> Bool {
    hashPassword(password) == hash
}
