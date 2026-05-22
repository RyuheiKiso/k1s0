// k1s0-impl: IMPL-tier1-0010 realizes=FR-tier1-010
// K1s0Analyzer.cs — k1s0 tier1 Roslyn DiagnosticAnalyzer 骨格実装
// tier1/CLAUDE.md §lint ツール「K1s0Analyzer」の実体。
// 禁止 API 使用・facade 経由強制・依存方向違反を Roslyn 静的解析で検出する。
// 診断 ID: K1S0001 (BannedApi), K1S0002 (FacadeRequired), K1S0003 (DependencyDirection)

// Microsoft.CodeAnalysis: DiagnosticAnalyzer / Diagnostic に使用する
using Microsoft.CodeAnalysis;
// Microsoft.CodeAnalysis.CSharp: C# 構文ツリー解析に使用する
using Microsoft.CodeAnalysis.CSharp;
// Microsoft.CodeAnalysis.CSharp.Syntax: InvocationExpressionSyntax 等に使用する
using Microsoft.CodeAnalysis.CSharp.Syntax;
// Microsoft.CodeAnalysis.Diagnostics: DiagnosticAnalyzer / AnalysisContext に使用する
using Microsoft.CodeAnalysis.Diagnostics;
// System.Collections.Immutable: ImmutableArray<DiagnosticDescriptor> に使用する
using System.Collections.Immutable;

// k1s0 tier1 lint analyzer 名前空間
namespace K1s0.Tier1.Lint;

/// <summary>
/// K1s0Analyzer は k1s0 tier1 固有の Roslyn DiagnosticAnalyzer。
/// tier1/CLAUDE.md §公開 API の型制約（禁止 API / facade 経由強制）を静的解析で enforce する。
/// </summary>
// DiagnosticAnalyzer アノテーション: C# 構文・セマンティクス解析を行う
[DiagnosticAnalyzer(LanguageNames.CSharp)]
public sealed class K1s0Analyzer : DiagnosticAnalyzer
{
    // K1S0001: 禁止 API 使用診断 ID
    private const string BannedApiId = "K1S0001";
    // K1S0002: facade 経由強制診断 ID
    private const string FacadeRequiredId = "K1S0002";
    // K1S0003: 依存方向違反診断 ID
    private const string DependencyDirectionId = "K1S0003";

    // BannedApiDescriptor: 禁止 API 使用診断ルール定義
    private static readonly DiagnosticDescriptor BannedApiDescriptor = new(
        // 診断 ID: K1S0001
        id: BannedApiId,
        // タイトル: 禁止 API 使用
        title: "k1s0 banned API usage",
        // メッセージテンプレート: 禁止 API 名を含む
        messageFormat: "Banned API `{0}` used. Use the approved facade or alternative: {1}",
        // カテゴリ: k1s0 tier1
        category: "k1s0.tier1",
        // 重大度: Error（CI fail）
        defaultSeverity: DiagnosticSeverity.Error,
        // デフォルトで有効化する
        isEnabledByDefault: true,
        // 説明: tier1/CLAUDE.md §公開 API の型制約を参照する
        description: "tier1/CLAUDE.md §公開 API の型制約: OSS 型を公開 API シグネチャに露出禁止（facade 経由必須）。"
    );

    // FacadeRequiredDescriptor: facade 経由強制診断ルール定義
    private static readonly DiagnosticDescriptor FacadeRequiredDescriptor = new(
        // 診断 ID: K1S0002
        id: FacadeRequiredId,
        // タイトル: facade 経由強制
        title: "k1s0 facade required for OSS type access",
        // メッセージテンプレート: OSS 型名と facade 型名を含む
        messageFormat: "Direct OSS type `{0}` in public API. Use facade type `{1}` instead.",
        // カテゴリ: k1s0 tier1
        category: "k1s0.tier1",
        // 重大度: Error（CI fail）
        defaultSeverity: DiagnosticSeverity.Error,
        // デフォルトで有効化する
        isEnabledByDefault: true,
        // 説明: tier1/CLAUDE.md §OSS 型を公開 API シグネチャに露出禁止を参照する
        description: "tier1/CLAUDE.md §公開 API の型制約: L3/L2*/L1+ カテゴリの OSS 型を公開 API シグネチャに露出禁止。"
    );

    // DependencyDirectionDescriptor: 依存方向違反診断ルール定義
    private static readonly DiagnosticDescriptor DependencyDirectionDescriptor = new(
        // 診断 ID: K1S0003
        id: DependencyDirectionId,
        // タイトル: 依存方向違反
        title: "k1s0 dependency direction violation",
        // メッセージテンプレート: 違反詳細を含む
        messageFormat: "Dependency direction violation: {0}",
        // カテゴリ: k1s0 tier1
        category: "k1s0.tier1",
        // 重大度: Error（CI fail）
        defaultSeverity: DiagnosticSeverity.Error,
        // デフォルトで有効化する
        isEnabledByDefault: true,
        // 説明: src/CLAUDE.md §依存方向の制約を参照する
        description: "src/CLAUDE.md §依存方向の制約: tier3 → tier1 / OSS 直接 import 禁止（tier2 SDK 経由必須）。"
    );

