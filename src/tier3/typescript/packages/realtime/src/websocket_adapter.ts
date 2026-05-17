// k1s0 tier3 WebSocket Transport アダプター
// Transport インターフェースを WebSocket で実装する
// 30s heartbeat（ping/pong）と exponential backoff reconnect（500ms → cap 30s）を備える

import type { Transport, TransportOptions, TransportReadyState } from "./transport.js";

// heartbeat の ping メッセージ（サーバーと合意したフォーマット）
const PING_MESSAGE = JSON.stringify({ type: "ping" });
// デフォルト heartbeat 間隔（30 秒）
const DEFAULT_HEARTBEAT_MS = 30_000;
// デフォルト初期 reconnect 待機時間（500ms）
const DEFAULT_INITIAL_DELAY_MS = 500;
// デフォルト最大 reconnect 待機時間（30 秒）
const DEFAULT_MAX_DELAY_MS = 30_000;
// exponential backoff の乗数（2 倍ずつ増やす）
const BACKOFF_MULTIPLIER = 2;

// WebSocket アダプター実装クラス
export class WebSocketAdapter implements Transport {
  // 接続先 URL
  private readonly _url: string;
  // WebSocket サブプロトコル
  private readonly _protocols: string[];
  // heartbeat 間隔（ms）
  private readonly _heartbeatIntervalMs: number;
  // 初期 reconnect 待機時間（ms）
  private readonly _initialReconnectDelayMs: number;
  // 最大 reconnect 待機時間（ms）
  private readonly _maxReconnectDelayMs: number;
  // 現在の WebSocket インスタンス（未接続時は null）
  private _ws: WebSocket | null = null;
  // heartbeat タイマー ID（clearInterval 用）
  private _heartbeatTimer: ReturnType<typeof setInterval> | null = null;
  // reconnect タイマー ID（clearTimeout 用）
  private _reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  // 現在の reconnect 待機時間（exponential backoff で増加する）
  private _reconnectDelayMs: number;
  // 明示的 close 要求フラグ（true の場合は再接続しない）
  private _closed = false;
  // メッセージ受信コールバック一覧
  private readonly _messageCallbacks: Array<(msg: unknown) => void> = [];
  // 切断コールバック一覧
  private readonly _closeCallbacks: Array<() => void> = [];
  // 接続確立 Promise の resolve/reject（connect() で使用する）
  private _connectResolve: (() => void) | null = null;
  private _connectReject: ((err: Error) => void) | null = null;
  // 現在の接続状態
  private _readyState: TransportReadyState = "closed";

  // コンストラクタ（URL とオプションを受け取る）
  constructor(url: string, options?: TransportOptions) {
    // 接続先 URL を保持する
    this._url = url;
    // サブプロトコルを保持する（デフォルトは空）
    this._protocols = options?.protocols ?? [];
    // heartbeat 間隔を設定する（デフォルト 30s）
    this._heartbeatIntervalMs = options?.heartbeatIntervalMs ?? DEFAULT_HEARTBEAT_MS;
    // 初期 reconnect 待機時間を設定する
    this._initialReconnectDelayMs = options?.initialReconnectDelayMs ?? DEFAULT_INITIAL_DELAY_MS;
    // 最大 reconnect 待機時間を設定する
    this._maxReconnectDelayMs = options?.maxReconnectDelayMs ?? DEFAULT_MAX_DELAY_MS;
    // reconnect 待機時間を初期値で初期化する
    this._reconnectDelayMs = this._initialReconnectDelayMs;
  }

  // 現在の接続状態を返す
  get readyState(): TransportReadyState {
    return this._readyState;
  }

  // WebSocket 接続を開始する（接続確立まで await できる）
  connect(): Promise<void> {
    // 接続状態を connecting に更新する
    this._readyState = "connecting";
    // 明示的 close フラグをクリアする
    this._closed = false;
    // Promise を生成して resolve/reject を保持する
    return new Promise<void>((resolve, reject) => {
      // 接続確立・失敗ハンドラを保持する
      this._connectResolve = resolve;
      this._connectReject = reject;
      // WebSocket を生成する
      this._openWebSocket();
    });
  }

