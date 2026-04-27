// ValueObject 群の単体テスト。

using K1s0.Tier2.ApprovalFlow.Domain.ValueObjects;
using Xunit;

namespace K1s0.Tier2.ApprovalFlow.Domain.Tests;

// ApprovalStatus / Money / ApprovalId の振る舞いを検証する。
public class ValueObjectsTests
{
    [Fact]
    [Trait("Category", "Unit")]
    public void ApprovalStatus_Parse_KnownValues()
    {
        Assert.Equal(ApprovalStatus.Pending, ApprovalStatus.Parse("PENDING"));
        Assert.Equal(ApprovalStatus.Approved, ApprovalStatus.Parse("approved"));
        Assert.Equal(ApprovalStatus.Rejected, ApprovalStatus.Parse(" REJECTED "));
        Assert.Equal(ApprovalStatus.Cancelled, ApprovalStatus.Parse("Cancelled"));
    }

    [Fact]
    [Trait("Category", "Unit")]
    public void ApprovalStatus_Parse_UnknownThrows()
    {
        Assert.Throws<ArgumentException>(() => ApprovalStatus.Parse("STARTED"));
        Assert.Throws<ArgumentException>(() => ApprovalStatus.Parse(""));
    }

    [Fact]
    [Trait("Category", "Unit")]
    public void Money_Add_SameCurrency()
    {
        var a = new Money("JPY", 1000);
        var b = new Money("JPY", 500);
        Assert.Equal(1500, a.Add(b).MinorAmount);
    }

    [Fact]
    [Trait("Category", "Unit")]
    public void Money_Add_DifferentCurrencyThrows()
    {
        var a = new Money("JPY", 1000);
        var b = new Money("USD", 500);
        Assert.Throws<InvalidOperationException>(() => a.Add(b));
    }

    [Fact]
    [Trait("Category", "Unit")]
    public void Money_RequiresCurrency()
    {
        Assert.Throws<ArgumentException>(() => new Money("", 1000));
    }

    [Fact]
    [Trait("Category", "Unit")]
    public void ApprovalId_RoundTrip()
    {
        var id = ApprovalId.NewId();
        var parsed = ApprovalId.Parse(id.ToString());
        Assert.Equal(id, parsed);
    }
}
