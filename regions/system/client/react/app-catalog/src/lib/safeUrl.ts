// M-8 対応: 画像URLのスキームを検証し、http/https 以外のスキームを拒否するユーティリティ
// javascript: や data: スキームによる XSS 攻撃を防ぐため、
// validation ライブラリの validateURL を使って安全なURLのみを許可する
import { validateURL } from '../../../../library/typescript/validation/src/index';

/**
 * 画像URLのスキームを検証し、安全なURLのみを返す
 * 無効またはスキームが http/https 以外の場合は undefined を返す
 */
export function safeImageUrl(url: string | undefined | null): string | undefined {
  if (!url) return undefined;
  try {
    validateURL(url);
    return url;
  } catch {
    return undefined;
  }
}
