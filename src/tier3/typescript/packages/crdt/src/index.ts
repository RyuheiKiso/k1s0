// k1s0 tier3 CRDT 型（tier3 内部閉込め）
// cross-cutting spec は不要。collaborative editing 用の基本 CRDT 型を提供する
// LWW-Register と G-Set の骨格を実装する

// LWW-Register（Last Write Wins Register）
// HLC タイムスタンプに基づいて最新値を保持する
export interface LWWRegister<T> {
  // 現在の値
  readonly value: T;
  // HLC タイムスタンプ（最後に書き込まれた時刻）
  readonly hlcTimestamp: string;
  // 書き込んだ actor の ID
  readonly actorId: string;
}

// LWW-Register のマージ（HLC タイムスタンプが大きい方を採用する）
export function mergeLWW<T>(a: LWWRegister<T>, b: LWWRegister<T>): LWWRegister<T> {
  // タイムスタンプが大きい方を返す（同値の場合は actorId の辞書順で解決する）
  if (a.hlcTimestamp > b.hlcTimestamp) return a;
  if (b.hlcTimestamp > a.hlcTimestamp) return b;
  // タイムスタンプが同一の場合は actorId 辞書順で resolve する
  return a.actorId >= b.actorId ? a : b;
}

// G-Set（Grow-only Set）
// 追加のみ可能な集合（削除不可）
export interface GSet<T> {
  // 集合の要素
  readonly elements: ReadonlySet<T>;
}

// G-Set の作成
export function createGSet<T>(initial?: readonly T[]): GSet<T> {
  // 初期要素から G-Set を作成する
  return { elements: new Set(initial ?? []) };
}

// G-Set へ要素を追加する（immutable）
export function addToGSet<T>(gset: GSet<T>, element: T): GSet<T> {
  // 新しい集合に要素を追加して返す
  const newElements = new Set(gset.elements);
  newElements.add(element);
  return { elements: newElements };
}

// G-Set のマージ（両集合の和集合を返す）
export function mergeGSet<T>(a: GSet<T>, b: GSet<T>): GSet<T> {
  // 両集合の和集合を返す（G-Set は削除不可のため union が正しい）
  const merged = new Set([...a.elements, ...b.elements]);
  return { elements: merged };
}

// VectorClock（concurrent edit presence 用）
export type VectorClock = Readonly<Record<string, number>>;

// VectorClock を増加させる（指定 actor の clock を +1 する）
export function incrementClock(clock: VectorClock, actorId: string): VectorClock {
  // 指定 actor の clock を increment する
  return { ...clock, [actorId]: (clock[actorId] ?? 0) + 1 };
}

// VectorClock のマージ（各 actor の最大値を採用する）
export function mergeClock(a: VectorClock, b: VectorClock): VectorClock {
  // 全 actor の max を取って merge する
  const allActors = new Set([...Object.keys(a), ...Object.keys(b)]);
  const merged: Record<string, number> = {};
  for (const actor of allActors) {
    merged[actor] = Math.max(a[actor] ?? 0, b[actor] ?? 0);
  }
  return merged;
}
