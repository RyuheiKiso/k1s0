// k1s0 tier3 BFF (Backend for Frontend) クライアント
// access_token を tier3 に公開しない設計（BFF cookie-only パターン）
// Keycloak OIDC + BFF auth-edge 経由で cookie セッション状態を確認する
// credentials: 'include' で HttpOnly cookie を自動送信する

import type { AuthCheckResult } from "./index.js";

// BFF auth エンドポイントのパス（BFF 側が Keycloak と通信する）
const BFF_AUTH_CHECK_PATH = "/auth/check";
// BFF ログアウトエンドポイントのパス
const BFF_LOGOUT_PATH = "/auth/logout";
// CSRF 対策用カスタムヘッダー（BFF がオリジン確認に使用する）
const X_REQUESTED_WITH_VALUE = "k1s0-spa";

// fetch が失敗した場合のエラー型
export class BffNetworkError extends Error {
  // HTTP ステータスコード（fetch 成功 = HTTP エラーの場合）
  readonly statusCode: number | null;

  // コンストラクタ（メッセージとオプションの HTTP ステータスを受け取る）
  constructor(message: string, statusCode: number | null = null) {
    // 親クラスのコンストラクタを呼ぶ
    super(message);
    // エラークラス名を設定する
    this.name = "BffNetworkError";
    // HTTP ステータスコードを保持する
    this.statusCode = statusCode;
  }
}

// BFF クライアントクラス（BFF auth-edge との通信を担当する）
export class BffClient {
  // BFF のベース URL（デフォルトは空文字 = 同一オリジン）
  private readonly _baseUrl: string;

  // コンストラクタ（ベース URL を受け取る。省略時は同一オリジン）
  constructor(baseUrl = "") {
    // ベース URL を保持する
    this._baseUrl = baseUrl;
  }

  // BFF にセッション状態を問い合わせる
  // access_token は BFF が保持し、tier3 には一切返さない
  async check(): Promise<AuthCheckResult> {
    // /auth/check を呼び出す（credentials: 'include' で cookie を送信する）
    const response = await this._fetch(BFF_AUTH_CHECK_PATH, {
      // GET メソッドで問い合わせる
      method: "GET",
    });
    // HTTP エラーの場合はエラーを投げる
    if (!response.ok) {
      throw new BffNetworkError(
        `BFF auth check failed: HTTP ${response.status}`,
        response.status,
      );
    }
    // JSON レスポンスを AuthCheckResult として返す
    return response.json() as Promise<AuthCheckResult>;
  }

  // BFF にログアウトを要求する（BFF 側で Keycloak Back-Channel Logout を実施する）
  async logout(): Promise<void> {
    // /auth/logout を呼び出す（POST で CSRF トークンを送る場合は BFF 側で実装）
    const response = await this._fetch(BFF_LOGOUT_PATH, {
      // POST メソッドでログアウトを要求する
      method: "POST",
    });
    // HTTP エラーの場合はエラーを投げる
    if (!response.ok) {
      throw new BffNetworkError(
        `BFF logout failed: HTTP ${response.status}`,
        response.status,
      );
    }
  }

  // 共通 fetch ラッパー（credentials: include + X-Requested-With を自動付与する）
  private async _fetch(path: string, init: RequestInit): Promise<Response> {
    // リクエスト URL を生成する
    const url = `${this._baseUrl}${path}`;
    // fetch を呼び出す
    const response = await fetch(url, {
      // 呼び出し元の init をスプレッドする
      ...init,
      // HttpOnly cookie を自動送信するため credentials を include に設定する
      credentials: "include",
      // リクエストヘッダーに CSRF 対策ヘッダーを付与する
      headers: {
        // JSON レスポンスを要求する
        Accept: "application/json",
        // CSRF 対策: BFF がオリジンを確認するためのカスタムヘッダー
        "X-Requested-With": X_REQUESTED_WITH_VALUE,
        // 呼び出し元が追加ヘッダーを渡した場合はマージする
        ...(init.headers ?? {}),
      },
    });
    // レスポンスを返す（エラー判定は呼び出し元に委ねる）
    return response;
  }
}

// デフォルト BffClient シングルトン（同一オリジン向け）
export const defaultBffClient = new BffClient();
