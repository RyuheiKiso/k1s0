import Testing
import Crypto
@testable import K1s0Encryption

@Suite("Encryption Tests")
struct EncryptionTests {
    @Test("暗号化と復号化が正しく動作すること")
    func testEncryptDecrypt() throws {
        let key = generateKey()
        let plaintext = "Hello, World!"
        let ciphertext = try encrypt(key: key, plaintext: plaintext)
        let decrypted = try decrypt(key: key, ciphertext: ciphertext)
        #expect(decrypted == plaintext)
    }

    @Test("異なるキーで復号化が失敗すること")
    func testDecryptWithWrongKey() throws {
        let key1 = generateKey()
        let key2 = generateKey()
        let ciphertext = try encrypt(key: key1, plaintext: "secret")
        #expect(throws: (any Error).self) {
            try decrypt(key: key2, ciphertext: ciphertext)
        }
    }

    @Test("無効なBase64入力がエラーになること")
    func testInvalidInput() {
        let key = generateKey()
        #expect(throws: EncryptionError.self) {
            try decrypt(key: key, ciphertext: "not-valid-base64!!!")
        }
    }

    @Test("パスワードハッシュが一致すること")
    func testPasswordHash() {
        let hash = hashPassword("myPassword123")
        #expect(verifyPassword("myPassword123", hash: hash))
    }

    @Test("異なるパスワードのハッシュが一致しないこと")
    func testPasswordHashMismatch() {
        let hash = hashPassword("password1")
        #expect(!verifyPassword("password2", hash: hash))
    }

    @Test("同じパスワードは常に同じハッシュになること")
    func testPasswordHashDeterministic() {
        let hash1 = hashPassword("test")
        let hash2 = hashPassword("test")
        #expect(hash1 == hash2)
    }
}
