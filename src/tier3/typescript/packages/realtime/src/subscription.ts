// k1s0 tier3 SSE/WS subscription manager
// per-tab 16 subscription 上限 / UA aware adapter 5 経路 / 30s presence TTL
// HTTP/2 multiplex 必須（max_concurrent_streams 100）

// subscription conformance_class（4 種固定）
export type SubscriptionConformanceClass =
  // アラート通知
  | "v1_alert"
  // イベントフィード
  | "v1_event_feed"
  // ライブスナップショット
  | "v1_live_snapshot"
  // インタラクティブ（collaborative editing）
  | "v1_interactive";

// UA aware adapter 経路（5 種）
export type UaAdapterRoute =
  // Chrome/Edge 直接
  | "chrome_edge_direct"
  // Chrome/Edge 経由 corp proxy
  | "chrome_edge_via_corp_proxy"
  // Firefox/Safari
  | "firefox_safari"
  // .NET Companion
  | "dotnet_companion"
  // Tauri native
  | "tauri_native";

// per-tab の最大 subscription 数（HTTP/2 の場合）
const MAX_SUBSCRIPTIONS_H2 = 16;
// legacy HTTP/1.1 環境での最大 subscription 数（別ポート 8443-legacy）
const MAX_SUBSCRIPTIONS_H1 = 6;

// subscription の設定
export interface SubscriptionConfig {
  // conformance class
  readonly conformanceClass: SubscriptionConformanceClass;
  // aggregate ID（subscribe 対象）
  readonly aggregateId: string;
  // UA adapter 経路
  readonly uaRoute: UaAdapterRoute;
}

// subscription の状態
export type SubscriptionStatus =
  // 接続中
  | "connected"
  // バックグラウンドタブで間引き中
  | "throttled"
  // ネットワーク切断で一時停止中（resume_token 保持）
  | "paused"
  // 上限超過で queue 中
  | "queued"
  // 解放済み
  | "released";

// subscription manager（per-tab singleton）
export class SubscriptionManager {
  // アクティブな subscription の Map
  private readonly _subscriptions = new Map<string, SubscriptionStatus>();
  // legacy HTTP/1.1 環境かどうか
  private readonly _isLegacyH1: boolean;

  // constructor で HTTP バージョンを指定する
  constructor(isLegacyH1 = false) {
    this._isLegacyH1 = isLegacyH1;
  }

  // subscription の上限を返す（H2: 16 / H1: 6）
  get maxSubscriptions(): number {
    return this._isLegacyH1 ? MAX_SUBSCRIPTIONS_H1 : MAX_SUBSCRIPTIONS_H2;
  }

  // subscription 数を返す
  get subscriptionCount(): number {
    return this._subscriptions.size;
  }

  // subscription を追加する（上限超過時は queued ステータスで追加する）
  subscribe(config: SubscriptionConfig): { id: string; status: SubscriptionStatus } {
    // subscription ID を生成する
    const id = `${config.conformanceClass}:${config.aggregateId}`;
    // 上限チェック
    const activeCount = [...this._subscriptions.values()].filter(
      (s) => s === "connected" || s === "throttled",
    ).length;
    // 上限超過の場合は queue に積む
    const status: SubscriptionStatus =
      activeCount >= this.maxSubscriptions ? "queued" : "connected";
    this._subscriptions.set(id, status);
    return { id, status };
  }

  // subscription を解放する（unmount で呼ぶ）
  unsubscribe(id: string): void {
    // subscription を削除する
    this._subscriptions.delete(id);
  }

  // バックグラウンドタブに切り替わったとき throttle する
  throttle(id: string): void {
    // connected の場合のみ throttled に変更する
    if (this._subscriptions.get(id) === "connected") {
      this._subscriptions.set(id, "throttled");
    }
  }

  // ネットワーク切断時に pause する
  pause(id: string): void {
    // connected / throttled の場合に pause する
    const current = this._subscriptions.get(id);
    if (current === "connected" || current === "throttled") {
      this._subscriptions.set(id, "paused");
    }
  }

  // ネットワーク復帰時に resume する（resume_token を使用して再接続する）
  resume(id: string, _resumeToken?: string): void {
    // paused の場合のみ connected に戻す
    if (this._subscriptions.get(id) === "paused") {
      this._subscriptions.set(id, "connected");
    }
  }
}
