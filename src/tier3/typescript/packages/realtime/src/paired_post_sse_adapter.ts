// k1s0 tier3 paired POST-SSE adapter
// /open, /send, /close の 3-endpoint で構成された paired POST-SSE トランスポートを実装する
// HTTP/2 が通らない企業プロキシ環境（chrome_edge_via_corp_proxy / firefox_safari ua_subclass）向け
// wall-clock TTL 禁止規律に従い Date.now() を TTL 計算に使用しない

import type { ResumeToken } from "./resume_token.js";

// SSE メッセージを表す型（サーバーから受信する各イベント）
export interface SseMessage {
  // イベント ID（EventSource の lastEventId に対応する）
  readonly id: string;
  // イベントタイプ（省略時は "message"）
  readonly eventType: string;
  // イベントデータ（JSON 文字列）
  readonly data: string;
}

// paired POST-SSE セッションを表す型
export interface PairedPostSseSession {
  // サーバーが発行したセッション ID（/open レスポンスから取得する）
  readonly sessionId: string;
  // SSE エンドポイントへの EventSource インスタンス（読み取り専用）
  readonly eventSource: EventSource;
  // セッションのクローズ関数
  readonly close: () => Promise<void>;
}

// paired POST-SSE adapter の設定型
export interface PairedPostSseConfig {
  // BFF の paired POST-SSE エンドポイントのベース URL
  // 例: "https://api.k1s0.io/v1/realtime/paired"
  readonly baseUrl: string;
  // /send リクエストのタイムアウト（ミリ秒）
  // wall-clock TTL 禁止規律に従い HLC ベースのタイムアウトを上位レイヤで管理する
  readonly sendTimeoutMs?: number;
  // /open リクエストの追加ヘッダー（認証トークン等）
  readonly headers?: Readonly<Record<string, string>>;
}

// SSE イベントハンドラーの型
export type SseMessageHandler = (message: SseMessage) => void;
// SSE エラーハンドラーの型
export type SseErrorHandler = (error: Event) => void;

/// openPairedPostSse は paired POST-SSE セッションを開始する
/// /open エンドポイントに POST してセッション ID を取得し、SSE ストリームを開く
/// resumeToken が指定された場合はリジューム接続を試みる
export async function openPairedPostSse(
  // adapter 設定
  config: PairedPostSseConfig,
  // SSE メッセージの受信ハンドラー
  onMessage: SseMessageHandler,
  // SSE エラーハンドラー（省略時はコンソールエラー出力）
  onError?: SseErrorHandler,
  // リジュームトークン（再接続時に指定する、省略時は新規セッション）
  resumeToken?: ResumeToken,
): Promise<PairedPostSseSession> {
  // /open エンドポイント URL を組み立てる
  const openUrl = `${config.baseUrl}/open`;

  // /open リクエストのボディを構築する（resume_token がある場合は含める）
  const openBody: Record<string, unknown> = {};
  // resume_token が指定されている場合はボディに含める（リジューム接続）
  if (resumeToken != null) {
    // resume_token を設定する（サーバーは指定の cursor から SSE を再開する）
    openBody["resume_token"] = resumeToken;
  }

  // /open リクエストの追加ヘッダーを構築する
  const openHeaders: Record<string, string> = {
    // Content-Type: JSON ボディを送信するため設定する
    "Content-Type": "application/json",
    // Accept: JSON レスポンスを要求する
    Accept: "application/json",
    // カスタムヘッダーを展開する（認証トークン等）
    ...(config.headers ?? {}),
  };

  // /open エンドポイントに POST してセッション ID を取得する
  const openResponse = await fetch(openUrl, {
    // POST メソッドを使用する
    method: "POST",
    // ヘッダーを設定する
    headers: openHeaders,
    // ボディを JSON 文字列にシリアライズして送信する
    body: JSON.stringify(openBody),
  });

  // /open レスポンスが 200 以外の場合はエラーを投げる
  if (!openResponse.ok) {
    // HTTP エラーレスポンスをエラーとして伝播する
    throw new Error(
      `paired POST-SSE /open 失敗: HTTP ${openResponse.status} ${openResponse.statusText} url=${openUrl}`,
    );
  }

  // /open レスポンスから JSON を解析してセッション ID を取得する
  const openData = (await openResponse.json()) as {
    // サーバーが発行したセッション ID
    session_id: string;
    // SSE ストリーム URL（省略時は baseUrl/open で開く）
    sse_url?: string;
  };

  // セッション ID が存在しない場合はエラーを投げる
  if (typeof openData.session_id !== "string" || openData.session_id.length === 0) {
    // セッション ID なしはサーバーの不正レスポンス
    throw new Error(`paired POST-SSE /open レスポンスに session_id がありません: ${JSON.stringify(openData)}`);
  }

  // SSE エンドポイント URL を決定する（サーバーが指定した URL > デフォルト URL）
  const sseUrl = openData.sse_url
    ?? `${config.baseUrl}/open?session_id=${encodeURIComponent(openData.session_id)}`;

  // EventSource を使って SSE ストリームを開く
  // EventSource は自動再接続機能を持つが、paired POST-SSE では /close で明示的に切断する
  const eventSource = new EventSource(sseUrl);

  // "message" イベントハンドラーを設定する（デフォルトイベントタイプ）
  eventSource.onmessage = (event: MessageEvent) => {
    // SseMessage に変換して onMessage ハンドラーを呼び出す
    const message: SseMessage = {
      // イベント ID を設定する（存在しない場合は空文字列）
      id: event.lastEventId ?? "",
      // デフォルトイベントタイプ
      eventType: "message",
      // イベントデータを設定する
      data: typeof event.data === "string" ? event.data : JSON.stringify(event.data),
    };
    // メッセージハンドラーを呼び出す
    onMessage(message);
  };

  // EventSource エラーハンドラーを設定する
  eventSource.onerror = (error: Event) => {
    // カスタムエラーハンドラーが指定されている場合は呼び出す
    if (onError != null) {
      // エラーハンドラーを呼び出す
      onError(error);
    } else {
      // デフォルトエラー処理: コンソールエラーを出力する
      console.error("paired POST-SSE EventSource エラー:", error);
    }
  };

  // セッションのクローズ関数を定義する（/close エンドポイントを呼び出す）
  const close = async (): Promise<void> => {
    // EventSource を先に閉じる（自動再接続を停止する）
    eventSource.close();

    // /close エンドポイントに POST してサーバー側のセッションを終了する
    const closeUrl = `${config.baseUrl}/close`;

    // /close リクエストを送信する（fire and forget でも可だが結果を確認する）
    const closeResponse = await fetch(closeUrl, {
      // POST メソッドを使用する
      method: "POST",
      // ヘッダーを設定する
      headers: {
        // Content-Type: JSON ボディを送信するため設定する
        "Content-Type": "application/json",
        // カスタムヘッダーを展開する
        ...(config.headers ?? {}),
      },
      // セッション ID をボディに含める
      body: JSON.stringify({ session_id: openData.session_id }),
    });

    // /close レスポンスが 200/204 以外の場合はコンソール警告を出力する（致命的ではない）
    if (!closeResponse.ok) {
      // /close 失敗はコンソール警告として記録する（セッションは既にクライアント側で閉じている）
      console.warn(
        `paired POST-SSE /close 警告: HTTP ${closeResponse.status} session_id=${openData.session_id}`,
      );
    }
  };

  // PairedPostSseSession を返す
  return {
    // セッション ID を設定する
    sessionId: openData.session_id,
    // EventSource インスタンスを設定する
    eventSource,
    // クローズ関数を設定する
    close,
  };
}

