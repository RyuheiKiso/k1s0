// 本ファイルは k1s0 依存方向 Roslyn DiagnosticAnalyzer。
//
// 解析対象: SimpleName / Qualified name / using directive など、シンボルが具体的な
// 名前空間を参照する箇所。SymbolAction で SymbolKind.NamedType / Method / Property
// それぞれが参照するシンボルの ContainingAssembly を tier 判定し、許容方向か検証する。
//
// 設計正典:
//   docs/05_実装/00_ディレクトリ設計/10_ルートレイアウト/05_依存方向ルール.md（IMP-DIR-ROOT-002）
//   docs/05_実装/10_ビルド設計/40_dotnet_sln境界/01_dotnet_sln境界.md
using System.Collections.Immutable;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.Diagnostics;

namespace K1s0.DependencyDirection.Analyzer;

[DiagnosticAnalyzer(LanguageNames.CSharp)]
public sealed class DependencyDirectionAnalyzer : DiagnosticAnalyzer
{
    // 診断対象一覧（Roslyn host が起動時に列挙する）
    public override ImmutableArray<DiagnosticDescriptor> SupportedDiagnostics =>
        ImmutableArray.Create(
            Diagnostics.SdkReferencesTier2,
            Diagnostics.SdkReferencesTier3,
            Diagnostics.Tier2ReferencesTier3,
            Diagnostics.Tier1ReferencesUpperLayers);

    public override void Initialize(AnalysisContext context)
    {
        // 並列実行を許可し、生成コードは解析対象外とする。
        context.ConfigureGeneratedCodeAnalysis(GeneratedCodeAnalysisFlags.None);
        context.EnableConcurrentExecution();

        // Compilation 開始時に current アセンブリの tier を確定し、SyntaxNodeAction を登録。
        context.RegisterCompilationStartAction(static startCtx =>
        {
            var currentAssembly = startCtx.Compilation.AssemblyName;
            var currentTier = AssemblyTierResolver.ResolveFromAssemblyName(currentAssembly);
            // current が解析対象外（外部 OSS / 名前が k1s0 規約外）なら何もしない。
            if (currentTier == Tier.Unknown)
            {
                return;
            }

            // IdentifierName のみを解析対象とする。QualifiedName / MemberAccess も
            // 登録すると同じ型参照に対して複数 diagnostic が発火し、IDE / テスト側の
            // expected count と食い違うため、末端の identifier だけに絞る。
            // namespace 識別子は Analyze 内で skip する。
            startCtx.RegisterSyntaxNodeAction(
                ctx => Analyze(ctx, currentTier),
                SyntaxKind.IdentifierName);
        });
    }

    // SyntaxNode 単位での解析。
    private static void Analyze(SyntaxNodeAnalysisContext context, Tier currentTier)
    {
        var symbolInfo = context.SemanticModel.GetSymbolInfo(context.Node, context.CancellationToken);
        var referenced = symbolInfo.Symbol;
        if (referenced is null)
        {
            return;
        }
        // namespace 識別子は tier 判定の対象外。型 / メンバの末端 identifier のみで
        // 違反を判定し、同一参照に対する重複 diagnostic を防ぐ。
        if (referenced.Kind == SymbolKind.Namespace)
        {
            return;
        }

        // 参照先の tier 判定。**namespace を最優先**で見て、namespace 不明時のみ
        // assembly 名で fallback する。namespace 優先の理由は次のとおり:
        //   - 同一 assembly に複数 tier の namespace を含む配置（テスト・mono-repo 単一
        //     プロジェクト等）でも、namespace prefix で論理 tier を識別できる。
        //   - 別 assembly 配置（K1s0.Tier2.* 等）でも namespace = assembly 名と一致する
        //     のが通常で、namespace fallback と assembly 解決の結果は一致する。
        var referencedNamespace = referenced.ContainingNamespace?.ToDisplayString();
        var referencedTier = AssemblyTierResolver.ResolveFromNamespace(referencedNamespace);
        if (referencedTier == Tier.Unknown)
        {
            var referencedAssembly = referenced.ContainingAssembly?.Name;
            referencedTier = AssemblyTierResolver.ResolveFromAssemblyName(referencedAssembly);
            if (referencedTier == Tier.Unknown)
            {
                return;
            }
        }
        // 違反 message の参照先表示は namespace を優先（より具体的）。
        var referencedDisplay = !string.IsNullOrEmpty(referencedNamespace)
            ? referencedNamespace!
            : (referenced.ContainingAssembly?.Name ?? "(unknown)");

        // 許容方向であれば warning なし。
        if (AssemblyTierResolver.IsAllowed(currentTier, referencedTier))
        {
            return;
        }

        // 違反: 適切な DiagnosticDescriptor を選択して報告する。
        var descriptor = SelectDescriptor(currentTier, referencedTier);
        if (descriptor is null)
        {
            return;
        }

        context.ReportDiagnostic(
            Diagnostic.Create(
                descriptor,
                context.Node.GetLocation(),
                referencedDisplay));
    }

    // current/referenced の tier 組合せから対応する Descriptor を選ぶ。
    private static DiagnosticDescriptor? SelectDescriptor(Tier current, Tier referenced)
    {
        return (current, referenced) switch
        {
            (Tier.Sdk, Tier.Tier2) => Diagnostics.SdkReferencesTier2,
            (Tier.Sdk, Tier.Tier3) => Diagnostics.SdkReferencesTier3,
            (Tier.Tier2, Tier.Tier3) => Diagnostics.Tier2ReferencesTier3,
            (Tier.Tier1, Tier.Sdk)
                or (Tier.Tier1, Tier.Tier2)
                or (Tier.Tier1, Tier.Tier3) => Diagnostics.Tier1ReferencesUpperLayers,
            _ => null,
        };
    }
}
