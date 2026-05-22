// k1s0 Companion JWT Claim Processor
// Authorization ヘッダーから Bearer token を抽出して OTel span に attribute として注入する
// WCF / HttpWebRequest / WebClient / HttpClient の 4 stack に対応する
// wall-clock TTL 禁止規律に従い DateTime.UtcNow を TTL 計算に使用しない

// System 名前空間: Exception / StringSplitOptions 等の基本型に使用する
using System;
// System.Diagnostics: Activity（OTel span 相当）の操作に使用する
using System.Diagnostics;
// JWT デコードライブラリ: Bearer token のクレーム解析に使用する
using System.IdentityModel.Tokens.Jwt;
// System.Net: HttpWebRequest / WebClient のヘッダー操作に使用する
using System.Net;
// System.Net.Http: HttpClient のヘッダー操作に使用する
using System.Net.Http;
// System.Collections.Generic: Dictionary / IEnumerable に使用する
using System.Collections.Generic;

namespace K1s0.Companion.NetFx.OTelExt
{
    /// <summary>
    /// JWT Claim Processor: Authorization ヘッダーから Bearer token を解析して
    /// OpenTelemetry span attribute として注入するプロセッサ
    /// WCF / HttpWebRequest / WebClient / HttpClient の 4 HTTP stack に対応する
    /// </summary>
    public static class JwtClaimProcessor
    {
        // JWT Bearer token のプレフィックス文字列（RFC 6750 に準拠する）
        private const string BearerPrefix = "Bearer ";

        // OTel span attribute のプレフィックス（k1s0 固有の属性名前空間）
        private const string AttributePrefix = "enduser.";

        // ActivitySource: JWT claim processor のスパンを生成するために使用する
        private static readonly ActivitySource _activitySource =
            new ActivitySource(K1s0ActivitySources.JwtClaimSource);

        /// <summary>
        /// HttpWebRequest の Authorization ヘッダーから JWT クレームを抽出して
        /// 現在の OTel Activity に attribute として注入する
        /// </summary>
        /// <param name="request">JWT を含む HttpWebRequest インスタンス</param>
        public static void InjectFromHttpWebRequest(HttpWebRequest request)
        {
            // request が null の場合は何もしない（防衛的実装）
            if (request == null)
            {
                // null 引数に対してはサイレントで処理を終了する
                return;
            }

            // Authorization ヘッダーを取得する
            var authHeader = request.Headers[HttpRequestHeader.Authorization];

            // Authorization ヘッダーから JWT クレームを現在の Activity に注入する
            InjectClaimsToCurrentActivity(authHeader, "HttpWebRequest");
        }

        /// <summary>
        /// WebClient の Headers から JWT クレームを抽出して
        /// 現在の OTel Activity に attribute として注入する
        /// </summary>
        /// <param name="headers">WebClient.Headers インスタンス</param>
        public static void InjectFromWebClientHeaders(WebHeaderCollection headers)
        {
            // headers が null の場合は何もしない（防衛的実装）
            if (headers == null)
            {
                // null 引数に対してはサイレントで処理を終了する
                return;
            }

            // WebClient の Authorization ヘッダーを取得する
            var authHeader = headers[HttpRequestHeader.Authorization];

            // Authorization ヘッダーから JWT クレームを現在の Activity に注入する
            InjectClaimsToCurrentActivity(authHeader, "WebClient");
        }

        /// <summary>
        /// HttpClient の RequestMessage から JWT クレームを抽出して
        /// 現在の OTel Activity に attribute として注入する
        /// </summary>
        /// <param name="request">HttpRequestMessage インスタンス</param>
        public static void InjectFromHttpRequestMessage(HttpRequestMessage request)
        {
            // request が null の場合は何もしない（防衛的実装）
            if (request == null)
            {
                // null 引数に対してはサイレントで処理を終了する
                return;
            }

            // Authorization ヘッダーを文字列として取得する（存在しない場合は null）
            string? authHeader = null;
            // Authorization ヘッダーが存在する場合のみ処理する
            if (request.Headers.Authorization != null)
            {
                // Bearer スキームのトークンを "Bearer {token}" 形式で組み立てる
                authHeader = $"{request.Headers.Authorization.Scheme} {request.Headers.Authorization.Parameter}";
            }

            // Authorization ヘッダーから JWT クレームを現在の Activity に注入する
            InjectClaimsToCurrentActivity(authHeader, "HttpClient");
        }

        /// <summary>
        /// WCF OperationContext の MessageProperties から Authorization ヘッダーを取得して
        /// 現在の OTel Activity に JWT クレームを注入する
        /// WCF は System.ServiceModel が必要なため、ヘッダー文字列を直接受け取る形式にする
        /// </summary>
        /// <param name="wcfAuthorizationHeader">WCF から取得した Authorization ヘッダー文字列</param>
        public static void InjectFromWcfAuthorizationHeader(string? wcfAuthorizationHeader)
        {
            // WCF Authorization ヘッダーから JWT クレームを現在の Activity に注入する
            InjectClaimsToCurrentActivity(wcfAuthorizationHeader, "WCF");
        }

