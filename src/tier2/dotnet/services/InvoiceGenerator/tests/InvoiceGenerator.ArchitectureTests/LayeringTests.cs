// 層間依存方向の Architecture テスト。

using K1s0.Tier2.InvoiceGenerator.Domain;
using K1s0.Tier2.InvoiceGenerator.Application.UseCases;
using K1s0.Tier2.InvoiceGenerator.Infrastructure.Persistence;
using NetArchTest.Rules;
using Xunit;

namespace K1s0.Tier2.InvoiceGenerator.ArchitectureTests;

public class LayeringTests
{
    [Fact]
    [Trait("Category", "Architecture")]
    public void Domain_Should_Not_Depend_On_Application()
    {
        var result = Types.InAssembly(typeof(DomainMarker).Assembly).Should().NotHaveDependencyOn("K1s0.Tier2.InvoiceGenerator.Application").GetResult();
        Assert.True(result.IsSuccessful, string.Join(", ", result.FailingTypeNames ?? Array.Empty<string>()));
    }

    [Fact]
    [Trait("Category", "Architecture")]
    public void Domain_Should_Not_Depend_On_Infrastructure()
    {
        var result = Types.InAssembly(typeof(DomainMarker).Assembly).Should().NotHaveDependencyOn("K1s0.Tier2.InvoiceGenerator.Infrastructure").GetResult();
        Assert.True(result.IsSuccessful, string.Join(", ", result.FailingTypeNames ?? Array.Empty<string>()));
    }

    [Fact]
    [Trait("Category", "Architecture")]
    public void Application_Should_Not_Depend_On_Infrastructure()
    {
        var result = Types.InAssembly(typeof(GenerateInvoiceUseCase).Assembly).Should().NotHaveDependencyOn("K1s0.Tier2.InvoiceGenerator.Infrastructure").GetResult();
        Assert.True(result.IsSuccessful, string.Join(", ", result.FailingTypeNames ?? Array.Empty<string>()));
    }

    [Fact]
    [Trait("Category", "Architecture")]
    public void Infrastructure_Should_Not_Depend_On_Application()
    {
        var result = Types.InAssembly(typeof(InMemoryInvoiceRepository).Assembly).Should().NotHaveDependencyOn("K1s0.Tier2.InvoiceGenerator.Application").GetResult();
        Assert.True(result.IsSuccessful, string.Join(", ", result.FailingTypeNames ?? Array.Empty<string>()));
    }
}
