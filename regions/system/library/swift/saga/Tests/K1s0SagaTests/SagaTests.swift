import Foundation
import Testing
@testable import K1s0Saga

@Suite("Saga Tests")
struct SagaTests {
    @Test("SagaStatus の rawValue が正しいこと")
    func testSagaStatusRawValue() {
        #expect(SagaStatus.started.rawValue == "started")
        #expect(SagaStatus.completed.rawValue == "completed")
        #expect(SagaStatus.failed.rawValue == "failed")
        #expect(SagaStatus.cancelled.rawValue == "cancelled")
    }

    @Test("StartSagaRequest が正しくエンコードされること")
    func testStartSagaRequestEncoding() throws {
        let request = StartSagaRequest(
            workflowName: "order-saga",
            payload: ["orderId": "order-1"],
            correlationId: "corr-1",
            initiatedBy: "order-service"
        )
        let data = try JSONEncoder().encode(request)
        let json = try JSONSerialization.jsonObject(with: data) as! [String: Any]
        #expect(json["workflow_name"] as? String == "order-saga")
        #expect(json["initiated_by"] as? String == "order-service")
    }

    @Test("SagaError の説明が含まれること")
    func testSagaErrorDescription() {
        let error = SagaError.apiError(statusCode: 404, message: "not found")
        #expect(error.description.contains("API_ERROR"))
        #expect(error.description.contains("404"))
    }
}
