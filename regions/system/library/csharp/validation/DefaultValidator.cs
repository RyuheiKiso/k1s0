using System.Text.RegularExpressions;

namespace K1s0.System.Validation;

public partial class DefaultValidator : IValidator
{
    [GeneratedRegex(@"^[^@\s]+@[^@\s]+\.[^@\s]+$")]
    private static partial Regex EmailRegex();

    [GeneratedRegex(@"^[a-z][a-z0-9_-]{1,62}$")]
    private static partial Regex TenantIdRegex();

    public void ValidateEmail(string email)
    {
        if (string.IsNullOrWhiteSpace(email) || !EmailRegex().IsMatch(email))
        {
            throw new ValidationException("email", $"Invalid email: {email}", "INVALID_EMAIL");
        }
    }

    public void ValidateUuid(string id)
    {
        if (!Guid.TryParse(id, out _))
        {
            throw new ValidationException("id", $"Invalid UUID: {id}", "INVALID_UUID");
        }
    }

    public void ValidateUrl(string url)
    {
        if (!Uri.TryCreate(url, UriKind.Absolute, out var uri) ||
            (uri.Scheme != "http" && uri.Scheme != "https"))
        {
            throw new ValidationException("url", $"Invalid URL: {url}", "INVALID_URL");
        }
    }

    public void ValidateTenantId(string tenantId)
    {
        if (string.IsNullOrWhiteSpace(tenantId) || !TenantIdRegex().IsMatch(tenantId))
        {
            throw new ValidationException("tenantId", $"Invalid tenant ID: {tenantId}", "INVALID_TENANT_ID");
        }
    }

    public void ValidatePagination(int page, int perPage)
    {
        if (page < 1)
        {
            throw new ValidationException("page", $"Page must be >= 1, got {page}", "INVALID_PAGE");
        }
        if (perPage < 1 || perPage > 100)
        {
            throw new ValidationException("perPage", $"PerPage must be 1-100, got {perPage}", "INVALID_PER_PAGE");
        }
    }

    public void ValidateDateRange(DateTime startDate, DateTime endDate)
    {
        if (startDate > endDate)
        {
            throw new ValidationException(
                "dateRange",
                $"Start date ({startDate:O}) must be <= end date ({endDate:O})",
                "INVALID_DATE_RANGE");
        }
    }
}
