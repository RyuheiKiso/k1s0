import type { WsConfig, WsMessage, ConnectionState } from './types.js';
import type { WsClient } from './client.js';

// NativeWsClient はブラウザ標準および Node.js の WebSocket API を使用した本番実装。
// 自動再接続、Ping/Pong ハートビート、メッセージキューイングをサポートする。
export class NativeWsClient implements WsClient {
  private ws: WebSocket | null = null;
  private _state: ConnectionState = 'disconnected';
  // receive() が待機中の場合に解決するResolver キュー
  private receiveResolvers: Array<(msg: WsMessage) => void> = [];
  // 受信済みだが receive() で取り出されていないメッセージキュー
  private receiveQueue: WsMessage[] = [];
  private reconnectAttempts = 0;
  private pingTimer: ReturnType<typeof setInterval> | null = null;
  // disconnect() 後の自動再接続を防止するフラグ
  private stopping = false;

  constructor(private readonly config: WsConfig) {}

  get state(): ConnectionState {
    return this._state;
  }

  // connect はWebSocket 接続を確立する。接続済みの場合はエラーをスローする。
  async connect(): Promise<void> {
    if (this._state !== 'disconnected') {
      throw new Error('Already connected');
    }
    this.stopping = false;
    this._state = 'connecting';
    await this.doConnect();
  }

  // disconnect は接続を閉じてリソースを解放する。
  async disconnect(): Promise<void> {
    if (this._state === 'disconnected') {
      throw new Error('Not connected');
    }
    this.stopping = true;
    this._state = 'closing';
    this.stopPingTimer();
    this.ws?.close(1000, 'Normal closure');
    this.ws = null;
    this._state = 'disconnected';
  }

  // send はメッセージを WebSocket に送信する。
  async send(message: WsMessage): Promise<void> {
    if (this._state !== 'connected' || this.ws === null) {
      throw new Error('Not connected');
    }
    if (message.type === 'text') {
      this.ws.send(message.payload as string);
    } else if (message.type === 'binary') {
      this.ws.send(message.payload as Uint8Array);
    } else if (message.type === 'close') {
      this.ws.close();
    }
    // ブラウザの WebSocket は Ping/Pong フレームをアプリ層で送信できないため無視する
  }

  // receive は次のメッセージを返す。メッセージがなければ到着まで待機する。
  async receive(): Promise<WsMessage> {
    if (this._state !== 'connected') {
      throw new Error('Not connected');
    }
    if (this.receiveQueue.length > 0) {
      return this.receiveQueue.shift()!;
    }
    return new Promise<WsMessage>((resolve) => {
      this.receiveResolvers.push(resolve);
    });
  }

  // doConnect は実際の WebSocket 接続を確立するPromiseを返す。
  // 再接続時にも使用する。
  private doConnect(): Promise<void> {
    return new Promise<void>((resolve, reject) => {
      const ws = new WebSocket(this.config.url);

      const onOpen = () => {
        this._state = 'connected';
        this.reconnectAttempts = 0;
        this.ws = ws;
        this.startPingTimer();
        cleanup();
        resolve();
      };

      const onError = () => {
        cleanup();
        reject(new Error('WebSocket connection failed'));
      };

      const cleanup = () => {
        ws.removeEventListener('open', onOpen);
        ws.removeEventListener('error', onError);
      };

      ws.addEventListener('open', onOpen);
      ws.addEventListener('error', onError);

      // 接続が閉じた場合は自動再接続を試みる
      ws.addEventListener('close', () => {
        this.stopPingTimer();
        if (!this.stopping && this._state !== 'disconnected') {
          this.scheduleReconnect();
        }
      });

      // 受信メッセージを待機中のresolverまたはキューに渡す
      ws.addEventListener('message', (event: MessageEvent) => {
        const msg = this.parseMessage(event);
        if (msg !== null) {
          if (this.receiveResolvers.length > 0) {
            this.receiveResolvers.shift()!(msg);
          } else {
            this.receiveQueue.push(msg);
          }
        }
      });
    });
  }

  // scheduleReconnect は再接続のスケジュールを管理する。
  // 試行回数が上限に達した場合は disconnected 状態に遷移する。
  private scheduleReconnect(): void {
    if (
      !this.config.reconnect ||
      this.reconnectAttempts >= this.config.maxReconnectAttempts
    ) {
      this._state = 'disconnected';
      return;
    }
    this._state = 'reconnecting';
    this.reconnectAttempts++;

    setTimeout(async () => {
      try {
        await this.doConnect();
      } catch {
        this.scheduleReconnect();
      }
    }, this.config.reconnectDelayMs);
  }

  // startPingTimer は ping インターバルが設定されている場合にタイマーを開始する。
  // ブラウザの WebSocket は Ping フレームをサポートしないため、接続維持用の空送信は行わない。
  private startPingTimer(): void {
    if (this.config.pingIntervalMs && this.config.pingIntervalMs > 0) {
      this.pingTimer = setInterval(() => {
        // ブラウザ WebSocket は Ping フレームをアプリ層から送信できないため何もしない
        // サーバーから Ping を受信した場合は自動的に Pong を返す
      }, this.config.pingIntervalMs);
    }
  }

  // stopPingTimer は ping タイマーを停止する。
  private stopPingTimer(): void {
    if (this.pingTimer !== null) {
      clearInterval(this.pingTimer);
      this.pingTimer = null;
    }
  }

  // parseMessage は MessageEvent を WsMessage に変換する。
  private parseMessage(event: MessageEvent): WsMessage | null {
    if (typeof event.data === 'string') {
      return { type: 'text', payload: event.data };
    } else if (event.data instanceof ArrayBuffer) {
      return { type: 'binary', payload: new Uint8Array(event.data) };
    } else if (event.data instanceof Uint8Array) {
      return { type: 'binary', payload: event.data };
    }
    return null;
  }
}
