// 金額値オブジェクト。通貨と数量をまとめて扱う。

namespace K1s0.Tier2.ApprovalFlow.Domain.ValueObjects;

// Money は通貨コード（ISO 4217）と整数 minor unit（円なら 100 = 100 円、ドルなら 100 = 1.00 USD）を持つ。
public readonly record struct Money
{
    // 通貨コード（"JPY" / "USD"）。
    public string Currency { get; }

    // minor unit（最小通貨単位での値）。負数も許容（返金等で使う）。
    public long MinorAmount { get; }

    // コンストラクタ。
    public Money(string currency, long minorAmount)
    {
        // 通貨コードは必須。
        if (string.IsNullOrWhiteSpace(currency))
        {
            throw new ArgumentException("currency is required", nameof(currency));
        }
        // 大文字に正規化する。
        Currency = currency.Trim().ToUpperInvariant();
        // 数量はそのまま保持。
        MinorAmount = minorAmount;
    }

    // 同一通貨同士の加算。通貨が異なれば InvalidOperationException。
    public Money Add(Money other)
    {
        if (!string.Equals(Currency, other.Currency, StringComparison.Ordinal))
        {
            throw new InvalidOperationException($"currency mismatch: {Currency} vs {other.Currency}");
        }
        return new Money(Currency, MinorAmount + other.MinorAmount);
    }

    // 文字列表現は "JPY:1000" のように通貨と数量を : で連結。
    public override string ToString() => $"{Currency}:{MinorAmount}";
}
