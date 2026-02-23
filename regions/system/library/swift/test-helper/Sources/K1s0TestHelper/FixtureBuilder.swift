import Foundation

/// テスト用フィクスチャビルダー。
public struct FixtureBuilder: Sendable {
    /// ランダム UUID を生成する。
    public static func uuid() -> String {
        UUID().uuidString.lowercased()
    }

    /// ランダムなテスト用メールアドレスを生成する。
    public static func email() -> String {
        "test-\(String(uuid().prefix(8)))@example.com"
    }

    /// ランダムなテスト用ユーザー名を生成する。
    public static func name() -> String {
        "user-\(String(uuid().prefix(8)))"
    }

    /// 指定範囲のランダム整数を生成する。
    public static func int(min: Int = 0, max: Int = 100) -> Int {
        guard min < max else { return min }
        return Int.random(in: min..<max)
    }

    /// テスト用テナント ID を生成する。
    public static func tenantId() -> String {
        "tenant-\(String(uuid().prefix(8)))"
    }
}
