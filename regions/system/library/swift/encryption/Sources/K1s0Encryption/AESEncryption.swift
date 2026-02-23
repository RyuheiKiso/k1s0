import Foundation
import Crypto

public func generateKey() -> SymmetricKey {
    SymmetricKey(size: .bits256)
}

public func encrypt(key: SymmetricKey, plaintext: String) throws -> String {
    let data = Data(plaintext.utf8)
    let sealed = try AES.GCM.seal(data, using: key)
    return sealed.combined!.base64EncodedString()
}

public func decrypt(key: SymmetricKey, ciphertext: String) throws -> String {
    guard let data = Data(base64Encoded: ciphertext) else {
        throw EncryptionError.invalidInput
    }
    let box = try AES.GCM.SealedBox(combined: data)
    let decrypted = try AES.GCM.open(box, using: key)
    guard let result = String(data: decrypted, encoding: .utf8) else {
        throw EncryptionError.decryptionFailed
    }
    return result
}