    // SupportedDiagnostics: この analyzer がサポートする診断ルールのリスト
    public override ImmutableArray<DiagnosticDescriptor> SupportedDiagnostics =>
        // 3 種類の診断ルールを返す
        ImmutableArray.Create(
            BannedApiDescriptor,
            FacadeRequiredDescriptor,
            DependencyDirectionDescriptor
        );

    // Initialize: analyzer の初期化。解析対象の登録を行う。
    public override void Initialize(AnalysisContext context)
    {
        // concurrent 解析を許可する（パフォーマンス向上）
        context.EnableConcurrentExecution();
        // 生成コードの解析設定（生成コードも解析対象にする）
        context.ConfigureGeneratedCodeAnalysis(
            GeneratedCodeAnalysisFlags.Analyze | GeneratedCodeAnalysisFlags.ReportDiagnostics
        );
        // InvocationExpression（メソッド呼び出し）を解析対象として登録する
        context.RegisterSyntaxNodeAction(
            // メソッド呼び出しの解析アクションを登録する
            AnalyzeInvocation,
            // InvocationExpression ノードを対象にする
            SyntaxKind.InvocationExpression
        );
        // UsingDirective（using 文）を解析対象として登録する
        context.RegisterSyntaxNodeAction(
            // using 文の解析アクションを登録する
            AnalyzeUsingDirective,
            // UsingDirective ノードを対象にする
            SyntaxKind.UsingDirective
        );
    }

    // AnalyzeInvocation: メソッド呼び出しを解析して禁止 API / facade 違反を検出する
    private static void AnalyzeInvocation(SyntaxNodeAnalysisContext context)
    {
        // InvocationExpressionSyntax にキャストする
        if (context.Node is not InvocationExpressionSyntax invocation)
        {
            // キャスト失敗時はスキップする
            return;
        }
        // メソッドのシンボル情報を取得する（セマンティクス解析）
        var symbol = context.SemanticModel.GetSymbolInfo(invocation).Symbol;
        // シンボルが取得できない場合はスキップする
        if (symbol is null)
        {
            return;
        }
        // シンボルの完全修飾名を取得する
        var fullName = symbol.ToDisplayString();
        // Npgsql.NpgsqlConnection の直接使用を検出する（facade 経由必須）
        if (fullName.StartsWith("Npgsql.", System.StringComparison.Ordinal))
        {
            // facade 経由強制違反として報告する
            context.ReportDiagnostic(
                Diagnostic.Create(
                    FacadeRequiredDescriptor,
                    // 違反箇所の位置情報を渡す
                    invocation.GetLocation(),
                    // OSS 型名
                    fullName,
                    // facade 型名
                    "K1s0.Tier1.IDbClient"
                )
            );
        }
        // Confluent.Kafka の直接使用を検出する（facade 経由必須）
        if (fullName.StartsWith("Confluent.Kafka.", System.StringComparison.Ordinal))
        {
            // facade 経由強制違反として報告する
            context.ReportDiagnostic(
                Diagnostic.Create(
                    FacadeRequiredDescriptor,
                    // 違反箇所の位置情報を渡す
                    invocation.GetLocation(),
                    // OSS 型名
                    fullName,
                    // facade 型名
                    "K1s0.Tier1.IMessagingProducer / IMessagingConsumer"
                )
            );
        }
    }

    // AnalyzeUsingDirective: using 文を解析して依存方向違反を検出する
    private static void AnalyzeUsingDirective(SyntaxNodeAnalysisContext context)
    {
        // UsingDirectiveSyntax にキャストする
        if (context.Node is not UsingDirectiveSyntax usingDirective)
        {
            // キャスト失敗時はスキップする
            return;
        }
        // using 文の名前空間を文字列として取得する
        var namespaceName = usingDirective.Name?.ToString() ?? string.Empty;
        // 依存方向違反パターン: tier3 から tier1 OSS を直接 import 禁止
        // NOTE: 現時点では using 文のファイルコンテキスト（tier3 かどうか）を
        // ディレクトリ名から判定する（SyntaxTree.FilePath で確認する）
        var filePath = context.Node.SyntaxTree.FilePath;
        // ファイルパスが tier3 配下かどうかを確認する
        var isTier3 = filePath.Contains("/tier3/", System.StringComparison.Ordinal) ||
                      filePath.Contains("\\tier3\\", System.StringComparison.Ordinal);
        // tier3 ファイルで tier1 直接参照を検出する
        if (isTier3 && namespaceName.StartsWith("K1s0.Tier1", System.StringComparison.Ordinal) &&
            !namespaceName.StartsWith("K1s0.Tier1.Interfaces", System.StringComparison.Ordinal))
        {
            // 依存方向違反として報告する
            context.ReportDiagnostic(
                Diagnostic.Create(
                    DependencyDirectionDescriptor,
                    // 違反箇所の位置情報を渡す
                    usingDirective.GetLocation(),
                    // 違反詳細: tier3 → tier1 直接依存禁止（tier2 SDK 経由必須）
                    $"tier3 cannot directly import `{namespaceName}`. Use tier2 SDK facade."
                )
            );
        }
    }
}
