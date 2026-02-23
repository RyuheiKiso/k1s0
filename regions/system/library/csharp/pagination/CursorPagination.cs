using System.Text;

namespace K1s0.System.Pagination;

public static class CursorPagination
{
    public static string Encode(string id) =>
        Convert.ToBase64String(Encoding.UTF8.GetBytes(id));

    public static string Decode(string cursor) =>
        Encoding.UTF8.GetString(Convert.FromBase64String(cursor));
}
