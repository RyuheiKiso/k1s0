// 本ファイルは tier2 .NET 共通の HTTP エラー応答 JSON 型。
//
// 設計動機:
//   tier3 BFF / Web 側 (`@k1s0/api-client` の ApiError) は
//   `{ error: { code, message, category } }` の旧形式と
//   `{ code, message }` の新形式の両方を受けるが、tier2 .NET は新形式を採用する
//   (Go の shared/errors との対称性、JSON 表面の単純化)。

namespace K1s0.Tier2.Common.Errors;

/// <summary>
/// HTTP エラー応答の JSON body。
/// JSON フィールドは <c>code</c> / <c>message</c> / <c>category</c> (snake_case 不要、すべて lowercase 英 1 単語)。
/// </summary>
public sealed record ErrorBody(
    /// <summary>E-T2-* / E-T1-* 等のコード。</summary>
    string Code,
    /// <summary>人間可読メッセージ (PII を含めない)。</summary>
    string Message,
    /// <summary>カテゴリ wire 文字列 (VALIDATION / NOT_FOUND / 等)。</summary>
    string Category);
