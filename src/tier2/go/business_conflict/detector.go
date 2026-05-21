// k1s0 tier2 business_conflict detector — Go 実装
// 10_テナント分離適合仕様.md §business_conflict_subtypes に基づく 4 subtype 検出ロジック
// src/tier2/business_conflict/subtypes.yaml の detector_rule を Go で物理化する
// Rust 実装（business_conflict.rs）と 4 言語等価強度を持つ

// パッケージ名: business_conflict
package business_conflict

// ConflictSubtype: 4 種の business conflict サブタイプを表す型エイリアス
// subtypes.yaml の 4 エントリ（stale_write / lost_update / supersede / concurrent_edit）と 1:1 対応する
type ConflictSubtype string

const (
	// ConflictSubtypeStaleWrite: 書き込み時点でデータが陳腐化していた（楽観ロック失敗）
	// detector_rule: last_seen_version != current_version（subtypes.yaml 参照）
	ConflictSubtypeStaleWrite ConflictSubtype = "stale_write"
	// ConflictSubtypeLostUpdate: 並行書き込みにより更新が消失した（フィールド差分重複）
	// detector_rule: concurrent_write_detected && field_overlap（subtypes.yaml 参照）
	ConflictSubtypeLostUpdate ConflictSubtype = "lost_update"
	// ConflictSubtypeSupersede: 後発の書き込みが先発を上書きした（意図的な上書き）
	// detector_rule: supersede_flag == true（subtypes.yaml 参照）
	ConflictSubtypeSupersede ConflictSubtype = "supersede"
	// ConflictSubtypeConcurrentEdit: 同一フィールドへの同時編集が検出された（プレゼンス情報ベース）
	// detector_rule: field_intersection_detected && concurrent_writers >= 2（subtypes.yaml 参照）
	ConflictSubtypeConcurrentEdit ConflictSubtype = "concurrent_edit"
)

// Detect: 4 subtype の detector_rule を評価して conflict subtype を返す
// Rust 実装の ConflictDetector::detect と意味的に等価な Go 版
//
// 引数:
//
//	lastSeenVersion   — クライアントが送信した base version（楽観ロックの基準点）
//	currentVersion    — サーバー側の現在 version（DB から取得した最新値）
//	fieldDiff         — クライアントが変更したフィールド名のスライス（空 = 変更なし）
//	concurrentWriters — presence 情報から取得した並行ライター数
//	supersedeFlag     — 同一アクターによる意図的な上書きフラグ
//
// 戻り値: conflict が確定した場合は (ConflictSubtype, true)、conflict がない場合は ("", false)
//
// 評価順序: supersede → concurrent_edit → lost_update → stale_write
// supersede は他の全 subtype より高い優先度を持つため最初に評価する
func Detect(
	lastSeenVersion int64,
	currentVersion int64,
	fieldDiff []string,
	concurrentWriters int,
	supersedeFlag bool,
) (ConflictSubtype, bool) {
	// P1: supersede_flag が true の場合は Supersede を最優先で返す
	// detector_rule: supersede_flag == true（subtypes.yaml 参照）
	if supersedeFlag {
		// 意図的な上書き: Supersede subtype を返す
		return ConflictSubtypeSupersede, true
	}

	// P2: 並行ライターが 2 人以上 かつ フィールド差分が非空の場合は ConcurrentEdit を返す
	// detector_rule: field_intersection_detected && concurrent_writers >= 2（subtypes.yaml 参照）
	// field_diff が非空 = クライアントが少なくとも 1 フィールドを変更している（intersection 有りとみなす）
	if concurrentWriters >= 2 && len(fieldDiff) > 0 {
		// 同時編集検出: ConcurrentEdit subtype を返す
		return ConflictSubtypeConcurrentEdit, true
	}

	// P3: version 不一致 かつ concurrent_writers >= 1 かつ フィールド差分が非空の場合は LostUpdate を返す
	// detector_rule: concurrent_write_detected && field_overlap（subtypes.yaml 参照）
	// concurrent_write_detected = lastSeenVersion != currentVersion かつ concurrentWriters >= 1
	if lastSeenVersion != currentVersion && concurrentWriters >= 1 && len(fieldDiff) > 0 {
		// 更新消失: LostUpdate subtype を返す
		return ConflictSubtypeLostUpdate, true
	}

	// P4: version のみ不一致の場合は StaleWrite を返す（最低優先度）
	// detector_rule: last_seen_version != current_version（subtypes.yaml 参照）
	if lastSeenVersion != currentVersion {
		// 陳腐化書込: StaleWrite subtype を返す
		return ConflictSubtypeStaleWrite, true
	}

	// 上記のいずれにも該当しない場合は conflict なし（空文字列と false を返す）
	return "", false
}
