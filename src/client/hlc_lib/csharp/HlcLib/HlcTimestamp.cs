// HlcTimestamp.cs — k1s0-hlc: Hybrid Logical Clock (HLC) の C# タイムスタンプ型
// Kulkarni et al. (2014) "Logical Physical Clocks" のタイムスタンプ構造体を C# で実装する。
// wall-clock 禁止規律（src/CLAUDE.md §wall-clock TTL 禁止）に従い、
// TTL / deadline の比較は HLC elapsed で管理し、DateTimeOffset.UtcNow を直接参照しない。
// Rust 実装（src/client/hlc_lib/rust/src/lib.rs）と同等の API を提供する。

// K1s0.HlcLib 名前空間を宣言する
namespace K1s0.HlcLib;

// HlcTimestamp: HLC のタイムスタンプ（WallMs + Logical + NodeId の 3 tuple）
// readonly record struct で不変値型として定義する（Rust の Copy トレイトと同等）
// IComparable<HlcTimestamp> を実装して全順序比較（WallMs > Logical > NodeId）を提供する
public readonly record struct HlcTimestamp(ulong WallMs, ushort Logical, ushort NodeId)
    : IComparable<HlcTimestamp>
{
    // Epoch: WallMs=0, Logical=0, NodeId=0 の最小タイムスタンプ定数（未初期化判定に使用）
    public static readonly HlcTimestamp Epoch = new HlcTimestamp(0UL, 0, 0);

    // U16Max: ushort の最大値（論理カウンタのオーバーフロー検出に使用する）
    internal const ushort U16Max = ushort.MaxValue;

    // FormatCompact は HlcTimestamp を文字列に変換する
    // 形式: "{wall_ms_hex_16}-{logical_04x}-{node_04x}"
    // tier2 cache_layer の generate_hlc_timestamp が生成していた形式と互換性を保つ
    public string FormatCompact()
    {
        // 16 桁 hex + 4 桁 hex + 4 桁 hex の形式でフォーマットして返す
        return $"{WallMs:x16}-{Logical:x4}-{NodeId:x4}";
    }

    // ParseCompact は FormatCompact が出力した文字列を HlcTimestamp にパースする
    // パースに失敗した場合は null を返す（不正入力を呼び出し元で処理させる）
    public static HlcTimestamp? ParseCompact(string s)
    {
        // ハイフン区切りで分割する（最大 3 パート）
        var parts = s.Split('-');
        // パート数が正確に 3 でなければ null を返す（形式不正）
        if (parts.Length != 3)
        {
            return null;
        }
        // wall_ms パート: 16 桁 hex → ulong に変換する（パース失敗時は null）
        if (!ulong.TryParse(parts[0], System.Globalization.NumberStyles.HexNumber, null, out var wallMs))
        {
            return null;
        }
        // logical パート: 4 桁 hex → ushort に変換する（パース失敗時は null）
        if (!ushort.TryParse(parts[1], System.Globalization.NumberStyles.HexNumber, null, out var logical))
        {
            return null;
        }
        // node_id パート: 4 桁 hex → ushort に変換する（パース失敗時は null）
        if (!ushort.TryParse(parts[2], System.Globalization.NumberStyles.HexNumber, null, out var nodeId))
        {
            return null;
        }
        // 入力文字列の長さが期待値（26 文字）と一致することを確認する（余分な文字を拒否）
        if (s.Length != 26)
        {
            return null;
        }
        // パースに成功した場合は HlcTimestamp を返す
        return new HlcTimestamp(wallMs, logical, nodeId);
    }

    // AddMs は self に durationMs を加算した deadline 用 HlcTimestamp を返す
    // wall-clock の直接使用を禁止するため、deadline 表現はこの関数を経由する
    // overflow 時は ulong.MaxValue に飽和する（Rust の saturating_add と同等）
    public HlcTimestamp AddMs(ulong durationMs)
    {
        // WallMs に durationMs を加算する（checked でオーバーフロー検出）
        ulong newWall;
        // unchecked で ulong のラップアラウンドを許可し、オーバーフローを手動で検出する
        unchecked
        {
            // 加算を実行する
            newWall = WallMs + durationMs;
        }
        // 加算結果が元の WallMs より小さい場合はオーバーフローが発生している
        if (newWall < WallMs)
        {
            // ulong.MaxValue に飽和させる（Rust の saturating_add と同等）
            newWall = ulong.MaxValue;
        }
        // deadline の先頭イベントを表すため Logical を 0 にリセットする
        // NodeId は引き継ぐ（deadline の発行者を追跡する）
        return new HlcTimestamp(newWall, 0, NodeId);
    }

    // ElapsedMsSince は reference から self までの経過ミリ秒を返す
    // self が reference より前の場合は 0 を返す（負の elapsed は表現しない）
    // deadline との差分比較（expired 判定）に使用する
    public ulong ElapsedMsSince(HlcTimestamp reference)
    {
        // WallMs の差分を返す（self が reference 以前なら 0 に飽和する）
        if (WallMs <= reference.WallMs)
        {
            // self が reference 以前の場合は 0 を返す（飽和減算）
            return 0UL;
        }
        // self が reference より後の場合は差分を返す
        return WallMs - reference.WallMs;
    }

    // IsExpiredAt は deadline と比較して self が期限切れかどうかを返す
    // current: 現在の HLC タイムスタンプ（HlcClock.Now() で取得）
    // true = current が self（deadline）を超えた = 期限切れ
    public bool IsExpiredAt(HlcTimestamp current)
    {
        // current が self 以上（後または同時）なら期限切れ
        return CompareTo(current) <= 0;
    }

    // CompareTo は IComparable<HlcTimestamp> の実装
    // 戻り値: 負（self < other）/ 0（self == other）/ 正（self > other）
    // 比較順序: WallMs → Logical → NodeId（辞書順）
    public int CompareTo(HlcTimestamp other)
    {
        // WallMs を比較する
        var wallCmp = WallMs.CompareTo(other.WallMs);
        if (wallCmp != 0)
        {
            // WallMs が異なれば WallMs の比較結果を返す
            return wallCmp;
        }
        // WallMs が同一なら Logical を比較する
        var logicalCmp = Logical.CompareTo(other.Logical);
        if (logicalCmp != 0)
        {
            // Logical が異なれば Logical の比較結果を返す
            return logicalCmp;
        }
        // Logical も同一なら NodeId を比較する（完全全順序を保証する）
        return NodeId.CompareTo(other.NodeId);
    }
}
