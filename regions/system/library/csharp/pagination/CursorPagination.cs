using System.Text;

namespace K1s0.System.Pagination;

public record CursorRequest(string? Cursor, uint Limit);

public record CursorMeta(string? NextCursor, bool HasMore);

public static class CursorPagination
{
    private const char Separator = '|';

    public static string Encode(string sortKey, string id) =>
        Convert.ToBase64String(Encoding.UTF8.GetBytes($"{sortKey}{Separator}{id}"));

    public static (string SortKey, string Id) Decode(string cursor)
    {
        var decoded = Encoding.UTF8.GetString(Convert.FromBase64String(cursor));
        var idx = decoded.IndexOf(Separator);
        if (idx < 0)
        {
            throw new FormatException("invalid cursor: missing separator");
        }

        return (decoded[..idx], decoded[(idx + 1)..]);
    }
}
