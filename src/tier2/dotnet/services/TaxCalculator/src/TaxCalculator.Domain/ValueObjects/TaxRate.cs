// 税率の値オブジェクト。

namespace K1s0.Tier2.TaxCalculator.Domain.ValueObjects;

// TaxRate は basis points（1/100 of 1%、すなわち 1000 = 10%）で税率を保持する。
//
// 浮動小数の丸め誤差を避けるため、整数 basis points（bp）で値を保持する。
public readonly record struct TaxRate
{
    // basis points 値（100 = 1%）。
    public int BasisPoints { get; }

    public TaxRate(int basisPoints)
    {
        if (basisPoints < 0)
        {
            throw new ArgumentException("basisPoints must be >= 0", nameof(basisPoints));
        }
        if (basisPoints > 100_000)
        {
            // 1000% を超える税率は誤入力とみなす。
            throw new ArgumentException("basisPoints must be <= 100000 (1000%)", nameof(basisPoints));
        }
        BasisPoints = basisPoints;
    }

    // 1000 basis points = 10% で固定（人間可読）。
    public override string ToString() => $"{BasisPoints / 100m:F2}%";

    // 既知の標準税率定数（消費税 10% / 軽減 8% / 0%）。
    public static readonly TaxRate Standard = new(1000);
    public static readonly TaxRate Reduced = new(800);
    public static readonly TaxRate Zero = new(0);
}
