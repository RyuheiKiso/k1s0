namespace K1s0.System.Pagination;

public record PageResponse<T>(
    IReadOnlyList<T> Items,
    long Total,
    uint Page,
    uint PerPage,
    uint TotalPages)
{
    public static PageResponse<T> Create(IReadOnlyList<T> items, long total, PageRequest req)
    {
        var totalPages = req.PerPage == 0 ? 0u : (uint)((total + req.PerPage - 1) / req.PerPage);
        return new PageResponse<T>(items, total, req.Page, req.PerPage, totalPages);
    }
}
