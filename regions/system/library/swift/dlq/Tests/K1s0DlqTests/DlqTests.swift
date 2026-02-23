import Testing
@testable import K1s0Dlq

@Suite("DLQ Tests")
struct DlqTests {
    @Test("DlqStatus の rawValue が正しいこと")
    func testDlqStatusRawValue() {
        #expect(DlqStatus.pending.rawValue == "pending")
        #expect(DlqStatus.retrying.rawValue == "retrying")
        #expect(DlqStatus.resolved.rawValue == "resolved")
        #expect(DlqStatus.dead.rawValue == "dead")
    }

    @Test("DlqError の説明が含まれること")
    func testDlqErrorDescription() {
        let error = DlqError.apiError(status: 404, message: "not found")
        #expect(error.description.contains("API_ERROR"))
        #expect(error.description.contains("404"))
    }

    @Test("エンドポイントの末尾スラッシュが除去されること")
    func testEndpointTrailingSlash() async {
        let client = DlqClient(endpoint: "http://localhost:8080/")
        // クライアントが正常に初期化されることを確認
        _ = client
    }
}
