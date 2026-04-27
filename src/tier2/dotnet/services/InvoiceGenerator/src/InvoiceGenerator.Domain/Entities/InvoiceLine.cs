// 請求書 1 行を表すエンティティ。

using K1s0.Tier2.InvoiceGenerator.Domain.ValueObjects;

namespace K1s0.Tier2.InvoiceGenerator.Domain.Entities;

// InvoiceLine は商品 / サービス 1 行。
public sealed class InvoiceLine
{
    // 商品コード / 説明。
    public string Description { get; }
    // 数量（正の整数）。
    public int Quantity { get; }
    // 単価（Money）。
    public Money UnitPrice { get; }

    public InvoiceLine(string description, int quantity, Money unitPrice)
    {
        if (string.IsNullOrWhiteSpace(description))
        {
            throw new ArgumentException("description is required", nameof(description));
        }
        if (quantity <= 0)
        {
            throw new ArgumentException("quantity must be > 0", nameof(quantity));
        }
        if (unitPrice.MinorAmount < 0)
        {
            throw new ArgumentException("unit price must be >= 0", nameof(unitPrice));
        }
        Description = description.Trim();
        Quantity = quantity;
        UnitPrice = unitPrice;
    }

    // 行の小計（unit price × quantity）。
    public Money Subtotal() => new(UnitPrice.Currency, UnitPrice.MinorAmount * Quantity);
}
