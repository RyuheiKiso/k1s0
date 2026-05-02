// 本ファイルは tier2 .NET 業務エラーの大分類と HTTP status 写像。
//
// docs 正典:
//   docs/04_概要設計/30_共通機能方式設計/
//   src/tier2/go/shared/errors/errors.go (Go 側と同じ分類を維持する)

namespace K1s0.Tier2.Common.Errors;

/// <summary>
/// tier2 業務エラーの大分類。Go 側の <c>errors.Category</c> と同じ集合を維持する。
/// </summary>
public enum Category
{
    /// <summary>入力バリデーション違反 (HTTP 400 相当)。</summary>
    Validation,
    /// <summary>リソース不存在 (HTTP 404 相当)。</summary>
    NotFound,
    /// <summary>楽観的排他違反 / 重複 (HTTP 409 相当)。</summary>
    Conflict,
    /// <summary>tier1 / 外部依存からのエラー (HTTP 502 相当)。</summary>
    Upstream,
    /// <summary>内部実装エラー (HTTP 500 相当、ログだけ詳細を残す)。</summary>
    Internal,
}

/// <summary>カテゴリ → HTTP status / wire 文字列 の写像を提供する拡張。</summary>
public static class CategoryExtensions
{
    /// <summary>
    /// Go 側の <c>Category.HTTPStatus()</c> と同じ写像を返す。
    /// 不明値は安全側に 500 にフォールバックする。
    /// </summary>
    public static int HttpStatus(this Category c) => c switch
    {
        Category.Validation => 400,
        Category.NotFound => 404,
        Category.Conflict => 409,
        Category.Upstream => 502,
        Category.Internal => 500,
        _ => 500,
    };

    /// <summary>
    /// JSON 応答 / ログで使う wire 文字列 (Go 側と同じく大文字 SNAKE_CASE)。
    /// </summary>
    public static string Wire(this Category c) => c switch
    {
        Category.Validation => "VALIDATION",
        Category.NotFound => "NOT_FOUND",
        Category.Conflict => "CONFLICT",
        Category.Upstream => "UPSTREAM",
        Category.Internal => "INTERNAL",
        _ => "INTERNAL",
    };
}
