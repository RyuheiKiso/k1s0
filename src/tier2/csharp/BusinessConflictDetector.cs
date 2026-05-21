// k1s0 tier2 business_conflict detector — C# (.NET 8+) 実装
// 10_テナント分離適合仕様.md §business_conflict_subtypes に基づく 4 subtype 検出ロジック
// src/tier2/business_conflict/subtypes.yaml の detector_rule を C# で物理化する
// Rust 実装（business_conflict.rs）と 4 言語等価強度を持つ

// 基本型（string / IEnumerable 等）
using System;
// コレクション列挙インターフェース
using System.Collections.Generic;
// LINQ を使ってコレクション操作を行う
using System.Linq;

// k1s0 tier2 名前空間
namespace K1s0.Tier2;

/// <summary>
/// ConflictSubtype: 4 種の business conflict サブタイプを表す列挙型
/// subtypes.yaml の 4 エントリ（stale_write / lost_update / supersede / concurrent_edit）と 1:1 対応する
/// </summary>
public enum ConflictSubtype
{
    // StaleWrite: 書き込み時点でデータが陳腐化していた（楽観ロック失敗）
    // detector_rule: last_seen_version != current_version
    StaleWrite,
    // LostUpdate: 並行書き込みにより更新が消失した（フィールド差分重複）
    // detector_rule: concurrent_write_detected && field_overlap
    LostUpdate,
    // Supersede: 後発の書き込みが先発を上書きした（意図的な上書き）
    // detector_rule: supersede_flag == true
    Supersede,
    // ConcurrentEdit: 同一フィールドへの同時編集が検出された（プレゼンス情報ベース）
    // detector_rule: field_intersection_detected && concurrent_writers >= 2
    ConcurrentEdit,
}

/// <summary>
/// BusinessConflictDetector: 4 subtype の detector_rule を評価するエンジン
/// Rust 実装の ConflictDetector と意味的に等価な C# 版
/// </summary>
public static class BusinessConflictDetector
{
    /// <summary>
    /// Detect: 4 subtype の detector_rule を評価して conflict subtype を返す
    /// Rust 実装の ConflictDetector::detect と意味的に等価なメソッド
    ///
    /// 評価順序: supersede → concurrent_edit → lost_update → stale_write
    /// supersede は他の全 subtype より高い優先度を持つため最初に評価する
    /// </summary>
    /// <param name="lastSeenVersion">クライアントが送信した base version（楽観ロックの基準点）</param>
    /// <param name="currentVersion">サーバー側の現在 version（DB から取得した最新値）</param>
    /// <param name="fieldDiff">クライアントが変更したフィールド名のコレクション（空 = 変更なし）</param>
    /// <param name="concurrentWriters">presence 情報から取得した並行ライター数</param>
    /// <param name="supersedeFlag">同一アクターによる意図的な上書きフラグ</param>
    /// <returns>conflict が確定した場合は ConflictSubtype?、conflict がない場合は null</returns>
    public static ConflictSubtype? Detect(
        // クライアントが送信した base version（楽観ロックの基準点）
        long lastSeenVersion,
        // サーバー側の現在 version（DB から取得した最新値）
        long currentVersion,
        // クライアントが変更したフィールド名のコレクション（null 不可）
        IEnumerable<string> fieldDiff,
        // presence 情報から取得した並行ライター数（0 以上の整数）
        int concurrentWriters,
        // 同一アクターによる意図的な上書きフラグ
        bool supersedeFlag)
    {
        // fieldDiff が null の場合は ArgumentNullException をスローする
        ArgumentNullException.ThrowIfNull(fieldDiff);

        // フィールド差分をリストに変換して重複評価を避ける
        var diffList = fieldDiff.ToList();

        // P1: supersede_flag が true の場合は Supersede を最優先で返す
        // detector_rule: supersede_flag == true（subtypes.yaml 参照）
        if (supersedeFlag)
        {
            // 意図的な上書き: Supersede subtype を返す
            return ConflictSubtype.Supersede;
        }

        // P2: 並行ライターが 2 人以上 かつ フィールド差分が非空の場合は ConcurrentEdit を返す
        // detector_rule: field_intersection_detected && concurrent_writers >= 2（subtypes.yaml 参照）
        // diffList が非空 = クライアントが少なくとも 1 フィールドを変更している（intersection 有りとみなす）
        if (concurrentWriters >= 2 && diffList.Count > 0)
        {
            // 同時編集検出: ConcurrentEdit subtype を返す
            return ConflictSubtype.ConcurrentEdit;
        }

        // P3: version 不一致 かつ concurrent_writers >= 1 かつ フィールド差分が非空の場合は LostUpdate を返す
        // detector_rule: concurrent_write_detected && field_overlap（subtypes.yaml 参照）
        // concurrent_write_detected = lastSeenVersion != currentVersion かつ concurrentWriters >= 1
        if (lastSeenVersion != currentVersion && concurrentWriters >= 1 && diffList.Count > 0)
        {
            // 更新消失: LostUpdate subtype を返す
            return ConflictSubtype.LostUpdate;
        }

        // P4: version のみ不一致の場合は StaleWrite を返す（最低優先度）
        // detector_rule: last_seen_version != current_version（subtypes.yaml 参照）
        if (lastSeenVersion != currentVersion)
        {
            // 陳腐化書込: StaleWrite subtype を返す
            return ConflictSubtype.StaleWrite;
        }

        // 上記のいずれにも該当しない場合は conflict なし（null を返す）
        return null;
    }
}