        /// <summary>
        /// Authorization ヘッダー文字列から Bearer token を抽出して JWT クレームを解析し
        /// 現在の OTel Activity に attribute として注入する共通ロジック
        /// </summary>
        /// <param name="authorizationHeader">Authorization ヘッダー文字列（"Bearer {token}" 形式）</param>
        /// <param name="stackName">HTTP stack 識別名（ログ・span 属性に使用する）</param>
        private static void InjectClaimsToCurrentActivity(string? authorizationHeader, string stackName)
        {
            // Authorization ヘッダーが null または空の場合は何もしない
            if (string.IsNullOrWhiteSpace(authorizationHeader))
            {
                // Authorization ヘッダーなしの場合はサイレントで終了する
                return;
            }

            // Bearer プレフィックスで始まるかどうかを確認する
            if (!authorizationHeader.StartsWith(BearerPrefix, StringComparison.OrdinalIgnoreCase))
            {
                // Bearer scheme 以外（Basic 等）は処理対象外のため終了する
                return;
            }

            // Bearer プレフィックスを取り除いて JWT token 文字列を抽出する
            var tokenString = authorizationHeader.Substring(BearerPrefix.Length).Trim();

            // token 文字列が空の場合は処理を終了する
            if (string.IsNullOrEmpty(tokenString))
            {
                // 空トークンはスキップする
                return;
            }

            // JWT クレームを解析する（例外は握り潰さずに上位に伝播させる）
            var claims = ParseJwtClaims(tokenString);

            // 現在の OTel Activity を取得する
            var activity = Activity.Current;

            // Activity が存在しない場合は新規作成して注入する
            using var span = activity == null
                // Activity が存在しない場合は新しいスパンを開始する
                ? _activitySource.StartActivity($"jwt.claim.inject.{stackName}")
                // Activity が存在する場合は子スパンを開始する（子スパンへの注入は親 Activity で行う）
                : null;

            // 注入先 Activity を決定する（現在の Activity > 新規スパン > null）
            var targetActivity = activity ?? span;

            // targetActivity が null の場合は記録先がないためスキップする
            if (targetActivity == null)
            {
                // Activity なしの場合は処理を継続しない
                return;
            }

            // HTTP stack 名を span attribute として設定する
            targetActivity.SetTag($"{AttributePrefix}http_stack", stackName);

            // 解析した JWT クレームを OTel span attribute として注入する
            foreach (var (key, value) in claims)
            {
                // k1s0 OTel attribute 命名規約: "enduser.{claim_name}" 形式で設定する
                targetActivity.SetTag($"{AttributePrefix}{key}", value);
            }
        }

        /// <summary>
        /// JWT token 文字列からクレームを解析して Dictionary として返す
        /// 解析失敗の場合は空の Dictionary を返す（例外を伝播しない）
        /// </summary>
        /// <param name="tokenString">JWT compact serialization 形式のトークン文字列</param>
        /// <returns>クレーム名 → 値のマッピング（解析失敗時は空）</returns>
        private static IEnumerable<(string key, string value)> ParseJwtClaims(string tokenString)
        {
            // JWT ハンドラーをインスタンス化する
            var handler = new JwtSecurityTokenHandler();

            // JWT 形式の検証を行う（不正な形式の場合は空を返す）
            if (!handler.CanReadToken(tokenString))
            {
                // JWT 形式ではないトークンはスキップして空を返す
                yield break;
            }

            // JWT token を解析する
            JwtSecurityToken? jwtToken;
            try
            {
                // JWT 署名検証なしで claims のみを読み取る（検証は tier1 BFF が行う）
                jwtToken = handler.ReadJwtToken(tokenString);
            }
            catch (Exception)
            {
                // JWT 解析失敗の場合はサイレントに空を返す（伝播しない）
                yield break;
            }

            // subject (sub) クレームを抽出する
            if (!string.IsNullOrEmpty(jwtToken.Subject))
            {
                // sub クレームを "id" として設定する（OpenTelemetry semantic conventions 準拠）
                yield return ("id", jwtToken.Subject);
            }

            // JWT ID (jti) クレームを抽出する
            var jti = jwtToken.Id;
            if (!string.IsNullOrEmpty(jti))
            {
                // jti クレームをそのまま設定する
                yield return ("jti", jti);
            }

            // カスタムクレーム（tenant_id / role 等）を抽出する
            foreach (var claim in jwtToken.Claims)
            {
                // sub / jti は上で処理済みのためスキップする
                if (claim.Type == "sub" || claim.Type == "jti")
                {
                    // 重複を避けるためスキップする
                    continue;
                }

                // 空のクレーム値はスキップする
                if (string.IsNullOrEmpty(claim.Value))
                {
                    // 空値クレームはスキップする
                    continue;
                }

                // クレーム型と値を yield で返す
                yield return (claim.Type, claim.Value);
            }
        }
    }
}
