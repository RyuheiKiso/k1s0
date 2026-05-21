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
        // Rust 実装（rust/src/redaction.rs）の PII_FIELDS と同一 10 キーを維持する
        private static readonly HashSet<string> PiiFieldKeys = new(StringComparer.OrdinalIgnoreCase)
        {
            // メールアドレスフィールド
            "email",
            // 電話番号フィールド
            "phone",
            // 住所フィールド
            "address",
            // 個人氏名フィールド
            "name",
            // マイナンバー / 税 ID フィールド
            "tax_id",
            // 生年月日フィールド（birth_date に統一）
            "birth_date",
            // クレジットカード番号フィールド
            "card_number",
            // 社会保障番号フィールド
            "ssn",
            // パスポート番号フィールド
            "passport_number",
            // 銀行口座番号フィールド
            "bank_account",
        };

        // RedactNode: JsonNode を再帰的に走査して PII フィールドを *** に置換する
        // node: redact 対象の JsonNode（in-place 変更を行う）
        private static System.Text.Json.Nodes.JsonNode? RedactNode(System.Text.Json.Nodes.JsonNode? node)
        {
            // node が null の場合はそのまま返す
            if (node is null) return null;
            // Object の場合は各フィールドを走査して PII キーを *** に置換する
            if (node is System.Text.Json.Nodes.JsonObject obj)
            {
                // PII フィールド名の一覧を先に収集してから置換する（走査中に変更を避けるため）
                var keys = new System.Collections.Generic.List<string>();
                // obj の全キーを収集する
                foreach (var kv in obj)
                {
                    // キーをリストに追加する
                    keys.Add(kv.Key);
                }
                // 各キーに対して処理を行う
                foreach (var key in keys)
                {
                    // PII フィールドであれば *** 文字列に置換する（キーは保持する）
                    if (PiiFieldKeys.Contains(key))
                    {
                        // キーを保持しつつ値を *** に上書きする
                        obj[key] = System.Text.Json.Nodes.JsonValue.Create("***");
                    }
                    else
                    {
                        // 非 PII フィールドは再帰的に走査する
                        obj[key] = RedactNode(obj[key]);
                    }
                }
                // redact 済みの Object を返す
                return obj;
            }
            // Array の場合は各要素を再帰的に処理する
            if (node is System.Text.Json.Nodes.JsonArray arr)
            {
                // 配列の各インデックスを走査する
                for (var i = 0; i < arr.Count; i++)
                {
                    // 各要素を再帰的に redact する
                    arr[i] = RedactNode(arr[i]);
                }
                // redact 済みの Array を返す
                return arr;
            }
            // String / Number / Bool / Null はそのまま返す
            return node;
        }

        // Redact: JSON 辞書から PII フィールドを *** に置換した新しい辞書を返す
        // src: redact 対象の辞書（変更しない）
        public static Dictionary<string, object?> Redact(Dictionary<string, object?> src)
        {
            // 結果辞書を初期化する
            var result = new Dictionary<string, object?>(capacity: src.Count);
            // 各 key-value ペアを走査する
            foreach (var (key, value) in src)
            {
                // PII フィールドであれば *** 文字列に置換する（キーは保持する）
                if (PiiFieldKeys.Contains(key))
                {
                    // キーを保持しつつ値を *** に置換する
                    result[key] = "***";
                }
                else
                {
                    // 非 PII フィールドはそのまま結果に追加する
                    result[key] = value;
                }
            }
            // redact 済み辞書を返す
            return result;
        }

        // RedactJson: JSON 文字列から PII フィールドを *** に置換した新しい JSON 文字列を返す
        // json: redact 対象の JSON 文字列（変更しない）
        public static string RedactJson(string json)
        {
            // JSON を JsonNode としてパースして mutable に操作する
            var node = System.Text.Json.Nodes.JsonNode.Parse(json);
            // node が null の場合は空 JSON を返す
            if (node is null) return "{}";
            // 再帰的に PII フィールドを *** に置換する
            var redacted = RedactNode(node);
            // redact 済みノードを JSON 文字列に変換して返す
            return redacted?.ToJsonString() ?? "{}";
        }
    }
}
