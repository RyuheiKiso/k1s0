namespace K1s0.Tracing;

public record TraceContext(string TraceId, string ParentId, byte Flags)
{
    public string ToTraceparent() => $"00-{TraceId}-{ParentId}-{Flags:x2}";

    public static TraceContext? FromTraceparent(string s)
    {
        var parts = s.Split('-');
        if (parts.Length != 4)
        {
            return null;
        }

        if (parts[0] != "00")
        {
            return null;
        }

        if (parts[1].Length != 32)
        {
            return null;
        }

        if (parts[2].Length != 16)
        {
            return null;
        }

        if (parts[3].Length != 2)
        {
            return null;
        }

        if (!byte.TryParse(parts[3], System.Globalization.NumberStyles.HexNumber, null, out var flags))
        {
            return null;
        }

        return new TraceContext(parts[1], parts[2], flags);
    }
}
