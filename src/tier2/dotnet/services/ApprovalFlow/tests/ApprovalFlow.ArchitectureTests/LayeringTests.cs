// 層間依存方向の Architecture テスト。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/04_サービス単位の内部構造.md

using K1s0.Tier2.ApprovalFlow.Domain;
using K1s0.Tier2.ApprovalFlow.Application.UseCases;
using K1s0.Tier2.ApprovalFlow.Infrastructure.Persistence;
using NetArchTest.Rules;
using Xunit;

namespace K1s0.Tier2.ApprovalFlow.ArchitectureTests;

// Onion 4 層構成の依存方向を強制する（dotnet test --filter Category=Architecture）。
public class LayeringTests
{
    [Fact]
    [Trait("Category", "Architecture")]
    public void Domain_Should_Not_Depend_On_Application()
    {
        var result = Types
            .InAssembly(typeof(DomainMarker).Assembly)
            .Should()
            .NotHaveDependencyOn("K1s0.Tier2.ApprovalFlow.Application")
            .GetResult();
        Assert.True(result.IsSuccessful, string.Join(", ", result.FailingTypeNames ?? Array.Empty<string>()));
    }

    [Fact]
    [Trait("Category", "Architecture")]
    public void Domain_Should_Not_Depend_On_Infrastructure()
    {
        var result = Types
            .InAssembly(typeof(DomainMarker).Assembly)
            .Should()
            .NotHaveDependencyOn("K1s0.Tier2.ApprovalFlow.Infrastructure")
            .GetResult();
        Assert.True(result.IsSuccessful, string.Join(", ", result.FailingTypeNames ?? Array.Empty<string>()));
    }

    [Fact]
    [Trait("Category", "Architecture")]
    public void Application_Should_Not_Depend_On_Infrastructure()
    {
        var result = Types
            .InAssembly(typeof(SubmitApprovalUseCase).Assembly)
            .Should()
            .NotHaveDependencyOn("K1s0.Tier2.ApprovalFlow.Infrastructure")
            .GetResult();
        Assert.True(result.IsSuccessful, string.Join(", ", result.FailingTypeNames ?? Array.Empty<string>()));
    }

    [Fact]
    [Trait("Category", "Architecture")]
    public void Infrastructure_Should_Not_Depend_On_Application()
    {
        var result = Types
            .InAssembly(typeof(InMemoryApprovalRepository).Assembly)
            .Should()
            .NotHaveDependencyOn("K1s0.Tier2.ApprovalFlow.Application")
            .GetResult();
        Assert.True(result.IsSuccessful, string.Join(", ", result.FailingTypeNames ?? Array.Empty<string>()));
    }
}
