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
            throw new ValidationException("email", $"Invalid email: {email}");
        }
    }

    public void ValidateUuid(string id)
    {
        if (!Guid.TryParse(id, out _))
        {
            throw new ValidationException("id", $"Invalid UUID: {id}");
        }
    }

    public void ValidateUrl(string url)
    {
        if (!Uri.TryCreate(url, UriKind.Absolute, out var uri) ||
            (uri.Scheme != "http" && uri.Scheme != "https"))
        {
            throw new ValidationException("url", $"Invalid URL: {url}");
        }
    }

    public void ValidateTenantId(string tenantId)
    {
        if (string.IsNullOrWhiteSpace(tenantId) || !TenantIdRegex().IsMatch(tenantId))
        {
            throw new ValidationException("tenantId", $"Invalid tenant ID: {tenantId}");
        }
    }
}
