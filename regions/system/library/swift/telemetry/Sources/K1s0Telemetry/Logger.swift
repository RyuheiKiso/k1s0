/// 構造化ログ出力（クロスプラットフォーム対応）。
public struct Logger: Sendable {
    private let subsystem: String
    private let category: String

    public init(subsystem: String, category: String) {
        self.subsystem = subsystem
        self.category = category
    }

    public func debug(_ message: String) {
        log(level: "DEBUG", message: message)
    }

    public func info(_ message: String) {
        log(level: "INFO", message: message)
    }

    public func warning(_ message: String) {
        log(level: "WARN", message: message)
    }

    public func error(_ message: String) {
        log(level: "ERROR", message: message)
    }

    private func log(level: String, message: String) {
        print("[\(level)] [\(subsystem)/\(category)] \(message)")
    }
}
