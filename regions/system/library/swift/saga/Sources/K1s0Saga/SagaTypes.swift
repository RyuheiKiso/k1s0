import Foundation

/// Saga のステータス。
public enum SagaStatus: String, Codable, Sendable {
    case started = "started"
    case running = "running"
    case completed = "completed"
    case compensating = "compensating"
    case failed = "failed"
    case cancelled = "cancelled"
}

/// Saga の状態。
public struct SagaState: Codable, Sendable {
    public let sagaId: String
    public let workflowName: String
    public let status: SagaStatus
    public let correlationId: String?
    public let initiatedBy: String
    public let createdAt: String
    public let updatedAt: String

    enum CodingKeys: String, CodingKey {
        case sagaId = "saga_id"
        case workflowName = "workflow_name"
        case status
        case correlationId = "correlation_id"
        case initiatedBy = "initiated_by"
        case createdAt = "created_at"
        case updatedAt = "updated_at"
    }
}

/// Saga 開始リクエスト。
public struct StartSagaRequest: Codable, Sendable {
    public let workflowName: String
    public let payload: [String: String]
    public let correlationId: String?
    public let initiatedBy: String

    enum CodingKeys: String, CodingKey {
        case workflowName = "workflow_name"
        case payload
        case correlationId = "correlation_id"
        case initiatedBy = "initiated_by"
    }

    public init(
        workflowName: String,
        payload: [String: String] = [:],
        correlationId: String? = nil,
        initiatedBy: String
    ) {
        self.workflowName = workflowName
        self.payload = payload
        self.correlationId = correlationId
        self.initiatedBy = initiatedBy
    }
}

/// Saga 開始レスポンス。
public struct StartSagaResponse: Codable, Sendable {
    public let sagaId: String
    public let status: SagaStatus

    enum CodingKeys: String, CodingKey {
        case sagaId = "saga_id"
        case status
    }
}