/// sendPairedPostSse は paired POST-SSE セッションにメッセージを送信する
/// /send エンドポイントに POST してサーバーにメッセージを届ける
/// response は SSE ストリームで非同期に受信する（/send レスポンス自体は空）
export async function sendPairedPostSse(
  // adapter 設定
  config: PairedPostSseConfig,
  // 送信先セッション ID
  sessionId: string,
  // 送信するメッセージデータ（任意の JSON シリアライズ可能な値）
  data: unknown,
): Promise<void> {
  // /send エンドポイント URL を組み立てる
  const sendUrl = `${config.baseUrl}/send`;

  // /send リクエストのボディを構築する
  const sendBody = {
    // セッション ID を設定する
    session_id: sessionId,
    // 送信データを設定する
    data,
  };

  // タイムアウト設定（AbortController を使用する）
  // wall-clock TTL 禁止規律に従い Date.now() を使わず HLC ベースのタイムアウトを想定する
  // ここでは HTTP レベルのタイムアウトのみ設定する（HLC は上位レイヤで管理する）
  const sendTimeoutMs = config.sendTimeoutMs ?? 5000;
  // AbortController でタイムアウトを実装する
  const controller = new AbortController();
  // タイムアウトタイマーを設定する（setTimeout は wall-clock だが HTTP レベルのタイムアウトのため許容する）
  const timeoutId = setTimeout(() => controller.abort(), sendTimeoutMs);

  try {
    // /send エンドポイントに POST する
    const sendResponse = await fetch(sendUrl, {
      // POST メソッドを使用する
      method: "POST",
      // ヘッダーを設定する
      headers: {
        // Content-Type: JSON ボディを送信するため設定する
        "Content-Type": "application/json",
        // カスタムヘッダーを展開する
        ...(config.headers ?? {}),
      },
      // ボディを JSON 文字列にシリアライズして送信する
      body: JSON.stringify(sendBody),
      // AbortSignal を設定する（タイムアウト制御）
      signal: controller.signal,
    });

    // /send レスポンスが 200/204 以外の場合はエラーを投げる
    if (!sendResponse.ok) {
      // HTTP エラーレスポンスをエラーとして伝播する
      throw new Error(
        `paired POST-SSE /send 失敗: HTTP ${sendResponse.status} session_id=${sessionId}`,
      );
    }
  } finally {
    // タイムアウトタイマーをクリアする（正常完了 / エラー両方でクリアする）
    clearTimeout(timeoutId);
  }
}
