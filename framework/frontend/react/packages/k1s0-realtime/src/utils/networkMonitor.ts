/**
 * ネットワーク状態監視ユーティリティ
 */

export type NetworkStatusListener = (online: boolean) => void;

/**
 * ネットワークの online/offline 状態を監視するクラス
 */
export class NetworkMonitor {
  private listeners: Set<NetworkStatusListener> = new Set();
  private handleOnline: () => void;
  private handleOffline: () => void;

  constructor() {
    this.handleOnline = () => this.notify(true);
    this.handleOffline = () => this.notify(false);
  }

  /** 監視を開始する */
  start(): void {
    window.addEventListener('online', this.handleOnline);
    window.addEventListener('offline', this.handleOffline);
  }

  /** 監視を停止する */
  stop(): void {
    window.removeEventListener('online', this.handleOnline);
    window.removeEventListener('offline', this.handleOffline);
    this.listeners.clear();
  }

  /** 現在のオンライン状態を取得する */
  isOnline(): boolean {
    return navigator.onLine;
  }

  /** リスナーを追加する */
  addListener(listener: NetworkStatusListener): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  private notify(online: boolean): void {
    for (const listener of this.listeners) {
      listener(online);
    }
  }
}
