using K1s0.System.Validation;

namespace K1s0.System.Validation.Tests;

public class DefaultValidatorTests
{
    private readonly DefaultValidator _validator = new();

    [Theory]
    [InlineData("user@example.com")]
    [InlineData("test+tag@domain.co.jp")]
    public void ValidateEmail_ValidInput_DoesNotThrow(string email)
    {
        _validator.ValidateEmail(email);
    }

    [Theory]
    [InlineData("")]
    [InlineData("not-an-email")]
    [InlineData("@missing-local.com")]
    public void ValidateEmail_InvalidInput_ThrowsValidationException(string email)
    {
        var ex = Assert.Throws<ValidationException>(() => _validator.ValidateEmail(email));
        Assert.Equal("email", ex.Field);
    }

    [Fact]
    public void ValidateUuid_ValidInput_DoesNotThrow()
    {
        _validator.ValidateUuid("550e8400-e29b-41d4-a716-446655440000");
    }

    [Theory]
    [InlineData("")]
    [InlineData("not-a-uuid")]
    [InlineData("550e8400-e29b-41d4-a716")]
    public void ValidateUuid_InvalidInput_ThrowsValidationException(string id)
    {
        var ex = Assert.Throws<ValidationException>(() => _validator.ValidateUuid(id));
        Assert.Equal("id", ex.Field);
    }

    [Theory]
    [InlineData("https://example.com")]
    [InlineData("http://localhost:8080/path")]
    public void ValidateUrl_ValidInput_DoesNotThrow(string url)
    {
        _validator.ValidateUrl(url);
    }

    [Theory]
    [InlineData("")]
    [InlineData("not-a-url")]
    [InlineData("ftp://invalid-scheme.com")]
    public void ValidateUrl_InvalidInput_ThrowsValidationException(string url)
    {
        var ex = Assert.Throws<ValidationException>(() => _validator.ValidateUrl(url));
        Assert.Equal("url", ex.Field);
    }

    [Theory]
    [InlineData("acme-corp")]
    [InlineData("tenant123")]
    public void ValidateTenantId_ValidInput_DoesNotThrow(string tenantId)
    {
        _validator.ValidateTenantId(tenantId);
    }

    [Theory]
    [InlineData("")]
    [InlineData("A")]
    [InlineData("1starts-with-digit")]
    public void ValidateTenantId_InvalidInput_ThrowsValidationException(string tenantId)
    {
        var ex = Assert.Throws<ValidationException>(() => _validator.ValidateTenantId(tenantId));
        Assert.Equal("tenantId", ex.Field);
    }
}
