// long_poll_adapter.ts — SSE + long-poll フォールバック Transport 実装
// route_selector.ts の createLongPollAdapter に接続する
// corp_proxy 配下でも動作するように fetch ベースの long-poll を実装する
// UA 文字列から corp_proxy を検出してフォールバック方針を決定する
// proxy 配下では SSE が切断される場合があるため long-poll を優先する

// Transport インターフェースと TransportOptions をインポートする
import type { Transport, TransportOptions, TransportReadyState } from "./transport.js";

// corp proxy と判断する UA 文字列のパターン（一般的な企業プロキシシグネチャ）
// User-Agent や Proxy ヘッダの特徴からプロキシ配下かを推測する
const CORP_PROXY_UA_PATTERNS = [
  // Zscaler プロキシの UA シグネチャ
  'ZscalerClient',
  // Blue Coat / Symantec プロキシのシグネチャ
  'BCSYMANTEC',
  // Cisco Umbrella のシグネチャ
  'Cisco Umbrella',
  // McAfee Web Gateway のシグネチャ
  'McAfee Web Gateway',
  // Forcepoint (WebSense) のシグネチャ
  'Forcepoint',
  // Palo Alto GlobalProtect のシグネチャ
  'GlobalProtect',
  // 汎用プロキシシグネチャ（一部の企業プロキシが付加する）
  'CorpProxy',
] as const;

// UA 文字列から corp_proxy 配下かを検出するヘルパー関数
// corp_proxy 配下では SSE が切断されやすいため long-poll を優先する
export function detectCorpProxy(ua: string): boolean {
  // UA 文字列を正規化する（大文字・小文字の揺れを吸収する）
  const normalizedUa = ua.toLowerCase();
  // 各プロキシシグネチャを順番に確認する
  for (const pattern of CORP_PROXY_UA_PATTERNS) {
    // シグネチャが UA 文字列に含まれる場合は corp_proxy と判断する
    if (normalizedUa.includes(pattern.toLowerCase())) {
      return true;
    }
  }
  // どのシグネチャにも一致しない場合は corp_proxy 以外と判断する
  return false;
}

// LongPollAdapter は SSE + long-poll フォールバックを実装する Transport
// SSE が切断された場合は fetch ベースの long-poll に自動フォールバックする
export class LongPollAdapter implements Transport {
  // 接続先 URL
  private readonly _url: string;
  // Transport オプション（再接続設定等）
  private readonly _options: TransportOptions | undefined;
  // 現在の接続状態
  private _readyState: TransportReadyState = 'connecting';
  // メッセージ受信コールバックの一覧
  private readonly _messageCallbacks: Array<(msg: unknown) => void> = [];
  // 切断コールバックの一覧
  private readonly _closeCallbacks: Array<() => void> = [];
  // SSE の EventSource インスタンス（SSE モード時に使用する）
  private _eventSource: EventSource | null = null;
  // long-poll の abort controller（fetch のキャンセルに使用する）
  private _pollAbortController: AbortController | null = null;
  // long-poll ループの実行フラグ
  private _polling = false;
  // corp_proxy 配下フラグ（UA 検出の結果を保持する）
  private readonly _isCorpProxy: boolean;
  // 再接続の初期待機時間（ミリ秒）
  private readonly _initialReconnectDelayMs: number;
  // 再接続の最大待機時間（ミリ秒）
  private readonly _maxReconnectDelayMs: number;

  // コンストラクタ（URL とオプションを受け取る）
  constructor(url: string, options?: TransportOptions) {
    // URL を設定する
    this._url = url;
    // オプションを設定する
    this._options = options;
    // 再接続の初期待機時間を設定する（デフォルト 500ms）
    this._initialReconnectDelayMs = options?.initialReconnectDelayMs ?? 500;
    // 再接続の最大待機時間を設定する（デフォルト 30000ms）
    this._maxReconnectDelayMs = options?.maxReconnectDelayMs ?? 30_000;
    // UA 文字列から corp_proxy 配下かを検出する
    const ua = typeof navigator !== 'undefined' ? navigator.userAgent : '';
    // UA 文字列で corp_proxy を検出する
    this._isCorpProxy = detectCorpProxy(ua);
  }

  // 現在の接続状態を返す
  get readyState(): TransportReadyState {
    return this._readyState;
  }

