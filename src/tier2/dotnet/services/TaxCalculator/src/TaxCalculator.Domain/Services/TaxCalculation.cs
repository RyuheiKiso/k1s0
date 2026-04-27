// 税計算 Domain Service。
//
// 役割:
//   税抜金額と税率から税額を導出する。整数 minor unit + basis points で計算し、
//   丸め誤差を避ける。リリース時点 では「銀行家の丸め」（half-to-even）固定、
//   リリース時点 で複数丸めポリシーをオプション化する。

using K1s0.Tier2.TaxCalculator.Domain.ValueObjects;

namespace K1s0.Tier2.TaxCalculator.Domain.Services;

// TaxCalculation は税抜金額 / 税率 → 税額 + 税込金額の計算を提供する。
public static class TaxCalculation
{
    // 計算結果。
    public readonly record struct Result(Money TaxableAmount, Money TaxAmount, Money TotalAmount, TaxRate AppliedRate);

    // exclusive（税抜）金額と税率から税額・税込金額を計算する。
    public static Result CalculateExclusive(Money taxable, TaxRate rate)
    {
        // 税額 = taxable * basisPoints / 10000。
        // long での乗算後に整数除算する。half-to-even（銀行家丸め）相当の挙動を Math.DivRem で実装する。
        var num = taxable.MinorAmount * (long)rate.BasisPoints;
        var quotient = Math.DivRem(num, 10000L, out var remainder);
        // 半端値の処理（half-to-even）。
        if (remainder * 2 > 10000)
        {
            // 半端が 0.5 超なら切り上げ。
            quotient += 1;
        }
        else if (remainder * 2 == 10000)
        {
            // ちょうど 0.5 の時は偶数側に倒す。
            if (quotient % 2 != 0)
            {
                quotient += 1;
            }
        }
        // remainder * 2 < 10000 はそのまま切り捨て。
        var taxAmount = new Money(taxable.Currency, quotient);
        var totalAmount = new Money(taxable.Currency, taxable.MinorAmount + taxAmount.MinorAmount);
        return new Result(taxable, taxAmount, totalAmount, rate);
    }

    // inclusive（税込）金額と税率から税抜・税額を逆算する。
    public static Result CalculateInclusive(Money inclusive, TaxRate rate)
    {
        // 税抜 = inclusive * 10000 / (10000 + basisPoints)。
        var denom = 10000L + rate.BasisPoints;
        var num = inclusive.MinorAmount * 10000L;
        var taxableMinor = Math.DivRem(num, denom, out var remainder);
        // half-to-even。
        if (remainder * 2 > denom)
        {
            taxableMinor += 1;
        }
        else if (remainder * 2 == denom)
        {
            if (taxableMinor % 2 != 0)
            {
                taxableMinor += 1;
            }
        }
        var taxableAmount = new Money(inclusive.Currency, taxableMinor);
        var taxAmount = new Money(inclusive.Currency, inclusive.MinorAmount - taxableMinor);
        return new Result(taxableAmount, taxAmount, inclusive, rate);
    }
}
