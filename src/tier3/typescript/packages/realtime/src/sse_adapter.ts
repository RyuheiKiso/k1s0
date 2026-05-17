// k1s0 tier3 SSE (Server-Sent Events) Transport アダプター
// EventSource を使用した server → client 単方向ストリームを実装する
// Last-Event-ID ヘッダーによる resume token でネットワーク切断後に再開できる

import type { Transport, TransportOptions, TransportReadyState } from "./transport.js";

// SSE アダプター実装クラス（server → client 単方向）
// send() は no-op として実装する（SSE は server → client のみ）
export class SseAdapter implements Transport {
  // 接続先 URL
  private readonly _url: string;
  // Last-Event-ID 初期値（resume token。接続中は EventSource が自動管理する）
  private _lastEventId: string;
  // 現在の EventSource インスタンス（未接続時は null）
  private _es: EventSource | null = null;
  // 明示的 close フラグ（true の場合は再接続しない）
  private _closed = false;
  // メッセージ受信コールバック一覧
  private readonly _messageCallbacks: Array<(msg: unknown) => void> = [];
  // 切断コールバック一覧
  private readonly _closeCallbacks: Array<() => void> = [];
  // connect() の resolve/reject
  private _connectResolve: (() => void) | null = null;
  private _connectReject: ((err: Error) => void) | null = null;
  // 現在の接続状態
  private _readyState: TransportReadyState = "closed";

  // コンストラクタ（URL とオプションを受け取る）
  constructor(url: string, options?: TransportOptions) {
    // 接続先 URL を保持する
    this._url = url;
    // Last-Event-ID 初期値を設定する（ない場合は空文字）
    this._lastEventId = options?.lastEventId ?? "";
  }

  // 現在の接続状態を返す
  get readyState(): TransportReadyState {
    return this._readyState;
  }

  // SSE 接続を開始する（接続確立まで await できる）
  connect(): Promise<void> {
    // 接続状態を connecting に更新する
    this._readyState = "connecting";
    // 明示的 close フラグをクリアする
    this._closed = false;
    // Promise を生成して resolve/reject を保持する
    return new Promise<void>((resolve, reject) => {
      // ハンドラを保持する
      this._connectResolve = resolve;
      this._connectReject = reject;
      // EventSource を生成する
      this._openEventSource();
    });
  }

  // EventSource インスタンスを生成してイベントを紐付ける内部メソッド
  private _openEventSource(): void {
    // 前の EventSource が残っている場合はクリアする
    if (this._es !== null) {
      this._es.close();
      this._es = null;
    }
    // Last-Event-ID が存在する場合は URL クエリパラメータに付加する
    // （EventSource は Last-Event-ID ヘッダーを自動送信するが、初回接続では URL パラメータが確実）
    const url = this._lastEventId !== ""
      ? `${this._url}?lastEventId=${encodeURIComponent(this._lastEventId)}`
      : this._url;
    // EventSource を生成する（CORS 対応: withCredentials = true）
    this._es = new EventSource(url, { withCredentials: true });

    // 接続確立時のハンドラ（onopen は接続成功時に発火する）
    this._es.onopen = () => {
      // 接続状態を open に更新する
      this._readyState = "open";
      // connect() の Promise を resolve する
      if (this._connectResolve !== null) {
        this._connectResolve();
        this._connectResolve = null;
        this._connectReject = null;
      }
    };

    // メッセージ受信時のハンドラ（名前なし event = "message"）
    this._es.onmessage = (event: MessageEvent) => {
      // Last-Event-ID を更新する（次の reconnect 時の resume token に使う）
      if (typeof event.lastEventId === "string" && event.lastEventId !== "") {
        this._lastEventId = event.lastEventId;
      }
      // JSON パースを試みる
      let parsed: unknown;
      try {
        parsed = JSON.parse(event.data as string);
      } catch {
        // パース失敗時は生の data をそのまま使う
        parsed = event.data;
      }
      // 登録済みコールバックに通知する
      for (const cb of this._messageCallbacks) {
        cb(parsed);
      }
    };

    // エラー時のハンドラ（EventSource は自動 reconnect する仕様だが、close 時はコールバックを呼ぶ）
    this._es.onerror = () => {
      // 接続前のエラーの場合は connect() の Promise を reject する
      if (this._connectReject !== null) {
        this._connectReject(new Error("SSE connection error"));
        this._connectResolve = null;
        this._connectReject = null;
      }
      // CLOSED 状態になった場合は切断コールバックを呼ぶ
      if (this._es?.readyState === EventSource.CLOSED) {
        // 接続状態を closed に更新する
        this._readyState = "closed";
        // 切断コールバックを呼ぶ
        for (const cb of this._closeCallbacks) {
          cb();
        }
        // 明示的 close でない場合は EventSource が自動 reconnect するため何もしない
      }
    };
  }

  // 送信は SSE 非対応のため no-op（server → client 専用）
  send(_msg: unknown): void {
    // SSE は server → client 単方向のため送信できない（no-op）
  }

  // メッセージ受信コールバックを登録する
  onMessage(cb: (msg: unknown) => void): void {
    // コールバックを配列に追加する
    this._messageCallbacks.push(cb);
  }

  // 切断コールバックを登録する
  onClose(cb: () => void): void {
    // コールバックを配列に追加する
    this._closeCallbacks.push(cb);
  }

  // 接続を明示的に閉じる（再接続を停止する）
  close(): void {
    // 明示的 close フラグを立てる
    this._closed = true;
    // EventSource を閉じる
    if (this._es !== null) {
      this._es.close();
      this._es = null;
    }
    // 接続状態を closed に更新する
    this._readyState = "closed";
  }
}

// SseAdapter を生成するファクトリ関数
export function createSseAdapter(url: string, options?: TransportOptions): Transport {
  // 新しい SseAdapter インスタンスを返す
  return new SseAdapter(url, options);
}