  // 接続を開始する（corp_proxy 配下か否かで SSE / long-poll を選択する）
  async connect(): Promise<void> {
    // 接続状態を connecting に設定する
    this._readyState = 'connecting';
    // corp_proxy 配下の場合は直接 long-poll から開始する（SSE は切断されやすい）
    if (this._isCorpProxy) {
      // corp_proxy 配下では long-poll を優先する
      console.info('[LongPollAdapter] corp_proxy detected, using long-poll directly');
      // long-poll ループを開始する
      this._startLongPoll();
      // 接続状態を open に設定する
      this._readyState = 'open';
      return;
    }
    // corp_proxy 以外の場合は SSE を試みる（失敗時は long-poll にフォールバック）
    if (typeof EventSource !== 'undefined') {
      // SSE 接続を試みる
      try {
        // SSE 接続を確立する
        await this._connectSSE();
        // SSE 接続成功時は open 状態に遷移する
        this._readyState = 'open';
        return;
      } catch {
        // SSE 接続失敗時は long-poll にフォールバックする
        console.info('[LongPollAdapter] SSE failed, falling back to long-poll');
      }
    }
    // SSE が利用できない場合または失敗した場合は long-poll を使用する
    this._startLongPoll();
    // 接続状態を open に設定する
    this._readyState = 'open';
  }

  // SSE 接続を確立するヘルパー
  private async _connectSSE(): Promise<void> {
    // SSE 接続のタイムアウト時間（5 秒）
    const SSE_TIMEOUT_MS = 5_000;
    // Promise で SSE 接続の成否を待機する
    return new Promise((resolve, reject) => {
      // EventSource インスタンスを生成する
      const es = new EventSource(this._url);
      // SSE インスタンスを保持する
      this._eventSource = es;
      // タイムアウトタイマーを設定する
      const timeout = setTimeout(() => {
        // タイムアウト時は SSE 接続を閉じてエラーを返す
        es.close();
        // タイムアウトエラーで reject する
        reject(new Error('SSE connection timeout'));
      }, SSE_TIMEOUT_MS);
      // SSE 接続成功ハンドラ
      es.onopen = (): void => {
        // タイムアウトタイマーをクリアする
        clearTimeout(timeout);
        // resolve して接続成功を通知する
        resolve();
      };
      // SSE メッセージ受信ハンドラ
      es.onmessage = (event: MessageEvent): void => {
        // 受信データを JSON パースして callback に渡す
        try {
          // JSON 文字列をパースする
          const parsed: unknown = JSON.parse(event.data);
          // 全登録コールバックを呼び出す
          for (const cb of this._messageCallbacks) {
            cb(parsed);
          }
        } catch {
          // JSON パースエラーは無視して生文字列を渡す
          for (const cb of this._messageCallbacks) {
            cb(event.data);
          }
        }
      };
      // SSE エラーハンドラ（切断時は long-poll にフォールバックする）
      es.onerror = (): void => {
        // タイムアウトタイマーをクリアする
        clearTimeout(timeout);
        // SSE 接続を閉じる
        es.close();
        // SSE インスタンスをリセットする
        this._eventSource = null;
        // 既に open 状態の場合は long-poll にフォールバックする
        if (this._readyState === 'open') {
          // corp_proxy による切断の可能性があるため long-poll にフォールバックする
          console.info('[LongPollAdapter] SSE disconnected, falling back to long-poll');
          // long-poll ループを開始する
          this._startLongPoll();
        } else {
          // 接続前エラーは reject する
          reject(new Error('SSE connection error'));
        }
      };
    });
  }

  // long-poll ループを開始するヘルパー
  private _startLongPoll(): void {
    // 既にポーリング中の場合は何もしない
    if (this._polling) {
      return;
    }
    // ポーリングフラグを立てる
    this._polling = true;
    // AbortController を生成する（close() 時のキャンセルに使用する）
    this._pollAbortController = new AbortController();
    // long-poll ループを非同期で開始する（await を待たない fire-and-forget）
    void this._pollLoop(this._pollAbortController.signal);
  }

