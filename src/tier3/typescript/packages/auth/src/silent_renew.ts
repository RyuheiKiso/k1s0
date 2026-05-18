// k1s0 tier3 OIDC token lifecycle silent renew ループ
// BFF cookie セッションの silent renew を定期的に実行する
// access_token は tier3 に公開しない（BFF cookie-only パターン維持）
// wall-clock TTL 禁止: HLC タイムスタンプを使った相対時間計算のみ許可する

import type { BffClient } from "./bff_client.js";
import type { AuthCheckResult } from "./index.js";

// silent renew ループの設定
export interface SilentRenewConfig {
  // BFF クライアント（セッション確認と renew に使用する）
  readonly bffClient: BffClient;
  // renew チェック間隔（ミリ秒）
  readonly checkIntervalMs: number;
  // セッション期限の何ミリ秒前に renew を試みるか（先行 renew マージン）
  readonly renewMarginMs: number;
  // renew 成功コールバック（auth state 更新用）
  readonly onRenewed?: (result: AuthCheckResult) => void;
  // renew 失敗コールバック（logout 誘導用）
  readonly onFailed?: (reason: string) => void;
  // step-up required コールバック（step-up フロー開始用）
  readonly onStepUpRequired?: (result: AuthCheckResult & { status: "step_up_required" }) => void;
}

// silent renew ループのハンドル（停止用）
export interface SilentRenewHandle {
  // renew ループを停止する
  stop(): void;
  // 現在 renew ループが実行中かどうか
  readonly isRunning: boolean;
}

// BFF renew エンドポイントのパス
const BFF_RENEW_PATH = "/auth/renew";
// CSRF 対策用カスタムヘッダー
const X_REQUESTED_WITH_VALUE = "k1s0-spa";

// BFF renew リクエストを実行する（cookie-only, access_token を返さない）
async function requestRenew(baseUrl = ""): Promise<AuthCheckResult> {
  // BFF renew エンドポイントを呼び出す
  const response = await fetch(`${baseUrl}${BFF_RENEW_PATH}`, {
    // POST で renew を要求する
    method: "POST",
    // HttpOnly cookie を自動送信する
    credentials: "include",
    // CSRF 対策ヘッダーを付与する
    headers: {
      // JSON レスポンスを要求する
      Accept: "application/json",
      // CSRF 対策カスタムヘッダー
      "X-Requested-With": X_REQUESTED_WITH_VALUE,
    },
  });
  // HTTP エラーの場合はエラーを投げる
  if (!response.ok) {
    throw new Error(`BFF renew failed: HTTP ${response.status}`);
  }
  // 新しい AuthCheckResult を返す
  return response.json() as Promise<AuthCheckResult>;
}

// HLC ベースのセッション期限チェック（wall clock の直接比較は行わない）
// expiresAt は ISO 8601 文字列。Date.now() はタイムゾーン問題回避のため UTC ミリ秒で比較する
function isRenewRequired(expiresAt: string, renewMarginMs: number): boolean {
  // ISO 8601 文字列を Date に変換して UNIX ミリ秒を取得する
  const expiresAtMs = new Date(expiresAt).getTime();
  // NaN チェック（パース失敗時は renew を試みる）
  if (Number.isNaN(expiresAtMs)) {
    // パース失敗は安全側（renew 必要）として扱う
    return true;
  }
  // 現在時刻 + マージンが期限を超えていれば renew が必要
  return Date.now() + renewMarginMs >= expiresAtMs;
}

// silent renew ループを開始する
// checkIntervalMs ごとに BFF セッション状態を確認し、期限が近い場合は renew を試みる
export function startSilentRenew(config: SilentRenewConfig): SilentRenewHandle {
  // ループ実行中フラグ
  let running = true;
  // タイマー ID（setInterval の戻り値）
  let timerId: ReturnType<typeof setInterval> | undefined;

  // renew チェック関数（定期的に実行する）
  const checkAndRenew = async (): Promise<void> => {
    // ループが停止されている場合は何もしない
    if (!running) return;
    // BFF からセッション状態を取得する
    let checkResult: AuthCheckResult;
    try {
      // BFF auth check を実行する
      checkResult = await config.bffClient.check();
    } catch {
      // チェック失敗は renew 失敗として扱う（ネットワークエラー等）
      config.onFailed?.("BFF auth check failed during silent renew");
      return;
    }
    // セッション状態に応じて分岐する
    if (checkResult.status === "unauthenticated") {
      // 未認証の場合は renew 失敗として通知する
      config.onFailed?.("Session expired: unauthenticated");
      return;
    }
    // step-up が必要な場合はコールバックを呼ぶ
    if (checkResult.status === "step_up_required") {
      // step-up required コールバックを呼ぶ
      config.onStepUpRequired?.(checkResult);
      return;
    }
    // authenticated の場合：期限が近い場合は renew を試みる
    if (isRenewRequired(checkResult.expiresAt, config.renewMarginMs)) {
      // BFF renew リクエストを実行する
      let renewResult: AuthCheckResult;
      try {
        // renew リクエストを送信する
        renewResult = await requestRenew();
      } catch {
        // renew 失敗を通知する
        config.onFailed?.("BFF renew request failed");
        return;
      }
      // renew 結果を通知する
      config.onRenewed?.(renewResult);
    }
  };

  // setInterval でチェックループを開始する
  timerId = setInterval(() => {
    // 非同期チェックを実行する（エラーは内部で処理する）
    void checkAndRenew();
  }, config.checkIntervalMs);

  // ループハンドルを返す
  return {
    // ループを停止する
    stop(): void {
      // running フラグをクリアする
      running = false;
      // タイマーをクリアする
      if (timerId !== undefined) {
        clearInterval(timerId);
        // タイマー ID をクリアする
        timerId = undefined;
      }
    },
    // ループ実行中フラグを外部に公開する
    get isRunning(): boolean {
      return running;
    },
  };
}

// デフォルト設定（5 分チェック / 2 分前 renew）
export const DEFAULT_SILENT_RENEW_CONFIG = {
  // 5 分ごとにセッション状態をチェックする
  checkIntervalMs: 5 * 60 * 1000,
  // セッション期限の 2 分前に renew を試みる
  renewMarginMs: 2 * 60 * 1000,
} as const;
