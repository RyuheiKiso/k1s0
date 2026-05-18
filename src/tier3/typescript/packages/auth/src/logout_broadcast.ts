// k1s0 tier3 BroadcastChannel を使った logout イベント配信
// OIDC Back-Channel Logout 受信後、同一オリジンの全タブ / ウィンドウに logout を通知する
// PII を含まない sessionHash のみを送信する（個人情報は含まない設計）
// BroadcastChannel は同一オリジン内のみ有効（cross-origin には送信しない）

import type { LogoutBroadcastEvent } from "./index.js";

// BroadcastChannel のチャンネル名（k1s0 tier3 logout 専用）
const LOGOUT_BROADCAST_CHANNEL = "k1s0_tier3_logout";

// logout ブロードキャスト送信関数
// sessionHash: ログアウト対象セッションの hash（PII を含まない）
export function broadcastLogout(sessionHash: string): void {
  // BroadcastChannel が利用可能かチェックする
  if (typeof BroadcastChannel === "undefined") {
    // BroadcastChannel が利用できない環境では何もしない（SSR / 古いブラウザ対策）
    return;
  }
  // BroadcastChannel を生成する（送信後すぐ close する）
  const channel = new BroadcastChannel(LOGOUT_BROADCAST_CHANNEL);
  // logout イベントを構築する（PII を含まない）
  const event: LogoutBroadcastEvent = {
    // イベント種別を設定する
    type: "logout_broadcast",
    // セッション hash を設定する（個人情報を含まない）
    sessionHash,
  };
  // 同一オリジンの全タブ / ウィンドウに送信する
  channel.postMessage(event);
  // 送信後すぐにチャンネルを閉じる（メモリリーク防止）
  channel.close();
}

// logout ブロードキャスト受信ハンドル（受信後の停止用）
export interface LogoutBroadcastHandle {
  // 受信リスナーを停止する
  stop(): void;
  // 現在リッスン中かどうか
  readonly isListening: boolean;
}

// logout ブロードキャストを受信するリスナーを登録する
// onLogout: logout イベント受信時に呼ぶコールバック（全タブのログアウト処理を実行する）
export function listenLogoutBroadcast(
  onLogout: (event: LogoutBroadcastEvent) => void,
): LogoutBroadcastHandle {
  // BroadcastChannel が利用可能かチェックする
  if (typeof BroadcastChannel === "undefined") {
    // BroadcastChannel が利用できない環境ではダミーハンドルを返す
    return {
      stop(): void { /* BroadcastChannel 非対応環境では何もしない */ },
      isListening: false,
    };
  }
  // BroadcastChannel を生成する（リッスン用）
  const channel = new BroadcastChannel(LOGOUT_BROADCAST_CHANNEL);
  // リッスン中フラグ
  let listening = true;

  // message イベントリスナーを登録する
  channel.addEventListener("message", (messageEvent: MessageEvent<unknown>) => {
    // リッスンが停止されている場合は無視する
    if (!listening) return;
    // message data の型チェックを行う
    const data = messageEvent.data;
    // LogoutBroadcastEvent の型ガード（type フィールドで判定する）
    if (
      data !== null &&
      typeof data === "object" &&
      "type" in data &&
      (data as { type: unknown }).type === "logout_broadcast" &&
      "sessionHash" in data &&
      typeof (data as { sessionHash: unknown }).sessionHash === "string"
    ) {
      // 型チェック済みのイベントとしてコールバックを呼ぶ
      onLogout(data as LogoutBroadcastEvent);
    }
  });

  // ハンドルを返す（stop() でリスナーを停止できる）
  return {
    // リスナーを停止する
    stop(): void {
      // listening フラグをクリアする
      listening = false;
      // BroadcastChannel を閉じる（メモリリーク防止）
      channel.close();
    },
    // リッスン中フラグを外部に公開する
    get isListening(): boolean {
      return listening;
    },
  };
}

// 全タブ logout フロー（BFF logout + 全タブ通知）
// bffLogout: BFF ログアウト関数（BffClient.logout() を渡す）
// sessionHash: ログアウト対象セッション hash（PII を含まない）
export async function performGlobalLogout(
  bffLogout: () => Promise<void>,
  sessionHash: string,
): Promise<void> {
  // BFF にログアウトを要求する（BFF 側で Keycloak Back-Channel Logout を実施する）
  await bffLogout();
  // logout ブロードキャストを同一オリジンの全タブに送信する
  broadcastLogout(sessionHash);
}
