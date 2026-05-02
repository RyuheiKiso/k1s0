// 本ファイルは tier2 .NET 業務エラーの基底例外型。
//
// docs 正典:
//   src/tier2/go/shared/errors/errors.go (Go 側 DomainError と同等の集合を保つ)
//
// 設計動機:
//   .NET 側は throw / catch で扱える Exception 派生として実装する。
//   ASP.NET Core middleware (DomainExceptionMiddleware) が catch して
//   ErrorBody JSON へ変換する。

namespace K1s0.Tier2.Common.Errors;

/// <summary>
/// tier2 業務エラーの基底例外。Code / Category / Message を持ち、HTTP / GraphQL
/// レスポンスへ一貫した形で写像できる。
/// </summary>
public class DomainException : Exception
{
    /// <summary>カテゴリ (HTTP status の根拠)。</summary>
    public Category Category { get; }

    /// <summary>E-T2-* コード (例: "E-T2-RECON-001")。Code を持たない経路では空文字。</summary>
    public string Code { get; }

    /// <summary>
    /// Category + Code 付きで DomainException を組み立てる。
    /// </summary>
    public DomainException(Category category, string code, string message, Exception? innerException = null)
        : base(message, innerException)
    {
        Category = category;
        Code = code;
    }

    /// <summary>
    /// CA1032 対応: 標準 Exception 互換 constructor (デフォルト Category=Internal / Code=空)。
    /// 業務コード経路では <see cref="DomainException(Category,string,string,Exception?)"/> を使うこと。
    /// </summary>
    public DomainException()
        : this(Category.Internal, string.Empty, "DomainException")
    {
    }

    /// <summary>
    /// CA1032 対応: 標準 Exception 互換 constructor (Category=Internal / Code=空)。
    /// </summary>
    public DomainException(string message)
        : this(Category.Internal, string.Empty, message)
    {
    }

    /// <summary>
    /// CA1032 対応: 標準 Exception 互換 constructor (Category=Internal / Code=空、cause 付き)。
    /// </summary>
    public DomainException(string message, Exception innerException)
        : this(Category.Internal, string.Empty, message, innerException)
    {
    }

    /// <summary>
    /// 簡潔な ToString。コード / カテゴリ / メッセージ / inner cause の順に整形する
    /// (Go の DomainError.Error() に対称)。
    /// </summary>
    public override string ToString()
    {
        if (InnerException is not null)
        {
            return $"{Code} [{Category.Wire()}] {Message}: {InnerException.Message}";
        }
        return $"{Code} [{Category.Wire()}] {Message}";
    }

    // 便利 factory (Go の `New(cat, code, msg)` 相当)。

    /// <summary>VALIDATION DomainException を組み立てる。</summary>
    public static DomainException Validation(string code, string message) =>
        new(Category.Validation, code, message);

    /// <summary>NOT_FOUND DomainException を組み立てる。</summary>
    public static DomainException NotFound(string code, string message) =>
        new(Category.NotFound, code, message);

    /// <summary>CONFLICT DomainException を組み立てる。</summary>
    public static DomainException Conflict(string code, string message) =>
        new(Category.Conflict, code, message);

    /// <summary>UPSTREAM DomainException を組み立てる (cause 付き)。</summary>
    public static DomainException Upstream(string code, string message, Exception? cause = null) =>
        new(Category.Upstream, code, message, cause);

    /// <summary>INTERNAL DomainException を組み立てる (cause 付き)。</summary>
    public static DomainException InternalError(string code, string message, Exception? cause = null) =>
        new(Category.Internal, code, message, cause);
}
