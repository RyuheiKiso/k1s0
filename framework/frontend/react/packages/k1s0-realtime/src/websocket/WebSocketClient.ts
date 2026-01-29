/**
 * WebSocket クライアント
 *
 * ネイティブ WebSocket API をラップし、イベントエミッター機能を提供する。
 */

import type { ConnectionStatus } from '../types.js';

export type WebSocketEventMap = {
  open: Event;
  close: CloseEvent;
  error: Event;
  message: MessageEvent;
  statusChange: ConnectionStatus;
};

type EventHandler<K extends keyof WebSocketEventMap> = (event: WebSocketEventMap[K]) => void;

/**
 * WebSocket クライアントクラス
 */
export class WebSocketClient {
  private socket: WebSocket | null = null;
  private status: ConnectionStatus = 'disconnected';
  private handlers: { [K in keyof WebSocketEventMap]?: Set<EventHandler<K>> } = {};

  /** 現在の接続状態を取得する */
  getStatus(): ConnectionStatus {
    return this.status;
  }

  /** WebSocket インスタンスを取得する */
  getSocket(): WebSocket | null {
    return this.socket;
  }

  /**
   * WebSocket 接続を開始する
   * @param url - 接続先 URL
   * @param protocols - サブプロトコル
   */
  connect(url: string, protocols?: string | string[]): void {
    if (this.socket && (this.socket.readyState === WebSocket.CONNECTING || this.socket.readyState === WebSocket.OPEN)) {
      return;
    }

    this.setStatus('connecting');
    this.socket = new WebSocket(url, protocols);

    this.socket.onopen = (event) => {
      this.setStatus('connected');
      this.emit('open', event);
    };

    this.socket.onclose = (event) => {
      this.setStatus('disconnected');
      this.emit('close', event);
    };

    this.socket.onerror = (event) => {
      this.emit('error', event);
    };

    this.socket.onmessage = (event) => {
      this.emit('message', event);
    };
  }

  /**
   * WebSocket 接続を切断する
   * @param code - クローズコード
   * @param reason - クローズ理由
   */
  disconnect(code?: number, reason?: string): void {
    if (!this.socket) return;

    if (this.socket.readyState === WebSocket.OPEN || this.socket.readyState === WebSocket.CONNECTING) {
      this.setStatus('disconnecting');
      this.socket.close(code, reason);
    }

    this.socket = null;
  }

  /**
   * メッセージを送信する
   * @param data - 送信データ
   */
  send(data: string | ArrayBufferLike | Blob | ArrayBufferView): void {
    if (!this.socket || this.socket.readyState !== WebSocket.OPEN) {
      throw new Error('WebSocket is not connected');
    }
    this.socket.send(data);
  }

  /** イベントハンドラを登録する */
  on<K extends keyof WebSocketEventMap>(event: K, handler: EventHandler<K>): void {
    if (!this.handlers[event]) {
      this.handlers[event] = new Set();
    }
    (this.handlers[event] as Set<EventHandler<K>>).add(handler);
  }

  /** イベントハンドラを解除する */
  off<K extends keyof WebSocketEventMap>(event: K, handler: EventHandler<K>): void {
    const set = this.handlers[event] as Set<EventHandler<K>> | undefined;
    set?.delete(handler);
  }

  /** 全イベントハンドラをクリアする */
  removeAllListeners(): void {
    this.handlers = {};
  }

  private setStatus(status: ConnectionStatus): void {
    this.status = status;
    this.emit('statusChange', status);
  }

  private emit<K extends keyof WebSocketEventMap>(event: K, data: WebSocketEventMap[K]): void {
    const set = this.handlers[event] as Set<EventHandler<K>> | undefined;
    if (set) {
      for (const handler of set) {
        handler(data);
      }
    }
  }
}
