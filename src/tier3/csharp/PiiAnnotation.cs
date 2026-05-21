// PiiAnnotation.cs — field_pii annotation の C# .NET 8+ 実装（Attribute + Reflection ベース）
// [FieldPii] attribute を PII を含むプロパティに付与し、
// PiiAnnotationExtensions.GetPiiFields<T>() で compile-time に PII フィールドを列挙する
// Y-tier3-4: field_pii annotation compile-time 強制（4 言語等価強度の C# 実装）

namespace K1s0.Tier3.State;

// ============================================================
// FieldPiiAttribute — PII フィールドを宣言する属性
// ============================================================

// FieldPiiAttribute は PII を含むプロパティに付与する属性
// AttributeUsage: プロパティのみに適用可能（AllowMultiple: false / Inherited: true）
[AttributeUsage(AttributeTargets.Property, AllowMultiple = false, Inherited = true)]
public sealed class FieldPiiAttribute : Attribute
{
    // FieldPiiAttribute の既定コンストラクタ（属性引数なし）
    // 将来的には PiiCategory（email / phone / address 等）を引数に取ることも検討できる
    public FieldPiiAttribute()
    {
        // 引数なしの属性（マーカーとしてのみ使用する）
    }

    // Category は PII のカテゴリを表す文字列プロパティ（optional: デフォルトは "general"）
    // 将来の拡張のために予約する（現バージョンでは使用しない）
    public string Category { get; init; } = "general";
}

// ============================================================
// PiiAnnotationExtensions — Reflection ベースの PII フィールド取得拡張メソッド
// ============================================================

// PiiAnnotationExtensions は [FieldPii] 属性を持つプロパティ名を Reflection で取得する拡張クラス
// static クラスとして定義し、拡張メソッドとして GetPiiFields<T>() を提供する
public static class PiiAnnotationExtensions
{
    // GetPiiFields<T> は型引数 T のプロパティから [FieldPii] を持つプロパティ名を返す
    // Reflection で typeof(T).GetProperties() を走査して [FieldPii] attribute を検出する
    // 返値: [FieldPii] 属性を持つプロパティ名の IEnumerable<string>（順序は定義順）
    public static IEnumerable<string> GetPiiFields<T>()
    {
        // typeof(T) から全パブリックインスタンスプロパティを取得する
        return typeof(T)
            // パブリックかつインスタンスプロパティのみ対象にする
            .GetProperties(System.Reflection.BindingFlags.Public | System.Reflection.BindingFlags.Instance)
            // [FieldPii] 属性が付与されているプロパティのみ絞り込む
            .Where(p => p.GetCustomAttributes(typeof(FieldPiiAttribute), inherit: true).Length > 0)
            // プロパティ名を取得する
            .Select(p => p.Name);
    }

    // GetPiiFields は非ジェネリック版の拡張メソッド（実行時型で動作する場合に使用する）
    // targetType: [FieldPii] を持つプロパティを保有する型（System.Type）
    // 返値: [FieldPii] 属性を持つプロパティ名の IEnumerable<string>
    public static IEnumerable<string> GetPiiFields(System.Type targetType)
    {
        // targetType が null の場合は空を返す（安全なフォールバック）
        if (targetType is null)
        {
            // null 型は空の列挙を返す
            return [];
        }
        // パブリックインスタンスプロパティを走査して [FieldPii] を持つプロパティ名を返す
        return targetType
            // パブリックかつインスタンスプロパティのみ対象にする
            .GetProperties(System.Reflection.BindingFlags.Public | System.Reflection.BindingFlags.Instance)
            // [FieldPii] 属性が付与されているプロパティのみ絞り込む
            .Where(p => p.GetCustomAttributes(typeof(FieldPiiAttribute), inherit: true).Length > 0)
            // プロパティ名を取得する
            .Select(p => p.Name);
    }

    // HasPiiFields<T> は型 T が [FieldPii] を持つプロパティを 1 件以上保有するかを返す
    // GetPiiFields<T> の convenience wrapper（存在確認のみが目的の場合に使用する）
    public static bool HasPiiFields<T>()
    {
        // GetPiiFields<T>() が 1 件以上返すかで判定する
        return GetPiiFields<T>().Any();
    }

    // ValidatePiiStripped<T> は payload Dictionary から [FieldPii] フィールドが strip 済みかを検証する
    // [FieldPii] で宣言されたプロパティ名が payload に含まれていないことを確認する
    // payload: PII strip 済みのはずの送信データ（string → object? の Dictionary）
    // 返値: strip 漏れの PII フィールド名のリスト（空リスト = strip 完了）
    public static IReadOnlyList<string> ValidatePiiStripped<T>(
        // PII strip 済みのはずの送信データ
        IDictionary<string, object?> payload
    )
    {
        // [FieldPii] フィールド名のセットを取得する
        var piiFieldSet = new HashSet<string>(GetPiiFields<T>(), StringComparer.Ordinal);
        // payload に残存する PII フィールドを収集する
        return payload.Keys
            // payload キーが PII フィールドセットに含まれるものを絞り込む
            .Where(k => piiFieldSet.Contains(k))
            // IReadOnlyList<string> に変換する
            .ToList()
            .AsReadOnly();
    }
}

// ============================================================
// 使用例（参照実装: proc_macro / source generator が将来自動生成するパターン）
// ============================================================
// 以下は [FieldPii] の正しい使用例（実際のドメインモデルは別ファイルに定義すること）:
//
//   public sealed class PersonalInfo
//   {
//       [FieldPii]
//       public string Email { get; init; } = string.Empty;
//
//       [FieldPii]
//       public string Name { get; init; } = string.Empty;
//
//       public string TenantId { get; init; } = string.Empty;  // PII でないフィールド
//   }
//
//   // PII フィールドを取得する:
//   IEnumerable<string> piiFields = PiiAnnotationExtensions.GetPiiFields<PersonalInfo>();
//   // => ["Email", "Name"]
//
//   // payload に PII フィールドが残存しないか検証する:
//   var payload = new Dictionary<string, object?> { ["TenantId"] = "t-001" };
//   IReadOnlyList<string> leaks = PiiAnnotationExtensions.ValidatePiiStripped<PersonalInfo>(payload);
//   // => [] (空リスト = strip 完了)
// ============================================================
