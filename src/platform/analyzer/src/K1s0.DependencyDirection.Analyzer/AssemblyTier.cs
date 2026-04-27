// 本ファイルはアセンブリ名から tier を判定する共通ヘルパ。
// Compilation.AssemblyName / TypeSymbol.ContainingAssembly.Name に対して prefix で比較する。
namespace K1s0.DependencyDirection.Analyzer;

// k1s0 リポジトリの tier 区分。
internal enum Tier
{
    // 区分外（外部 OSS / BCL 等、解析対象外）
    Unknown,
    // K1s0.Sdk.* （sdk 層）
    Sdk,
    // K1s0.Tier1.* （tier1 層）
    Tier1,
    // K1s0.Tier2.* （tier2 層）
    Tier2,
    // K1s0.Tier3.* （tier3 層）
    Tier3,
}

// 静的ヘルパ。
internal static class AssemblyTierResolver
{
    // アセンブリ名（例 K1s0.Sdk.Grpc / K1s0.Tier2.ApprovalFlow）から Tier を判定する。
    public static Tier ResolveFromAssemblyName(string? assemblyName)
    {
        if (string.IsNullOrEmpty(assemblyName))
        {
            return Tier.Unknown;
        }
        // prefix で k1s0 内の tier を識別する。
        if (assemblyName!.StartsWith("K1s0.Sdk", System.StringComparison.Ordinal))
        {
            return Tier.Sdk;
        }
        if (assemblyName.StartsWith("K1s0.Tier1", System.StringComparison.Ordinal))
        {
            return Tier.Tier1;
        }
        if (assemblyName.StartsWith("K1s0.Tier2", System.StringComparison.Ordinal))
        {
            return Tier.Tier2;
        }
        if (assemblyName.StartsWith("K1s0.Tier3", System.StringComparison.Ordinal))
        {
            return Tier.Tier3;
        }
        return Tier.Unknown;
    }

    // 名前空間文字列（例 K1s0.Tier3.Web）から Tier を判定する。
    public static Tier ResolveFromNamespace(string? fullNamespace)
    {
        // アセンブリと同じロジックで再利用する。
        return ResolveFromAssemblyName(fullNamespace);
    }

    // current → referenced への参照が許容されているかを判定する。
    // 許容 (true): 同 tier / 上位 → 下位（tier3→tier2→sdk→tier1）
    // 違反 (false): tier3→tier1 直接 / sdk→tier2 / tier1→tier2 等
    public static bool IsAllowed(Tier current, Tier referenced)
    {
        // Unknown は解析対象外（外部 OSS）
        if (current == Tier.Unknown || referenced == Tier.Unknown)
        {
            return true;
        }
        // 同 tier 内は常に OK
        if (current == referenced)
        {
            return true;
        }
        return current switch
        {
            // sdk は contracts 経由のみ許容（tier1 / tier2 / tier3 を参照しない）
            Tier.Sdk => false,
            // tier1 は contracts 経由のみ（sdk / tier2 / tier3 を参照しない）
            Tier.Tier1 => false,
            // tier2 は sdk / contracts のみ参照可
            Tier.Tier2 => referenced == Tier.Sdk,
            // tier3 は tier2 / sdk のみ参照可（tier1 / contracts を直接参照禁止）
            Tier.Tier3 => referenced == Tier.Tier2 || referenced == Tier.Sdk,
            _ => true,
        };
    }
}
