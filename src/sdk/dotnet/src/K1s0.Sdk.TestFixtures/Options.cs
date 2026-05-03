// src/sdk/dotnet/src/K1s0.Sdk.TestFixtures/Options.cs
//
// k1s0 .NET SDK test-fixtures: Options / Stack 定義。
// 4 言語対称 API の field 名を C# イディオム（PascalCase）で実装する。
//
// 設計正典:
//   ADR-TEST-010
//   docs/05_実装/30_CI_CD設計/35_e2e_test_design/30_test_fixtures/01_4言語対称API.md

namespace K1s0.Sdk.TestFixtures;

// kind cluster に install する k1s0 stack の規模
public enum Stack
{
    // Dapr + tier1 facade + Keycloak + 1 backend のみ install
    Minimum,
    // user suite 任意 stack 全部入り（owner 経路ではない）
    Full,
}

// Setup の動作を制御するパラメータ
//
// 4 言語対称化のため field 名は対応関係を保つ:
//   Go: KindNodes / Stack / AddOns / Tenant / Namespace
//   .NET: KindNodes / Stack / AddOns / Tenant / Namespace
public sealed class Options
{
    // kind cluster の node 数（既定 2）
    public int KindNodes { get; init; } = 2;

    // install する k1s0 stack（既定 Minimum）
    public Stack Stack { get; init; } = Stack.Minimum;

    // Setup 時に追加で install する任意 component の名前一覧
    public IReadOnlyList<string> AddOns { get; init; } = Array.Empty<string>();

    // デフォルトの tenant ID
    public string Tenant { get; init; } = "tenant-a";

    // k1s0 install 先 namespace
    public string Namespace { get; init; } = "k1s0";
}
