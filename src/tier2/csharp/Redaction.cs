// k1s0 tier2 PII フィールド redaction 実装（C# 版）
// tier2/CLAUDE.md §PII 含有フィールドは field_pii annotation + Outbox 書込時に redact
// Rust 実装（rust/src/redaction.rs）の 4 言語等価実装
using System;
using System.Collections.Generic;
using System.Text.Json;

namespace K1s0.Tier2
{
    // PII フィールドの redaction ロジックを提供する静的クラス
    public static class Redaction
    {
        // PII フィールドとして扱うキー一覧（field_pii annotation 対応）
        private static readonly HashSet<string> PiiFieldKeys = new(StringComparer.OrdinalIgnoreCase)
        {
            // 個人氏名フィールド
            "name",
            // メールアドレスフィールド
            "email",
            // 電話番号フィールド
            "phone",
            // 住所フィールド
            "address",
            // 生年月日フィールド
            "date_of_birth",
            // 社会保障番号フィールド
            "ssn",
        };

        // Redact: JSON 辞書から PII フィールドを除去した新しい辞書を返す
        // src: redact 対象の辞書（変更しない）
        public static Dictionary<string, object?> Redact(Dictionary<string, object?> src)
        {
            // 結果辞書を初期化する
            var result = new Dictionary<string, object?>(capacity: src.Count);
            // 各 key-value ペアを走査する
            foreach (var (key, value) in src)
            {
                // PII フィールドであれば除外する（redact）
                if (PiiFieldKeys.Contains(key))
                {
                    // PII フィールドは結果に含めない
                    continue;
                }
                // 非 PII フィールドはそのまま結果に追加する
                result[key] = value;
            }
            // redact 済み辞書を返す
            return result;
        }

        // RedactJson: JSON 文字列から PII フィールドを除去した新しい JSON 文字列を返す
        // json: redact 対象の JSON 文字列（変更しない）
        public static string RedactJson(string json)
        {
            // JSON をパースして辞書に変換する
            var dict = JsonSerializer.Deserialize<Dictionary<string, object?>>(json);
            // dict が null の場合は空 JSON を返す
            if (dict is null) return "{}";
            // PII フィールドを除去する
            var redacted = Redact(dict);
            // redact 済み辞書を JSON に変換して返す
            return JsonSerializer.Serialize(redacted);
        }
    }
}
