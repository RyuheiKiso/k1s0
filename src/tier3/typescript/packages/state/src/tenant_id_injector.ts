// tenant_id_injector.ts — tenant_id の BFF injector
// __Host-tenant cookie からテナント ID を読み取り ClientState に提供する
// セキュリティ制約: クエリパラメータ / URL に tenant_id フィールド禁止（CLAUDE.md §データ保護）
// __Host- prefix cookie はセキュアな HttpOnly + SameSite=Strict を要件とする
// cookie の読み取りは document.cookie 経由（HttpOnly でない場合のみ可能）

// ClientState 型をインポートする（packages/state/src/reducer.ts と整合させる）
import type { ClientState } from './reducer.js';

// tenant cookie 名の定数定義（__Host- prefix で Secure + path=/ を強制する）
const TENANT_COOKIE_NAME = '__Host-tenant' as const;

// cookie 文字列から指定名の cookie 値を解析するヘルパー関数
// document.cookie の全 cookie 文字列をパースして該当値を返す
function parseCookieValue(cookieString: string, name: string): string | null {
  // cookie 文字列をセミコロンで分割して各エントリを処理する
  const entries = cookieString.split(';');
  for (const entry of entries) {
    // 各エントリの先頭の空白を除去する
    const trimmed = entry.trim();
    // 名前と値を = で分割する
    const eqIndex = trimmed.indexOf('=');
    // = が存在しない場合はスキップする
    if (eqIndex === -1) {
      continue;
    }
    // cookie 名を取得して trim する
    const cookieName = trimmed.slice(0, eqIndex).trim();
    // 対象の cookie 名と一致する場合は値を返す
    if (cookieName === name) {
      // cookie 値を取得して URL デコードする
      const rawValue = trimmed.slice(eqIndex + 1).trim();
      // decodeURIComponent で URL エンコードされた値をデコードする
      try {
        return decodeURIComponent(rawValue);
      } catch {
        // デコードに失敗した場合は raw 値をそのまま返す
        return rawValue;
      }
    }
  }
  // 対象の cookie が見つからない場合は null を返す
  return null;
}

// document.cookie から __Host-tenant cookie を読み取る
// 実行環境がブラウザでない場合（SSR / Node.js 環境）は null を返す
function readTenantCookie(): string | null {
  // ブラウザ環境かどうかを確認する（document が存在するか）
  if (typeof document === 'undefined') {
    // SSR / Node.js 環境では cookie を読み取れないため null を返す
    return null;
  }
  // document.cookie から __Host-tenant cookie の値を取得する
  return parseCookieValue(document.cookie, TENANT_COOKIE_NAME);
}

// ClientState から tenant ID を注入する関数
// __Host-tenant cookie を読み取り、存在する場合はその値を返す
// クエリパラメータ / URL / state フィールドに tenant_id を公開しない（CLAUDE.md §データ保護）
export function injectTenantId<T, TPayload>(
  // ClientState を受け取る（型引数で汎用的に対応する）
  state: ClientState<T, TPayload>,
): string | null {
  // state パラメーターは将来の拡張（state から tenant 情報を取得する場合）のために保持する
  void state;
  // __Host-tenant cookie からテナント ID を取得する
  const tenantId = readTenantCookie();
  // テナント ID を返す（null = 未設定またはブラウザ環境でない）
  return tenantId;
}

// テナント ID が設定されていることを表明する関数
// テナント ID が取得できない場合は例外をスローする（起動時チェックに使用する）
export function assertTenantIdSet(): void {
  // __Host-tenant cookie を読み取る
  const tenantId = readTenantCookie();
  // テナント ID が null または空の場合は例外をスローする
  if (tenantId === null || tenantId === '') {
    // テナント ID が設定されていない場合はエラーをスローする
    // エラーメッセージに tenant ID の値を含めない（情報漏洩防止）
    throw new Error(
      '__Host-tenant cookie が設定されていません。' +
      'BFF ログイン処理を経由してテナント ID を取得してください。',
    );
  }
  // テナント ID が設定されている場合は正常終了する（戻り値なし）
}

// テスト用のヘルパー関数（テスト環境でのみ使用する）
// Node.js テスト環境で document.cookie を模倣するために使用する
// 本番コードからは呼び出さない
export function _parseCookieValueForTest(cookieString: string, name: string): string | null {
  // テスト用の cookie パーサーを提供する（本番コードと同一実装）
  return parseCookieValue(cookieString, name);
}
