// 本ファイルは k1s0 依存方向 Analyzer の DiagnosticDescriptor 定義集。
// docs/05_実装/00_ディレクトリ設計/10_ルートレイアウト/05_依存方向ルール.md
//
// 診断 ID 命名規約: `K1S0DEPDIR0001`〜
// Severity 既定は Error（CI で TreatWarningsAsErrors と組合せ build fail）。
using Microsoft.CodeAnalysis;

namespace K1s0.DependencyDirection.Analyzer;

internal static class Diagnostics
{
    // 共通の Category / HelpLinkUri prefix。
    private const string Category = "DependencyDirection";
    private const string HelpLinkBase =
        "https://github.com/k1s0/k1s0/blob/main/docs/05_実装/00_ディレクトリ設計/10_ルートレイアウト/05_依存方向ルール.md";

    // K1S0DEPDIR0001: SDK → tier2 違反
    public static readonly DiagnosticDescriptor SdkReferencesTier2 = new(
        id: "K1S0DEPDIR0001",
        title: "SDK は tier2 を参照してはならない",
        messageFormat: "K1s0.Sdk.* アセンブリは K1s0.Tier2.* を参照できません（参照先: {0}）",
        category: Category,
        defaultSeverity: DiagnosticSeverity.Error,
        isEnabledByDefault: true,
        description: "tier3 → tier2 → sdk → tier1 の一方向ルール（IMP-DIR-ROOT-002）に違反。",
        helpLinkUri: HelpLinkBase);

    // K1S0DEPDIR0002: SDK → tier3 違反
    public static readonly DiagnosticDescriptor SdkReferencesTier3 = new(
        id: "K1S0DEPDIR0002",
        title: "SDK は tier3 を参照してはならない",
        messageFormat: "K1s0.Sdk.* アセンブリは K1s0.Tier3.* を参照できません（参照先: {0}）",
        category: Category,
        defaultSeverity: DiagnosticSeverity.Error,
        isEnabledByDefault: true,
        description: "tier3 → tier2 → sdk → tier1 の一方向ルール（IMP-DIR-ROOT-002）に違反。",
        helpLinkUri: HelpLinkBase);

    // K1S0DEPDIR0003: tier2 → tier3 違反
    public static readonly DiagnosticDescriptor Tier2ReferencesTier3 = new(
        id: "K1S0DEPDIR0003",
        title: "tier2 は tier3 を参照してはならない",
        messageFormat: "K1s0.Tier2.* アセンブリは K1s0.Tier3.* を参照できません（参照先: {0}）",
        category: Category,
        defaultSeverity: DiagnosticSeverity.Error,
        isEnabledByDefault: true,
        description: "tier3 → tier2 → sdk → tier1 の一方向ルール（IMP-DIR-ROOT-002）に違反。",
        helpLinkUri: HelpLinkBase);

    // K1S0DEPDIR0004: tier1 → tier2 / tier3 / sdk 違反
    public static readonly DiagnosticDescriptor Tier1ReferencesUpperLayers = new(
        id: "K1S0DEPDIR0004",
        title: "tier1 は上位層（tier2 / tier3 / sdk）を参照してはならない",
        messageFormat: "K1s0.Tier1.* アセンブリは {0} を参照できません",
        category: Category,
        defaultSeverity: DiagnosticSeverity.Error,
        isEnabledByDefault: true,
        description: "tier1 は contracts のみに依存し、sdk / tier2 / tier3 を参照しない（IMP-DIR-ROOT-002）。",
        helpLinkUri: HelpLinkBase);
}