  // long-poll ループの本体（再接続バックオフ付き）
  private async _pollLoop(signal: AbortSignal): Promise<void> {
    // 再接続待機時間を指数バックオフで増加させる
    let delayMs = this._initialReconnectDelayMs;
    // シグナルがキャンセルされるまでループする
    while (!signal.aborted) {
      try {
        // last-event-id を URL クエリパラメータに付与する（resume token）
        const lastEventId = this._options?.lastEventId;
        // poll URL を構築する（last-event-id を付与する）
        const pollUrl = lastEventId
          ? `${this._url}?lastEventId=${encodeURIComponent(lastEventId)}`
          : this._url;
        // fetch で long-poll リクエストを送信する（タイムアウトは 30 秒）
        const response = await fetch(pollUrl, {
          // GET リクエストで long-poll を開始する
          method: 'GET',
          // キャッシュを無効化する（新鮮なデータを取得する）
          cache: 'no-store',
          // クレデンシャルを同一オリジンのみに制限する
          credentials: 'same-origin',
          // AbortController でキャンセル可能にする
          signal,
          // Accept ヘッダで long-poll を要求する（サーバーが判断する）
          headers: { 'Accept': 'application/json' },
        });
        // レスポンスが成功でない場合はバックオフして再試行する
        if (!response.ok) {
          // エラーログを出力する
          console.warn('[LongPollAdapter] poll request failed, status:', response.status);
          // バックオフ待機する
          await this._delay(delayMs, signal);
          // バックオフ時間を増加させる（最大値を超えない）
          delayMs = Math.min(delayMs * 2, this._maxReconnectDelayMs);
          continue;
        }
        // レスポンスボディを JSON として取得する
        const data: unknown = await response.json();
        // 成功した場合はバックオフをリセットする
        delayMs = this._initialReconnectDelayMs;
        // 全登録コールバックを呼び出す
        for (const cb of this._messageCallbacks) {
          cb(data);
        }
      } catch (error) {
        // AbortError はループ終了シグナルなので再スローしない
        if (error instanceof DOMException && error.name === 'AbortError') {
          break;
        }
        // その他のエラーはバックオフして再試行する
        console.warn('[LongPollAdapter] poll error:', error);
        // バックオフ待機する
        await this._delay(delayMs, signal);
        // バックオフ時間を増加させる（最大値を超えない）
        delayMs = Math.min(delayMs * 2, this._maxReconnectDelayMs);
      }
    }
    // ポーリングフラグをリセットする
    this._polling = false;
    // close コールバックを呼び出す
    for (const cb of this._closeCallbacks) {
      cb();
    }
  }

  // 指定時間待機するヘルパー（AbortSignal でキャンセル可能）
  private _delay(ms: number, signal: AbortSignal): Promise<void> {
    // Promise で待機する（シグナルがキャンセルされた場合は即時 resolve する）
    return new Promise((resolve) => {
      // タイムアウトタイマーを設定する
      const timeout = setTimeout(resolve, ms);
      // AbortSignal のキャンセルイベントを監視する
      signal.addEventListener('abort', () => {
        // タイムアウトをクリアして即時 resolve する
        clearTimeout(timeout);
        resolve();
      }, { once: true });
    });
  }

  // メッセージを送信する（long-poll は server → client 専用なので no-op）
  send(_msg: unknown): void {
    // long-poll は server→client 専用のため送信は no-op として実装する
    // client → server の送信が必要な場合は別途 fetch POST を使用すること
  }

  // メッセージ受信コールバックを登録する
  onMessage(cb: (msg: unknown) => void): void {
    // コールバックをリストに追加する
    this._messageCallbacks.push(cb);
  }

  // 接続切断コールバックを登録する
  onClose(cb: () => void): void {
    // コールバックをリストに追加する
    this._closeCallbacks.push(cb);
  }

  // 接続を明示的に閉じる
  close(): void {
    // 接続状態を closed に設定する
    this._readyState = 'closed';
    // SSE が接続中の場合は閉じる
    if (this._eventSource !== null) {
      this._eventSource.close();
      this._eventSource = null;
    }
    // long-poll が動いている場合は中断する
    if (this._pollAbortController !== null) {
      this._pollAbortController.abort();
      this._pollAbortController = null;
    }
  }
}

// LongPollAdapter のファクトリ関数（route_selector.ts の createLongPollAdapter として使用する）
// UA 文字列から corp_proxy を検出して適切な接続方式を選択する
export function createLongPollAdapter(url: string, options?: TransportOptions): Transport {
  // LongPollAdapter インスタンスを生成して返す
  return new LongPollAdapter(url, options);
}
