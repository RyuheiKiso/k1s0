// 金額値オブジェクト。

namespace K1s0.Tier2.InvoiceGenerator.Domain.ValueObjects;

// Money は通貨と minor unit 整数の組。
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
        Currency = currency.Trim().ToUpperInvariant();
        MinorAmount = minorAmount;
    }

    // 同一通貨同士の加算。
    public Money Add(Money other)
    {
        if (!string.Equals(Currency, other.Currency, StringComparison.Ordinal))
        {
            throw new InvalidOperationException($"currency mismatch: {Currency} vs {other.Currency}");
        }
        return new Money(Currency, MinorAmount + other.MinorAmount);
    }

    public static Money Zero(string currency) => new(currency, 0);
    public override string ToString() => $"{Currency}:{MinorAmount}";
}
