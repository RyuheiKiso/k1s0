// 承認の一意識別子。Guid をラップする値オブジェクト。

namespace K1s0.Tier2.ApprovalFlow.Domain.ValueObjects;

// ApprovalId は承認 1 件を識別する不変の値オブジェクト。
public readonly record struct ApprovalId(Guid Value)
{
    // 新規承認用の ID を生成する。
    public static ApprovalId NewId() => new(Guid.NewGuid());

    // 文字列から復元する。失敗時は FormatException。
    public static ApprovalId Parse(string s) => new(Guid.Parse(s));

    // 文字列表現は Guid の標準フォーマット（hyphenated）。
    public override string ToString() => Value.ToString();
}
