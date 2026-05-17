// k1s0 tier3 realtime Transport インターフェース
// WebSocket / SSE / long-poll 各アダプターが実装する共通契約を定義する
// UA aware adapter 経路（subscription.ts の UaAdapterRoute）の物理基盤となる

// Transport の接続状態（3 種）
export type TransportReadyState =
  // 接続確立中
  | "connecting"
  // 接続済み（送受信可能）
  | "open"
  // 切断済み（再接続待ちまたは終了）
  | "closed";

// Transport の共通インターフェース
// 全アダプター（WebSocket / SSE / long-poll）がこの型を実装する
export interface Transport {
  // 接続を開始する（Promise は接続確立またはエラーで resolve/reject する）
  connect(): Promise<void>;
  // メッセージを送信する（server → client 専用の SSE は no-op として実装してよい）
  send(msg: unknown): void;
  // メッセージ受信時に呼ばれるコールバックを登録する（複数登録可能）
  onMessage(cb: (msg: unknown) => void): void;
  // 接続切断時に呼ばれるコールバックを登録する
  onClose(cb: () => void): void;
  // 接続を明示的に閉じる（再接続ループも停止する）
  close(): void;
  // 現在の接続状態を返す
  readonly readyState: TransportReadyState;
}

// Transport ファクトリ関数の型（createXxxAdapter 関数の共通シグネチャ）
// URL と任意オプションを受け取り Transport を返す
export type TransportFactory = (url: string, options?: TransportOptions) => Transport;

// Transport 共通オプション
export interface TransportOptions {
  // WebSocket サブプロトコル（WebSocketAdapter のみ使用）
  readonly protocols?: string[];
  // 再接続の最大待機時間（ミリ秒、デフォルト 30000）
  readonly maxReconnectDelayMs?: number;
  // 再接続の初期待機時間（ミリ秒、デフォルト 500）
  readonly initialReconnectDelayMs?: number;
  // heartbeat 間隔（ミリ秒、デフォルト 30000）
  readonly heartbeatIntervalMs?: number;
  // Last-Event-ID（SSEAdapter の resume token）
  readonly lastEventId?: string;
}