  // WebSocket インスタンスを生成してイベントを紐付ける内部メソッド
  private _openWebSocket(): void {
    // 前の WebSocket が残っている場合はクリアする
    if (this._ws !== null) {
      // イベントハンドラを削除してリークを防ぐ
      this._ws.onopen = null;
      this._ws.onmessage = null;
      this._ws.onerror = null;
      this._ws.onclose = null;
    }
    // 新しい WebSocket を生成する
    this._ws = new WebSocket(this._url, this._protocols.length > 0 ? this._protocols : undefined);

    // 接続確立時のハンドラ
    this._ws.onopen = () => {
      // 接続状態を open に更新する
      this._readyState = "open";
      // reconnect 待機時間を初期値にリセットする（成功したので backoff をリセット）
      this._reconnectDelayMs = this._initialReconnectDelayMs;
      // heartbeat タイマーを開始する
      this._startHeartbeat();
      // connect() の Promise を resolve する
      if (this._connectResolve !== null) {
        this._connectResolve();
        this._connectResolve = null;
        this._connectReject = null;
      }
    };

    // メッセージ受信時のハンドラ
    this._ws.onmessage = (event: MessageEvent) => {
      // pong メッセージは heartbeat 応答なので無視する
      let parsed: unknown;
      try {
        // JSON パースを試みる
        parsed = JSON.parse(event.data as string);
      } catch {
        // パース失敗時は生の data をそのまま使う
        parsed = event.data;
      }
      // pong の場合はスキップする
      if (
        typeof parsed === "object" &&
        parsed !== null &&
        "type" in parsed &&
        (parsed as Record<string, unknown>)["type"] === "pong"
      ) {
        return;
      }
      // 登録済みコールバックに通知する
      for (const cb of this._messageCallbacks) {
        cb(parsed);
      }
    };

    // エラー発生時のハンドラ（onclose が続けて発火するため reconnect は onclose に委譲）
    this._ws.onerror = () => {
      // 接続前のエラーの場合は connect() の Promise を reject する
      if (this._connectReject !== null) {
        this._connectReject(new Error("WebSocket connection error"));
        this._connectResolve = null;
        this._connectReject = null;
      }
    };

    // 切断時のハンドラ
    this._ws.onclose = () => {
      // heartbeat タイマーを停止する
      this._stopHeartbeat();
      // 接続状態を closed に更新する
      this._readyState = "closed";
      // 切断コールバックを呼ぶ
      for (const cb of this._closeCallbacks) {
        cb();
      }
      // 明示的 close の場合は再接続しない
      if (this._closed) {
        return;
      }
      // exponential backoff で再接続をスケジュールする
      this._scheduleReconnect();
    };
  }

  // heartbeat タイマーを開始する（30s ごとに ping を送信する）
  private _startHeartbeat(): void {
    // 既存のタイマーを停止する
    this._stopHeartbeat();
    // 指定間隔で ping を送信する
    this._heartbeatTimer = setInterval(() => {
      // WebSocket が open の場合のみ ping を送信する
      if (this._ws?.readyState === WebSocket.OPEN) {
        this._ws.send(PING_MESSAGE);
      }
    }, this._heartbeatIntervalMs);
  }

  // heartbeat タイマーを停止する
  private _stopHeartbeat(): void {
    // タイマーが存在する場合のみクリアする
    if (this._heartbeatTimer !== null) {
      clearInterval(this._heartbeatTimer);
      this._heartbeatTimer = null;
    }
  }

  // exponential backoff で再接続をスケジュールする
  private _scheduleReconnect(): void {
    // 現在の待機時間後に再接続する
    this._reconnectTimer = setTimeout(() => {
      // 再接続前に明示的 close の場合はスキップする
      if (this._closed) {
        return;
      }
      // WebSocket を再生成する
      this._openWebSocket();
    }, this._reconnectDelayMs);
    // 次回の待機時間を 2 倍にする（上限を cap する）
    this._reconnectDelayMs = Math.min(
      this._reconnectDelayMs * BACKOFF_MULTIPLIER,
      this._maxReconnectDelayMs,
    );
  }

  // メッセージを送信する（open 状態のみ有効）
  send(msg: unknown): void {
    // open 状態でない場合は何もしない
    if (this._ws === null || this._ws.readyState !== WebSocket.OPEN) {
      return;
    }
    // JSON 文字列に変換して送信する
    this._ws.send(JSON.stringify(msg));
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

  // 接続を明示的に閉じる（再接続ループも停止する）
  close(): void {
    // 明示的 close フラグを立てる
    this._closed = true;
    // heartbeat タイマーを停止する
    this._stopHeartbeat();
    // reconnect タイマーを停止する
    if (this._reconnectTimer !== null) {
      clearTimeout(this._reconnectTimer);
      this._reconnectTimer = null;
    }
    // WebSocket を閉じる
    if (this._ws !== null) {
      this._ws.close();
      this._ws = null;
    }
    // 接続状態を closed に更新する
    this._readyState = "closed";
  }
}

// WebSocketAdapter を生成するファクトリ関数
export function createWebSocketAdapter(url: string, options?: TransportOptions): Transport {
  // 新しい WebSocketAdapter インスタンスを返す
  return new WebSocketAdapter(url, options);
}
