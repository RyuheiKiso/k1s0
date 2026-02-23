/// テレメトリの初期化（アクターベースでスレッドセーフ）。
public actor TelemetrySetup: Sendable {
    public static let shared = TelemetrySetup()

    private var _config: TelemetryConfig?

    private init() {}

    /// テレメトリを初期化する。
    public func initialize(config: TelemetryConfig) {
        _config = config
    }

    /// 設定された設定を返す。
    public var config: TelemetryConfig? {
        _config
    }

    /// グローバルメトリクスインスタンス。
    public static let metrics = Metrics()
}
