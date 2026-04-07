import { useContext } from 'react';
import { AuthContext, type AuthContextValue } from './AuthContext';

/**
 * L-015 監査対応: useAuth のエラーメッセージをカスタマイズするオプション。
 * i18n 対応のため、呼び出し元がエラーメッセージを上書きできるようにする。
 * 未指定時はデフォルトの日本語メッセージを使用する。
 */
export interface UseAuthOptions {
  /**
   * AuthProvider 外で useAuth を呼び出した際のエラーメッセージ。
   * i18n ライブラリのキーを解決した文字列を渡すことを想定している。
   * 例: t('auth.error.outside_provider')
   */
  outsideProviderMessage?: string;
}

/**
 * L-015 / DOCS-MED-005 監査対応: useAuth のデフォルトエラーメッセージ定数。
 * アプリケーション側で i18n 文字列を渡せる場合はオプションで上書きすること。
 * デフォルト値は英語とし、多言語環境でも共通のベースラインを提供する。
 * 日本語表示が必要な場合は outsideProviderMessage オプションで上書きすること。
 */
export const AUTH_ERROR_MESSAGES = {
  OUTSIDE_PROVIDER: 'useAuth must be used within an AuthProvider',
} as const;

/**
 * AuthContext を取得するカスタムフック。
 * AuthProvider の外で呼び出した場合はエラーをスローする。
 * @param options - エラーメッセージをカスタマイズするオプション（i18n 対応用）
 */
export function useAuth(options?: UseAuthOptions): AuthContextValue {
  const context = useContext(AuthContext);
  if (!context) {
    // L-015 監査対応: オプションでエラーメッセージを上書き可能にし、i18n に対応する
    const message = options?.outsideProviderMessage ?? AUTH_ERROR_MESSAGES.OUTSIDE_PROVIDER;
    throw new Error(message);
  }
  return context;
}
