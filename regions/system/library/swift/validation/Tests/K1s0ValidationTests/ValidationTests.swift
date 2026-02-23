import Testing
@testable import K1s0Validation

@Suite("Validation Tests")
struct ValidationTests {
    @Test("有効なメールアドレスが検証を通過すること")
    func testValidEmail() throws {
        try validateEmail("user@example.com")
        try validateEmail("test.name+tag@domain.co.jp")
    }

    @Test("無効なメールアドレスがエラーになること")
    func testInvalidEmail() {
        #expect(throws: ValidationError.self) {
            try validateEmail("invalid")
        }
        #expect(throws: ValidationError.self) {
            try validateEmail("@domain.com")
        }
        #expect(throws: ValidationError.self) {
            try validateEmail("user@")
        }
    }

    @Test("有効なUUIDが検証を通過すること")
    func testValidUUID() throws {
        try validateUUID("550e8400-e29b-41d4-a716-446655440000")
    }

    @Test("無効なUUIDがエラーになること")
    func testInvalidUUID() {
        #expect(throws: ValidationError.self) {
            try validateUUID("not-a-uuid")
        }
        #expect(throws: ValidationError.self) {
            try validateUUID("")
        }
    }

    @Test("有効なURLが検証を通過すること")
    func testValidURL() throws {
        try validateURL("https://example.com")
        try validateURL("http://localhost:8080/path")
    }

    @Test("無効なURLがエラーになること")
    func testInvalidURL() {
        #expect(throws: ValidationError.self) {
            try validateURL("ftp://example.com")
        }
        #expect(throws: ValidationError.self) {
            try validateURL("")
        }
    }

    @Test("有効なテナントIDが検証を通過すること")
    func testValidTenantID() throws {
        try validateTenantID("tenant-01")
        try validateTenantID("abc")
    }

    @Test("無効なテナントIDがエラーになること")
    func testInvalidTenantID() {
        #expect(throws: ValidationError.self) {
            try validateTenantID("-invalid")
        }
        #expect(throws: ValidationError.self) {
            try validateTenantID("AB")
        }
        #expect(throws: ValidationError.self) {
            try validateTenantID("a")
        }
    }
}
