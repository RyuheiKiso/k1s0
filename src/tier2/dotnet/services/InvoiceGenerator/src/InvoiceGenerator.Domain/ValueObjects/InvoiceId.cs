// 請求書識別子の値オブジェクト。

namespace K1s0.Tier2.InvoiceGenerator.Domain.ValueObjects;

// InvoiceId は Guid をラップする値オブジェクト。
public readonly record struct InvoiceId(Guid Value)
{
    public static InvoiceId NewId() => new(Guid.NewGuid());
    public static InvoiceId Parse(string s) => new(Guid.Parse(s));
    public override string ToString() => Value.ToString();
}
