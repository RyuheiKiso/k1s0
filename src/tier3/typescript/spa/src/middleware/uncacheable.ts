// k1s0 tier3 uncacheable middleware
// AuthorizationDenied レスポンスの non-cacheable enforcement を実装する
// Cache-Control: no-store, no-cache ヘッダーを設定して認可拒否レスポンスがキャッシュされないことを保証する
// tier3/CLAUDE.md: 認可結果（AuthorizationDenied）の client-side cache 禁止

// AuthorizationDenied HTTP ステータスコード（403 Forbidden）
const HTTP_FORBIDDEN = 403;
// Cache-Control ヘッダー名
const CACHE_CONTROL_HEADER = "Cache-Control";
// Pragma ヘッダー名（HTTP/1.0 互換性）
const PRAGMA_HEADER = "Pragma";
// Expires ヘッダー名（HTTP/1.0 互換性）
const EXPIRES_HEADER = "Expires";
// non-cacheable Cache-Control 値（no-store が最強）
const NO_CACHE_DIRECTIVE = "no-store, no-cache, must-revalidate, proxy-revalidate";
// Pragma の no-cache 値（HTTP/1.0 互換）
const PRAGMA_NO_CACHE = "no-cache";
// Expires の過去日時値（即時期限切れを表す）
const EXPIRES_ZERO = "0";

// Authorization denied レスポンスかどうかを判定する
export function isAuthorizationDenied(response: Response): boolean {
  // HTTP 403 の場合は認可拒否とみなす
  return response.status === HTTP_FORBIDDEN;
}

// non-cacheable ヘッダーを持つ新しい Response を生成する
// 元のレスポンスを変更せず、ヘッダーを追加したコピーを返す
export function withNoCacheHeaders(response: Response): Response {
  // 元のレスポンスのヘッダーをコピーする
  const newHeaders = new Headers(response.headers);
  // Cache-Control: no-store, no-cache を設定する
  newHeaders.set(CACHE_CONTROL_HEADER, NO_CACHE_DIRECTIVE);
  // Pragma: no-cache を設定する（HTTP/1.0 互換性）
  newHeaders.set(PRAGMA_HEADER, PRAGMA_NO_CACHE);
  // Expires: 0 を設定する（即時期限切れ、HTTP/1.0 互換）
  newHeaders.set(EXPIRES_HEADER, EXPIRES_ZERO);
  // 新しいヘッダーを持つ Response を返す（body はストリームのため同一参照）
  return new Response(response.body, {
    // ステータスコードを維持する
    status: response.status,
    // ステータステキストを維持する
    statusText: response.statusText,
    // non-cacheable ヘッダーを設定する
    headers: newHeaders,
  });
}

// fetch middleware: AuthorizationDenied の場合に non-cacheable ヘッダーを強制付与する
// fetch の wrapper として使用する（Service Worker / interceptor から呼ぶ）
export async function uncacheableMiddleware(
  request: Request,
  next: (req: Request) => Promise<Response>,
): Promise<Response> {
  // 元の fetch を実行する
  const response = await next(request);
  // AuthorizationDenied の場合は non-cacheable ヘッダーを付与する
  if (isAuthorizationDenied(response)) {
    // non-cacheable ヘッダーを強制付与したレスポンスを返す
    return withNoCacheHeaders(response);
  }
  // その他のレスポンスはそのまま返す
  return response;
}

// fetch API のラッパー（uncacheable middleware を適用する）
// 通常の fetch の代わりにこの関数を使用することで AuthorizationDenied の cache を防ぐ
export async function fetchWithUncacheable(
  input: string | URL | Request,
  init?: RequestInit,
): Promise<Response> {
  // 入力を Request オブジェクトに変換する
  const request = new Request(input, init);
  // middleware を適用して fetch を実行する
  return uncacheableMiddleware(request, (req) => fetch(req));
}
