// TaxCalculation Domain Service の単体テスト。

using K1s0.Tier2.TaxCalculator.Domain.Services;
using K1s0.Tier2.TaxCalculator.Domain.ValueObjects;
using Xunit;

namespace K1s0.Tier2.TaxCalculator.Domain.Tests;

public class TaxCalculationTests
{
    [Fact]
    [Trait("Category", "Unit")]
    public void Exclusive_StandardRate_10Percent()
    {
        // 1000 円 × 10% = 100 円税。
        var taxable = new Money("JPY", 1000);
        var result = TaxCalculation.CalculateExclusive(taxable, TaxRate.Standard);
        Assert.Equal(1000, result.TaxableAmount.MinorAmount);
        Assert.Equal(100, result.TaxAmount.MinorAmount);
        Assert.Equal(1100, result.TotalAmount.MinorAmount);
    }

    [Fact]
    [Trait("Category", "Unit")]
    public void Exclusive_ReducedRate_8Percent()
    {
        // 1000 円 × 8% = 80 円税。
        var result = TaxCalculation.CalculateExclusive(new Money("JPY", 1000), TaxRate.Reduced);
        Assert.Equal(80, result.TaxAmount.MinorAmount);
    }

    [Fact]
    [Trait("Category", "Unit")]
    public void Exclusive_ZeroRate()
    {
        var result = TaxCalculation.CalculateExclusive(new Money("JPY", 1000), TaxRate.Zero);
        Assert.Equal(0, result.TaxAmount.MinorAmount);
        Assert.Equal(1000, result.TotalAmount.MinorAmount);
    }

    [Fact]
    [Trait("Category", "Unit")]
    public void Inclusive_RoundTrip()
    {
        // exclusive 1000 円 + 10% → inclusive 1100 円。逆算で 1000 円が戻ること。
        var inclusive = TaxCalculation.CalculateExclusive(new Money("JPY", 1000), TaxRate.Standard).TotalAmount;
        var roundTrip = TaxCalculation.CalculateInclusive(inclusive, TaxRate.Standard);
        Assert.Equal(1000, roundTrip.TaxableAmount.MinorAmount);
    }

    [Fact]
    [Trait("Category", "Unit")]
    public void TaxRate_RejectsNegative()
    {
        Assert.Throws<ArgumentException>(() => new TaxRate(-100));
    }

    [Fact]
    [Trait("Category", "Unit")]
    public void TaxRate_RejectsAbove1000Percent()
    {
        Assert.Throws<ArgumentException>(() => new TaxRate(100_001));
    }
}
