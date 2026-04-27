// 請求書集約ルート。

using K1s0.Tier2.InvoiceGenerator.Domain.ValueObjects;

namespace K1s0.Tier2.InvoiceGenerator.Domain.Entities;

// Invoice は請求書 1 件を表す集約ルート。
public sealed class Invoice
{
    public InvoiceId Id { get; }
    public string Customer { get; }
    public IReadOnlyList<InvoiceLine> Lines { get; }
    public Money Total { get; }
    public DateTimeOffset IssuedAt { get; }

    private Invoice(InvoiceId id, string customer, IReadOnlyList<InvoiceLine> lines, Money total, DateTimeOffset issuedAt)
    {
        Id = id;
        Customer = customer;
        Lines = lines;
        Total = total;
        IssuedAt = issuedAt;
    }

    // 行リストから Invoice を生成する。
    public static Invoice Create(string customer, IReadOnlyList<InvoiceLine> lines, DateTimeOffset issuedAt)
    {
        // CA1062: 公開 API では参照型引数の null を必ず明示検証する。
        ArgumentNullException.ThrowIfNull(lines);
        if (string.IsNullOrWhiteSpace(customer))
        {
            throw new ArgumentException("customer is required", nameof(customer));
        }
        if (lines.Count == 0)
        {
            throw new ArgumentException("at least one line is required", nameof(lines));
        }
        // 全行で通貨が一致していること。
        var currency = lines[0].UnitPrice.Currency;
        if (lines.Any(l => !string.Equals(l.UnitPrice.Currency, currency, StringComparison.Ordinal)))
        {
            throw new InvalidOperationException("all lines must share the same currency");
        }
        // 合計を計算する。
        var total = Money.Zero(currency);
        foreach (var line in lines)
        {
            total = total.Add(line.Subtotal());
        }
        return new Invoice(InvoiceId.NewId(), customer.Trim(), lines, total, issuedAt);
    }
}
