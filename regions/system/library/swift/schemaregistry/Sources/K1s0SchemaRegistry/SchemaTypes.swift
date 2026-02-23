/// スキーマフォーマット種別。
public enum SchemaType: String, Codable, Sendable {
    case avro = "AVRO"
    case json = "JSON"
    case protobuf = "PROTOBUF"
}

/// スキーマ互換性モード。
public enum CompatibilityMode: String, Codable, Sendable {
    case backward = "BACKWARD"
    case backwardTransitive = "BACKWARD_TRANSITIVE"
    case forward = "FORWARD"
    case forwardTransitive = "FORWARD_TRANSITIVE"
    case full = "FULL"
    case fullTransitive = "FULL_TRANSITIVE"
    case none = "NONE"
}

/// 登録済みスキーマ。
public struct RegisteredSchema: Codable, Sendable {
    public let id: Int
    public let subject: String
    public let version: Int
    public let schema: String
    public let schemaType: SchemaType

    enum CodingKeys: String, CodingKey {
        case id, subject, version, schema
        case schemaType = "schemaType"
    }
}
