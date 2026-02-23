import Foundation

/// Schema Registry 接続設定。
public struct SchemaRegistryConfig: Sendable {
    public let url: String
    public let compatibility: CompatibilityMode
    public let timeoutSeconds: TimeInterval

    public init(
        url: String,
        compatibility: CompatibilityMode = .backward,
        timeoutSeconds: TimeInterval = 30
    ) {
        self.url = url
        self.compatibility = compatibility
        self.timeoutSeconds = timeoutSeconds
    }

    /// Confluent 規則に従ったサブジェクト名を返す。
    ///
    /// 規則: `{topic}-value`
    public static func subjectName(for topic: String) -> String {
        "\(topic)-value"
    }
}
