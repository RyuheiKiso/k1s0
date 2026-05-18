// k1s0 tier3 CSP + SRI ヘルパー（設計方針 21: CSP + SRI enforcement）
// unsafe-inline / unsafe-eval 禁止（tier3/CLAUDE.md セキュリティポリシー）
// 第三者 CDN リソースに SRI（Subresource Integrity）必須
// Vite plugin helper として使用する（vite.config.ts から import する）

// CSP ディレクティブの型定義
export interface CspDirectives {
  // デフォルトのソース制限
  readonly "default-src": readonly string[];
  // スクリプトのソース制限（unsafe-inline / unsafe-eval 禁止）
  readonly "script-src": readonly string[];
  // スタイルのソース制限
  readonly "style-src": readonly string[];
  // 画像のソース制限
  readonly "img-src": readonly string[];
  // フォントのソース制限
  readonly "font-src": readonly string[];
  // 接続先（fetch / WebSocket / SSE）の制限
  readonly "connect-src": readonly string[];
  // フレームのソース制限
  readonly "frame-src": readonly string[];
  // form action の制限
  readonly "form-action": readonly string[];
  // base URI の制限
  readonly "base-uri": readonly string[];
  // object / embed / applet の制限
  readonly "object-src": readonly string[];
}

// k1s0 tier3 デフォルト CSP ディレクティブ（unsafe-inline / unsafe-eval 完全排除）
export const K1S0_DEFAULT_CSP: CspDirectives = {
  // デフォルト: 同一オリジンのみ
  "default-src": ["'self'"],
  // スクリプト: 同一オリジン + nonce のみ（unsafe-inline / unsafe-eval 禁止）
  "script-src": ["'self'"],
  // スタイル: 同一オリジン + nonce のみ
  "style-src": ["'self'"],
  // 画像: 同一オリジン + data URI（avatar 等）
  "img-src": ["'self'", "data:"],
  // フォント: 同一オリジンのみ
  "font-src": ["'self'"],
  // 接続先: 同一オリジン + BFF API（wss: は WSS のみ許可）
  "connect-src": ["'self'"],
  // フレーム: 自己以外は禁止（clickjacking 防止）
  "frame-src": ["'none'"],
  // form action: 同一オリジンのみ
  "form-action": ["'self'"],
  // base URI: 同一オリジンのみ（base タグの hijacking 防止）
  "base-uri": ["'self'"],
  // object / embed: 完全禁止（Flash / plugin 禁止）
  "object-src": ["'none'"],
};

// CSP ディレクティブから Content-Security-Policy ヘッダー値を生成する
export function buildCspHeader(directives: CspDirectives): string {
  // 各ディレクティブを "; " で区切って結合する
  return Object.entries(directives)
    .map(([directive, sources]) => {
      // ディレクティブ名とソースリストを結合する
      return `${directive} ${(sources as readonly string[]).join(" ")}`;
    })
    .join("; ");
}

// nonce を生成する（CSP nonce-{value} で inline script を許可する場合に使用）
// crypto.getRandomValues を使って 16 バイトのランダムバイト列を base64 エンコードする
export function generateCspNonce(): string {
  // crypto.getRandomValues が利用可能かチェックする
  if (typeof crypto === "undefined" || typeof crypto.getRandomValues === "undefined") {
    // フォールバック: Math.random を使用する（テスト環境用）
    return Math.random().toString(36).slice(2) + Math.random().toString(36).slice(2);
  }
  // 16 バイトのランダムバイト列を生成する
  const bytes = new Uint8Array(16);
  // crypto.getRandomValues でランダムバイト列を埋める
  crypto.getRandomValues(bytes);
  // base64 エンコードして返す
  const binary = Array.from(bytes).map((b) => String.fromCharCode(b)).join("");
  return btoa(binary);
}

// SRI ハッシュの型定義（sha256 / sha384 / sha512 をサポートする）
export type SriHashAlgorithm = "sha256" | "sha384" | "sha512";

// SRI integrity 属性値を構築する
// algorithm: ハッシュアルゴリズム（sha256 / sha384 / sha512）
// hashBase64: base64 エンコード済みのハッシュ値
export function buildSriIntegrity(algorithm: SriHashAlgorithm, hashBase64: string): string {
  // "algorithm-hash" 形式で integrity 属性値を返す
  return `${algorithm}-${hashBase64}`;
}

// 外部リソースの SRI integrity を検証するためのメタデータ型
export interface ExternalResourceMeta {
  // リソースの URL
  readonly url: string;
  // SRI integrity 値（algorithm-hash 形式）
  readonly integrity: string;
  // crossorigin 属性値（SRI には anonymous が必要）
  readonly crossorigin: "anonymous" | "use-credentials";
}

// 外部リソースリスト（SRI 必須の CDN リソースをここで宣言する）
// 実際の hash は本番ビルド時に tools/sri_generator で生成する
export const EXTERNAL_RESOURCES: readonly ExternalResourceMeta[] = [
  // 現時点では外部 CDN リソースなし（tier3/CLAUDE.md: 第三者 CDN リソースに SRI 必須）
];

// Vite plugin 用の CSP / SRI meta タグ HTML を生成する
// vite.config.ts の transformIndexHtml で使用する
export function generateCspMetaTag(directives: CspDirectives): string {
  // CSP ヘッダー値を生成する
  const cspValue = buildCspHeader(directives);
  // meta http-equiv タグを生成する（HTTP ヘッダーの代替）
  return `<meta http-equiv="Content-Security-Policy" content="${cspValue}">`;
}

// k1s0 tier3 デフォルト CSP の meta タグを生成する
export function generateDefaultCspMetaTag(): string {
  // デフォルト CSP ディレクティブから meta タグを生成する
  return generateCspMetaTag(K1S0_DEFAULT_CSP);
}
