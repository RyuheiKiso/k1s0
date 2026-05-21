/**
 * k1s0 tier2 business_conflict detector — TypeScript 実装
 * 10_テナント分離適合仕様.md §business_conflict_subtypes に基づく 4 subtype 検出ロジック
 * src/tier2/business_conflict/subtypes.yaml の detector_rule を TypeScript で物理化する
 * Rust 実装（business_conflict.rs）と 4 言語等価強度を持つ
 */

// ConflictSubtype: 4 種の business conflict サブタイプを表す union 型
// subtypes.yaml の 4 エントリ（stale_write / lost_update / supersede / concurrent_edit）と 1:1 対応する
export type ConflictSubtype =
  // stale_write: 書き込み時点でデータが陳腐化していた（楽観ロック失敗）
  // detector_rule: last_seen_version != current_version（subtypes.yaml 参照）
  | "stale_write"
  // lost_update: 並行書き込みにより更新が消失した（フィールド差分重複）
  // detector_rule: concurrent_write_detected && field_overlap（subtypes.yaml 参照）
  | "lost_update"
  // supersede: 後発の書き込みが先発を上書きした（意図的な上書き）
  // detector_rule: supersede_flag == true（subtypes.yaml 参照）
  | "supersede"
  // concurrent_edit: 同一フィールドへの同時編集が検出された（プレゼンス情報ベース）
  // detector_rule: field_intersection_detected && concurrent_writers >= 2（subtypes.yaml 参照）
  | "concurrent_edit";

/**
 * detectConflict: 4 subtype の detector_rule を評価して conflict subtype を返す
 * Rust 実装の ConflictDetector::detect と意味的に等価な TypeScript 版
 *
 * 評価順序: supersede → concurrent_edit → lost_update → stale_write
 * supersede は他の全 subtype より高い優先度を持つため最初に評価する
 *
 * @param lastSeenVersion  クライアントが送信した base version（楽観ロックの基準点）
 * @param currentVersion   サーバー側の現在 version（DB から取得した最新値）
 * @param fieldDiff        クライアントが変更したフィールド名の配列（空配列 = 変更なし）
 * @param concurrentWriters presence 情報から取得した並行ライター数
 * @param supersedeFlag    同一アクターによる意図的な上書きフラグ
 * @returns conflict が確定した場合は ConflictSubtype、conflict がない場合は null
 */
// detectConflict: detector_rule を評価して conflict subtype を返す関数
export function detectConflict(
  // クライアントが送信した base version（楽観ロックの基準点）
  lastSeenVersion: number,
  // サーバー側の現在 version（DB から取得した最新値）
  currentVersion: number,
  // クライアントが変更したフィールド名の配列（空配列 = 変更なし）
  fieldDiff: readonly string[],
  // presence 情報から取得した並行ライター数（0 以上の整数）
  concurrentWriters: number,
  // 同一アクターによる意図的な上書きフラグ
  supersedeFlag: boolean,
): ConflictSubtype | null {
  // P1: supersede_flag が true の場合は Supersede を最優先で返す
  // detector_rule: supersede_flag == true（subtypes.yaml 参照）
  if (supersedeFlag) {
    // 意図的な上書き: "supersede" を返す
    return "supersede";
  }

  // P2: 並行ライターが 2 人以上 かつ フィールド差分が非空の場合は ConcurrentEdit を返す
  // detector_rule: field_intersection_detected && concurrent_writers >= 2（subtypes.yaml 参照）
  // fieldDiff が非空 = クライアントが少なくとも 1 フィールドを変更している（intersection 有りとみなす）
  if (concurrentWriters >= 2 && fieldDiff.length > 0) {
    // 同時編集検出: "concurrent_edit" を返す
    return "concurrent_edit";
  }

  // P3: version 不一致 かつ concurrent_writers >= 1 かつ フィールド差分が非空の場合は LostUpdate を返す
  // detector_rule: concurrent_write_detected && field_overlap（subtypes.yaml 参照）
  // concurrent_write_detected = lastSeenVersion !== currentVersion かつ concurrentWriters >= 1
  if (lastSeenVersion !== currentVersion && concurrentWriters >= 1 && fieldDiff.length > 0) {
    // 更新消失: "lost_update" を返す
    return "lost_update";
  }

  // P4: version のみ不一致の場合は StaleWrite を返す（最低優先度）
  // detector_rule: last_seen_version != current_version（subtypes.yaml 参照）
  if (lastSeenVersion !== currentVersion) {
    // 陳腐化書込: "stale_write" を返す
    return "stale_write";
  }

  // 上記のいずれにも該当しない場合は conflict なし（null を返す）
  return null;
}
