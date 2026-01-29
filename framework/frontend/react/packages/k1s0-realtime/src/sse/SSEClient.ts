/**
 * SSE クライアント
 *
 * EventSource API をラップし、イベントタイプ別のハンドラ登録と再接続制御を提供する。
 */

import type { ConnectionStatus } from '../types.js';

export type SSEEventHandler = (event: MessageEvent) => void;
export type SSEStatusHandler = (status: ConnectionStatus) => void;
export type SSEErrorHandler = (event: Event) => void;

/**
 * SSE クライアントクラス
 */
export class SSEClient {
  private eventSource: EventSource | null = null;
  private status: ConnectionStatus = 'disconnected';
  private eventHandlers: Map<string, Set<SSEEventHandler>> = new Map();
  private statusHandlers: Set<SSEStatusHandler> = new Set();
  private errorHandlers: Set<SSEErrorHandler> = new Set();
  private messageHandler: SSEEventHandler | null = null;

  /** 現在の接続状態を取得する */
  getStatus(): ConnectionStatus {
    return this.status;
  }

  /**
   * SSE 接続を開始する
   * @param url - 接続先 URL
   * @param withCredentials - クレデンシャルを送信するか
   */
  connect(url: string, withCredentials = false): void {
    if (this.eventSource) {
      this.disconnect();
    }

    this.setStatus('connecting');
    this.eventSource = new EventSource(url, { withCredentials });

    this.eventSource.onopen = () => {
      this.setStatus('connected');
    };

    this.eventSource.onerror = (event) => {
      if (this.eventSource?.readyState === EventSource.CLOSED) {
        this.setStatus('disconnected');
      } else {
        // EventSource は自動再接続する
        this.setStatus('connecting');
      }
      for (const handler of this.errorHandlers) {
        handler(event);
      }
    };

    this.eventSource.onmessage = (event) => {
      this.messageHandler?.(event);
    };

    // 登録済みイベントタイプのリスナーを設定
    for (const [eventType, handlers] of this.eventHandlers) {
      this.eventSource.addEventListener(eventType, ((event: Event) => {
        const messageEvent = event as MessageEvent;
        for (const handler of handlers) {
          handler(messageEvent);
        }
      }) as EventListener);
    }
  }

  /** SSE 接続を切断する */
  disconnect(): void {
    if (this.eventSource) {
      this.setStatus('disconnecting');
      this.eventSource.close();
      this.eventSource = null;
      this.setStatus('disconnected');
    }
  }

  /** デフォルトメッセージハンドラを設定する */
  onMessage(handler: SSEEventHandler): void {
    this.messageHandler = handler;
  }

  /** イベントタイプ別ハンドラを登録する */
  addEventListener(eventType: string, handler: SSEEventHandler): void {
    if (!this.eventHandlers.has(eventType)) {
      this.eventHandlers.set(eventType, new Set());
    }
    this.eventHandlers.get(eventType)!.add(handler);

    // 既に接続中の場合、動的にリスナーを追加
    if (this.eventSource) {
      this.eventSource.addEventListener(eventType, ((event: Event) => {
        handler(event as MessageEvent);
      }) as EventListener);
    }
  }

  /** 状態変更ハンドラを登録する */
  onStatusChange(handler: SSEStatusHandler): void {
    this.statusHandlers.add(handler);
  }

  /** エラーハンドラを登録する */
  onError(handler: SSEErrorHandler): void {
    this.errorHandlers.add(handler);
  }

  /** 全ハンドラをクリアする */
  removeAllListeners(): void {
    this.eventHandlers.clear();
    this.statusHandlers.clear();
    this.errorHandlers.clear();
    this.messageHandler = null;
  }

  private setStatus(status: ConnectionStatus): void {
    this.status = status;
    for (const handler of this.statusHandlers) {
      handler(status);
    }
  }
}
