/// 設定エラー。
public enum ConfigError: Error, Sendable {
    case readFile(String)
    case parseJSON(String)
    case validation(String)
}

extension ConfigError: CustomStringConvertible {
    public var description: String {
        switch self {
        case .readFile(let path):
            return "READ_FILE_ERROR: \(path)"
        case .parseJSON(let reason):
            return "PARSE_JSON_ERROR: \(reason)"
        case .validation(let reason):
            return "VALIDATION_ERROR: \(reason)"
        }
    }
}
