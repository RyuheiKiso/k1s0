// concurrency_guard.ts — aggregate_id ごとに max_one_in_flight の Semaphore を実装する
// spec 11 §per-aggregate write 並行 1 件以下: reducer OL set 時に enforce する
// layers.yaml invariant の max_one_in_flight_per_aggregate を物理化する

// aggregate_id ごとの in-flight Promise を管理するマップ
const inFlightMap: Map<string, Promise<void>> = new Map();

// aggregate の in-flight を制御する関数
// aggregateId: 対象 aggregate の識別子
// fn: aggregate に対して排他的に実行する非同期処理
export async function withAggregateExclusivity<T>(
  // aggregate の識別子を受け取る
  aggregateId: string,
  // 排他的に実行する非同期処理を受け取る
  fn: () => Promise<T>
): Promise<T> {
  // 現在 in-flight の Promise を取得する
  const current = inFlightMap.get(aggregateId);
  // 現在 in-flight の処理が存在する場合は完了を待つ
  if (current !== undefined) {
    // 先行処理の完了を待機する（エラーは無視して進む）
    await current.catch(() => undefined);
  }
  // 自分の処理を Promise として定義する
  let resolve!: () => void;
  // 排他制御用の Promise を作成する
  const token = new Promise<void>((r) => { resolve = r; });
  // in-flight マップに登録する
  inFlightMap.set(aggregateId, token);
  // 処理を実行する
  try {
    // 排他的な処理を実行して結果を返す
    return await fn();
  } finally {
    // 処理が完了したら in-flight マップから削除する
    if (inFlightMap.get(aggregateId) === token) {
      inFlightMap.delete(aggregateId);
    }
    // 次の待機者を解放する
    resolve();
  }
}

// aggregate が現在 in-flight かどうかを確認する関数
export function isAggregateInFlight(aggregateId: string): boolean {
  // in-flight マップに aggregateId が存在するか確認する
  return inFlightMap.has(aggregateId);
}
