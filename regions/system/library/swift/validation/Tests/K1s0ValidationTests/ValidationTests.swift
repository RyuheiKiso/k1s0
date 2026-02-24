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

    @Test("有効なページネーションが検証を通過すること")
    func testValidPagination() throws {
        try validatePagination(page: 1, perPage: 10)
        try validatePagination(page: 1, perPage: 1)
        try validatePagination(page: 1, perPage: 100)
        try validatePagination(page: 999, perPage: 50)
    }

    @Test("無効なページ番号がエラーになること")
    func testInvalidPage() {
        #expect(throws: ValidationError.self) {
            try validatePagination(page: 0, perPage: 10)
        }
        #expect(throws: ValidationError.self) {
            try validatePagination(page: -1, perPage: 10)
        }
    }

    @Test("無効なperPageがエラーになること")
    func testInvalidPerPage() {
        #expect(throws: ValidationError.self) {
            try validatePagination(page: 1, perPage: 0)
        }
        #expect(throws: ValidationError.self) {
            try validatePagination(page: 1, perPage: 101)
        }
    }

    @Test("有効な日付範囲が検証を通過すること")
    func testValidDateRange() throws {
        let start = Date(timeIntervalSince1970: 1704067200) // 2024-01-01
        let end = Date(timeIntervalSince1970: 1735689599)   // 2024-12-31
        try validateDateRange(startDate: start, endDate: end)
    }

    @Test("同一日付が検証を通過すること")
    func testEqualDateRange() throws {
        let dt = Date(timeIntervalSince1970: 1718438400) // 2024-06-15
        try validateDateRange(startDate: dt, endDate: dt)
    }

    @Test("開始日が終了日より後の場合にエラーになること")
    func testInvalidDateRange() {
        let start = Date(timeIntervalSince1970: 1735689599) // 2024-12-31
        let end = Date(timeIntervalSince1970: 1704067200)   // 2024-01-01
        #expect(throws: ValidationError.self) {
            try validateDateRange(startDate: start, endDate: end)
        }
    }

    @Test("ValidationErrorにcodeプロパティがあること")
    func testValidationErrorCode() {
        let emailErr = ValidationError.invalidEmail("bad")
        #expect(emailErr.code == "INVALID_EMAIL")

        let pageErr = ValidationError.invalidPage(0)
        #expect(pageErr.code == "INVALID_PAGE")

        let dateErr = ValidationError.invalidDateRange("bad")
        #expect(dateErr.code == "INVALID_DATE_RANGE")
    }

    @Test("ValidationErrorsコレクションが動作すること")
    func testValidationErrors() {
        var errors = ValidationErrors()
        #expect(!errors.hasErrors())
        #expect(errors.getErrors().isEmpty)

        errors.add(.invalidEmail("bad"))
        errors.add(.invalidPage(0))

        #expect(errors.hasErrors())
        #expect(errors.getErrors().count == 2)
        #expect(errors.getErrors()[0].code == "INVALID_EMAIL")
        #expect(errors.getErrors()[1].code == "INVALID_PAGE")
    }
}
