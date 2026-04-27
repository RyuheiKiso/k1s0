// 承認状態の値オブジェクト（enum 風、未知値の早期拒否）。

namespace K1s0.Tier2.ApprovalFlow.Domain.ValueObjects;

// ApprovalStatus は 4 つの固定値しか取らない。
public readonly record struct ApprovalStatus
{
    // 内部値。
    private readonly string _value;

    // private コンストラクタ。
    private ApprovalStatus(string value) => _value = value;

    // 既知の状態定数。
    public static readonly ApprovalStatus Pending = new("PENDING");
    public static readonly ApprovalStatus Approved = new("APPROVED");
    public static readonly ApprovalStatus Rejected = new("REJECTED");
    public static readonly ApprovalStatus Cancelled = new("CANCELLED");

    // 全 4 状態の集合。
    private static readonly IReadOnlyDictionary<string, ApprovalStatus> Known = new Dictionary<string, ApprovalStatus>(StringComparer.OrdinalIgnoreCase)
    {
        [Pending._value] = Pending,
        [Approved._value] = Approved,
        [Rejected._value] = Rejected,
        [Cancelled._value] = Cancelled,
    };

    // 文字列から ApprovalStatus を生成する。未知値は ArgumentException。
    public static ApprovalStatus Parse(string s)
    {
        // 前後空白除去。
        var trimmed = s?.Trim();
        // 空 / null は不正。
        if (string.IsNullOrEmpty(trimmed))
        {
            throw new ArgumentException("ApprovalStatus must not be empty", nameof(s));
        }
        // 既知集合を引く。
        if (Known.TryGetValue(trimmed!, out var status))
        {
            return status;
        }
        // 未知値は不正。
        throw new ArgumentException($"Unknown ApprovalStatus: {s}", nameof(s));
    }

    // 文字列表現を返す（PENDING / APPROVED / REJECTED / CANCELLED）。
    public override string ToString() => _value;
}
