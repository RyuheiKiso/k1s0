import Testing
@testable import K1s0FeatureFlag

@Suite("FeatureFlag Tests")
struct FeatureFlagTests {
    @Test("有効なフラグが正しく評価されること")
    func testEvaluateEnabled() async throws {
        let client = InMemoryFeatureFlagClient()
        let flag = FeatureFlag(
            id: "flag-1",
            flagKey: "new-feature",
            description: "新機能フラグ",
            enabled: true
        )
        await client.setFlag(flag)

        let context = EvaluationContext(userId: "user-1")
        let result = try await client.evaluate("new-feature", context: context)
        #expect(result.enabled)
        #expect(result.reason == "FLAG_ENABLED")
    }

    @Test("無効なフラグが正しく評価されること")
    func testEvaluateDisabled() async throws {
        let client = InMemoryFeatureFlagClient()
        let flag = FeatureFlag(
            id: "flag-2",
            flagKey: "disabled-feature",
            description: "無効なフラグ",
            enabled: false
        )
        await client.setFlag(flag)

        let context = EvaluationContext()
        let result = try await client.evaluate("disabled-feature", context: context)
        #expect(!result.enabled)
        #expect(result.reason == "FLAG_DISABLED")
    }

    @Test("isEnabled が有効なフラグで true を返すこと")
    func testIsEnabled() async throws {
        let client = InMemoryFeatureFlagClient()
        let flag = FeatureFlag(
            id: "flag-3",
            flagKey: "active-flag",
            description: "アクティブフラグ",
            enabled: true
        )
        await client.setFlag(flag)

        let context = EvaluationContext(tenantId: "tenant-1")
        let enabled = try await client.isEnabled("active-flag", context: context)
        #expect(enabled)
    }

    @Test("getFlag がフラグ定義を返すこと")
    func testGetFlag() async throws {
        let client = InMemoryFeatureFlagClient()
        let flag = FeatureFlag(
            id: "flag-4",
            flagKey: "my-flag",
            description: "テストフラグ",
            enabled: true,
            variants: [
                FlagVariant(name: "control", value: "A", weight: 50),
                FlagVariant(name: "treatment", value: "B", weight: 50),
            ]
        )
        await client.setFlag(flag)

        let retrieved = try await client.getFlag("my-flag")
        #expect(retrieved.id == "flag-4")
        #expect(retrieved.flagKey == "my-flag")
        #expect(retrieved.variants.count == 2)
    }

    @Test("存在しないフラグへのアクセスで flagNotFound エラーになること")
    func testFlagNotFoundError() async throws {
        let client = InMemoryFeatureFlagClient()
        let context = EvaluationContext()

        do {
            _ = try await client.evaluate("nonexistent-flag", context: context)
            Issue.record("エラーがスローされるべき")
        } catch let error as FeatureFlagError {
            switch error {
            case .flagNotFound(let key):
                #expect(key == "nonexistent-flag")
            }
        }
    }
}
