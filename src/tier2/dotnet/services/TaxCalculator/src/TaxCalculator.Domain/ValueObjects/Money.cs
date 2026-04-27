// 金額値オブジェクト。

namespace K1s0.Tier2.TaxCalculator.Domain.ValueObjects;

public readonly record struct Money
{
    public string Currency { get; }
    public long MinorAmount { get; }

    public Money(string currency, long minorAmount)
    {
        if (string.IsNullOrWhiteSpace(currency))
        {
            throw new ArgumentException("currency is required", nameof(currency));
        }
        if (minorAmount < 0)
        {
            throw new ArgumentException("minorAmount must be >= 0", nameof(minorAmount));
        }
        Currency = currency.Trim().ToUpperInvariant();
        MinorAmount = minorAmount;
    }

    public override string ToString() => $"{Currency}:{MinorAmount}";
}
