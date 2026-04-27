// Invoice エンティティの単体テスト。

using K1s0.Tier2.InvoiceGenerator.Domain.Entities;
using K1s0.Tier2.InvoiceGenerator.Domain.ValueObjects;
using Xunit;

namespace K1s0.Tier2.InvoiceGenerator.Domain.Tests;

public class InvoiceTests
{
    private static DateTimeOffset Now() => new(2026, 4, 27, 12, 0, 0, TimeSpan.Zero);

    [Fact]
    [Trait("Category", "Unit")]
    public void Create_ComputesTotal()
    {
        var lines = new List<InvoiceLine>
        {
            new("apple", 3, new Money("JPY", 100)),
            new("banana", 2, new Money("JPY", 200)),
        };
        var invoice = Invoice.Create("Acme", lines, Now());
        // 3*100 + 2*200 = 700。
        Assert.Equal(700, invoice.Total.MinorAmount);
        Assert.Equal("JPY", invoice.Total.Currency);
        Assert.Equal("Acme", invoice.Customer);
    }

    [Fact]
    [Trait("Category", "Unit")]
    public void Create_RequiresLines()
    {
        Assert.Throws<ArgumentException>(() => Invoice.Create("Acme", new List<InvoiceLine>(), Now()));
    }

    [Fact]
    [Trait("Category", "Unit")]
    public void Create_RejectsMixedCurrency()
    {
        var lines = new List<InvoiceLine>
        {
            new("a", 1, new Money("JPY", 100)),
            new("b", 1, new Money("USD", 100)),
        };
        Assert.Throws<InvalidOperationException>(() => Invoice.Create("Acme", lines, Now()));
    }

    [Fact]
    [Trait("Category", "Unit")]
    public void InvoiceLine_RequiresPositiveQuantity()
    {
        Assert.Throws<ArgumentException>(() => new InvoiceLine("a", 0, new Money("JPY", 100)));
        Assert.Throws<ArgumentException>(() => new InvoiceLine("a", -1, new Money("JPY", 100)));
    }
}
