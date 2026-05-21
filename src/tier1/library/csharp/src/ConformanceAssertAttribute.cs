// ConformanceAssertAttribute.cs — spec 01 §assertion id の連結
// docs/04_詳細設計/01_適合仕様/01_Bidi適合仕様.md §scenarios.yaml の assertion id を
// C# test runner が claim するための型安全デコレータ attribute を提供する。
// Rust conformance_assert.rs / Go conformance_assert.go /
// TypeScript conformanceAssert.ts と 4 言語等価強度を保つ。

// System: Attribute 基底クラス / AttributeUsageAttribute に使用する
using System;

// k1s0 tier1 名前空間
namespace K1s0.Tier1;

/// <summary>
/// ConformanceAssertAttribute は scenarios.yaml の assertion id を
/// C# テストメソッドに静的に対応づける attribute。
/// CI 整合 2「scenarios.yaml の assertion id が全 class × 全 adapter × 全言語で実装されていること」
/// の C# 物理機構として機能する。
/// </summary>
/// <example>
/// <code>
/// [ConformanceAssert("bidi_v1_interactive__grpc_native_assert_001")]
/// public void Test_PlSeqMonotonic_GrpcNative() { /* test body */ }
/// </code>
/// </example>
// AttributeUsage: Method のみに適用可能（AllowMultiple=false で重複宣言を禁止する）
[AttributeUsage(AttributeTargets.Method, AllowMultiple = false)]
public sealed class ConformanceAssertAttribute : Attribute
{
    /// <summary>
    /// AssertId は scenarios.yaml の assertion フィールドの値を保持する。
    /// 例: "pl_seq_monotonic_per_session" / "rd_seq_continuous_across_resume"
    /// </summary>
    // AssertId プロパティ: assertion id 文字列を読み取り専用で保持する
    public string AssertId { get; }

    /// <summary>
    /// ConformanceAssertAttribute を assertion id で初期化する。
    /// </summary>
    /// <param name="assertId">
    /// scenarios.yaml の assertion フィールドに対応する assertion id 文字列。
    /// 例: "pl_seq_monotonic_per_session"
    /// </param>
    // コンストラクタ: assertId 引数から AssertId プロパティを初期化する
    public ConformanceAssertAttribute(string assertId)
    {
        // assertId が null の場合は ArgumentNullException を投げる（型安全性の物理保証）
        AssertId = assertId ?? throw new ArgumentNullException(nameof(assertId),
            "ConformanceAssertAttribute の assertId は null 禁止（scenarios.yaml 整合要件）");
    }
}
