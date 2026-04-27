// CalculateTaxUseCase の単体テスト。

using K1s0.Tier2.TaxCalculator.Application.UseCases;
using Xunit;

namespace K1s0.Tier2.TaxCalculator.Application.Tests;

public class CalculateTaxTests
{
    [Fact]
    [Trait("Category", "Unit")]
    public void Execute_Exclusive()
    {
        var useCase = new CalculateTaxUseCase();
        var output = useCase.Execute(new CalculateTaxUseCase.Input("EXCLUSIVE", "JPY", 1000, 1000));
        Assert.Equal(1000, output.TaxableMinorAmount);
        Assert.Equal(100, output.TaxMinorAmount);
        Assert.Equal(1100, output.TotalMinorAmount);
    }

    [Fact]
    [Trait("Category", "Unit")]
    public void Execute_Inclusive()
    {
        var useCase = new CalculateTaxUseCase();
        var output = useCase.Execute(new CalculateTaxUseCase.Input("INCLUSIVE", "JPY", 1100, 1000));
        Assert.Equal(1000, output.TaxableMinorAmount);
        Assert.Equal(100, output.TaxMinorAmount);
        Assert.Equal(1100, output.TotalMinorAmount);
    }

    [Fact]
    [Trait("Category", "Unit")]
    public void Execute_InvalidMode_Throws()
    {
        var useCase = new CalculateTaxUseCase();
        Assert.Throws<ArgumentException>(() => useCase.Execute(new CalculateTaxUseCase.Input("UNKNOWN", "JPY", 1000, 1000)));
    }
}
