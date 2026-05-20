// presence.ts — presence client（30s TTL + 15s heartbeat）
// spec 11 §presence indicator: アクターの online 状態を 30s TTL + heartbeat で管理する
// 05_リアルタイム更新UX.md §presence indicator の物理実装

// アクターの presence エントリを表す型
export interface PresenceEntry {
  // アクターの識別子
  actorId: string;
  // presence の有効期限（UNIX ミリ秒）
  expiresAtMs: number;
}

// PresenceClient は presence state を管理するクラス
export class PresenceClient {
  // アクター ID → 有効期限のマップ（server-side state）
  private readonly entries: Map<string, number> = new Map();
  // TTL の長さ（ミリ秒）: 30 秒
  private readonly ttlMs: number = 30_000;
  // heartbeat インターバルの ID
  private heartbeatTimer: ReturnType<typeof setInterval> | null = null;
  // 自分自身のアクター ID
  private readonly selfActorId: string;

  // コンストラクタ: 自分自身のアクター ID を受け取る
  constructor(actorId: string) {
    // 自分自身のアクター ID を設定する
    this.selfActorId = actorId;
  }

  // heartbeat を開始する（15 秒ごとに自分の presence を更新する）
  startHeartbeat(): void {
    // 既に heartbeat が動いている場合は停止する
    if (this.heartbeatTimer !== null) {
      clearInterval(this.heartbeatTimer);
    }
    // 即座に自分の presence を登録する
    this.updatePresence(this.selfActorId);
    // 15 秒ごとに heartbeat を送信する
    this.heartbeatTimer = setInterval(() => {
      // 自分の presence を更新する
      this.updatePresence(this.selfActorId);
    }, 15_000);
  }

  // heartbeat を停止する
  stopHeartbeat(): void {
    // heartbeat タイマーが存在する場合は停止する
    if (this.heartbeatTimer !== null) {
      clearInterval(this.heartbeatTimer);
      // タイマー ID をリセットする
      this.heartbeatTimer = null;
    }
  }

  // 指定したアクターの presence を更新する（TTL を 30 秒延長する）
  updatePresence(actorId: string): void {
    // 現在時刻 + TTL を有効期限として設定する
    this.entries.set(actorId, Date.now() + this.ttlMs);
  }

  // 現在 online のアクター一覧を返す（TTL 切れを除外する）
  getActiveActors(now: number = Date.now()): PresenceEntry[] {
    // TTL 内のエントリのみをフィルタして返す
    return Array.from(this.entries.entries())
      .filter(([, expiresAtMs]) => expiresAtMs > now)
      .map(([actorId, expiresAtMs]) => ({ actorId, expiresAtMs }));
  }
}

// PresenceTracker は PresenceClient の canonical 別名（C-7 public API 統一）
// spec 11 §presence indicator での呼称が PresenceTracker のため alias を提供する
export { PresenceClient as PresenceTracker };
