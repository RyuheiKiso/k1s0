// 本ファイルは依存方向 Analyzer の golden test。違反 / 非違反コードに対する
// 期待 diagnostic を verifier 経由で検証する。
using Xunit;
using Microsoft.CodeAnalysis.CSharp.Testing;
using Microsoft.CodeAnalysis.Testing;
using Verifier = Microsoft.CodeAnalysis.CSharp.Testing.CSharpAnalyzerTest<
    K1s0.DependencyDirection.Analyzer.DependencyDirectionAnalyzer,
    Microsoft.CodeAnalysis.Testing.DefaultVerifier>;

namespace K1s0.DependencyDirection.Analyzer.Tests;

public class DependencyDirectionAnalyzerTests
{
    // ケース 1: 違反なし（同じ tier 内の参照）
    [Fact]
    public async Task SameTier_NoDiagnostic()
    {
        var test = new Verifier
        {
            // current アセンブリ K1s0.Sdk.Smoke が同 tier の K1s0.Sdk.Other を参照（OK）
            TestCode = """
            namespace K1s0.Sdk.Smoke
            {
                public class A
                {
                    public void Use(K1s0.Sdk.Other.B b) { _ = b; }
                }
            }
            namespace K1s0.Sdk.Other
            {
                public class B { }
            }
            """,
            SolutionTransforms = { (sol, projId) => sol.WithProjectAssemblyName(projId, "K1s0.Sdk.Smoke") },
        };
        await test.RunAsync();
    }

    // ケース 2: SDK → Tier2 違反（K1S0DEPDIR0001）
    [Fact]
    public async Task SdkReferencingTier2_ReportsDiagnostic()
    {
        var test = new Verifier
        {
            TestCode = """
            namespace K1s0.Sdk.Smoke
            {
                public class A
                {
                    public void Use(K1s0.Tier2.ApprovalFlow.B b) { _ = b; }
                }
            }
            namespace K1s0.Tier2.ApprovalFlow
            {
                public class B { }
            }
            """,
            SolutionTransforms = { (sol, projId) => sol.WithProjectAssemblyName(projId, "K1s0.Sdk.Smoke") },
        };
        // current=Sdk, referenced=Tier2 で K1S0DEPDIR0001 が必ず 1 件以上出る期待。
        test.ExpectedDiagnostics.Add(
            DiagnosticResult.CompilerWarning("K1S0DEPDIR0001").WithDefaultPath("/0/Test0.cs"));
        await test.RunAsync();
    }
}
