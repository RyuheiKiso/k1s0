namespace K1s0.System.Pagination;

public record PageRequest(uint Page, uint PerPage);

public record PaginationMeta(long Total, uint Page, uint PerPage, uint TotalPages);

public static class PerPageValidator
{
    public const uint MinPerPage = 1;
    public const uint MaxPerPage = 100;

    public static uint Validate(uint perPage)
    {
        if (perPage < MinPerPage || perPage > MaxPerPage)
        {
            throw new ArgumentOutOfRangeException(
                nameof(perPage),
                perPage,
                $"perPage must be between {MinPerPage} and {MaxPerPage}");
        }

        return perPage;
    }
}
