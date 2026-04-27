// 税計算ユースケース。

using K1s0.Tier2.TaxCalculator.Domain.Services;
using K1s0.Tier2.TaxCalculator.Domain.ValueObjects;

namespace K1s0.Tier2.TaxCalculator.Application.UseCases;

// CalculateTaxUseCase は exclusive / inclusive モード両対応の計算ユースケース。
public sealed class CalculateTaxUseCase
{
    // 入力 DTO。Mode は "EXCLUSIVE" または "INCLUSIVE"。
    public sealed record Input(string Mode, string Currency, long MinorAmount, int RateBasisPoints);

    // 出力 DTO。
    public sealed record Output(long TaxableMinorAmount, long TaxMinorAmount, long TotalMinorAmount, string Currency, int AppliedRateBasisPoints);

    // 計算する。
    // CA1822: 本クラスは将来的に依存性注入（時刻取得・rate provider 等）を受ける
    // インスタンスメソッドとして拡張する想定。リリース時点では state を持たないが
    // インスタンスメソッドのまま残す（CA1822 を NoWarn する）。
    [System.Diagnostics.CodeAnalysis.SuppressMessage(
        "Performance", "CA1822:Mark members as static",
        Justification = "DI 拡張を見据えてインスタンスメソッドのまま残す")]
    public Output Execute(Input input)
    {
        // CA1062: 公開 API では引数 null を必ず明示検証する。
        ArgumentNullException.ThrowIfNull(input);
        // Money / TaxRate を構築する（バリデーション含む）。
        var money = new Money(input.Currency, input.MinorAmount);
        var rate = new TaxRate(input.RateBasisPoints);
        // モードで分岐する。
        TaxCalculationResult result = input.Mode?.Trim().ToUpperInvariant() switch
        {
            "EXCLUSIVE" => TaxCalculation.CalculateExclusive(money, rate),
            "INCLUSIVE" => TaxCalculation.CalculateInclusive(money, rate),
            _ => throw new ArgumentException($"invalid mode: {input.Mode}", nameof(input)),
        };
        return new Output(result.TaxableAmount.MinorAmount, result.TaxAmount.MinorAmount, result.TotalAmount.MinorAmount, result.TaxableAmount.Currency, result.AppliedRate.BasisPoints);
    }
}
