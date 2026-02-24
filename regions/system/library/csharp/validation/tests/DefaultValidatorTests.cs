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

    [Theory]
    [InlineData(1, 10)]
    [InlineData(1, 1)]
    [InlineData(1, 100)]
    [InlineData(999, 50)]
    public void ValidatePagination_ValidInput_DoesNotThrow(int page, int perPage)
    {
        _validator.ValidatePagination(page, perPage);
    }

    [Fact]
    public void ValidatePagination_PageLessThan1_ThrowsWithCode()
    {
        var ex = Assert.Throws<ValidationException>(() => _validator.ValidatePagination(0, 10));
        Assert.Equal("page", ex.Field);
        Assert.Equal("INVALID_PAGE", ex.Code);
    }

    [Theory]
    [InlineData(1, 0)]
    [InlineData(1, 101)]
    public void ValidatePagination_PerPageOutOfRange_ThrowsWithCode(int page, int perPage)
    {
        var ex = Assert.Throws<ValidationException>(() => _validator.ValidatePagination(page, perPage));
        Assert.Equal("perPage", ex.Field);
        Assert.Equal("INVALID_PER_PAGE", ex.Code);
    }

    [Fact]
    public void ValidateDateRange_ValidRange_DoesNotThrow()
    {
        var start = new DateTime(2024, 1, 1, 0, 0, 0, DateTimeKind.Utc);
        var end = new DateTime(2024, 12, 31, 23, 59, 59, DateTimeKind.Utc);
        _validator.ValidateDateRange(start, end);
    }

    [Fact]
    public void ValidateDateRange_EqualDates_DoesNotThrow()
    {
        var dt = new DateTime(2024, 6, 15, 12, 0, 0, DateTimeKind.Utc);
        _validator.ValidateDateRange(dt, dt);
    }

    [Fact]
    public void ValidateDateRange_StartAfterEnd_ThrowsWithCode()
    {
        var start = new DateTime(2024, 12, 31, 23, 59, 59, DateTimeKind.Utc);
        var end = new DateTime(2024, 1, 1, 0, 0, 0, DateTimeKind.Utc);
        var ex = Assert.Throws<ValidationException>(() => _validator.ValidateDateRange(start, end));
        Assert.Equal("dateRange", ex.Field);
        Assert.Equal("INVALID_DATE_RANGE", ex.Code);
    }

    [Fact]
    public void ValidationException_HasCode()
    {
        var ex = Assert.Throws<ValidationException>(() => _validator.ValidateEmail("bad"));
        Assert.Equal("INVALID_EMAIL", ex.Code);
    }

    [Fact]
    public void ValidationErrors_EmptyCollection()
    {
        var errors = new ValidationErrors();
        Assert.False(errors.HasErrors());
        Assert.Empty(errors.GetErrors());
    }

    [Fact]
    public void ValidationErrors_AddAndRetrieve()
    {
        var errors = new ValidationErrors();
        errors.Add(new ValidationException("email", "bad", "INVALID_EMAIL"));
        errors.Add(new ValidationException("page", "bad", "INVALID_PAGE"));

        Assert.True(errors.HasErrors());
        Assert.Equal(2, errors.GetErrors().Count);
        Assert.Equal("INVALID_EMAIL", errors.GetErrors()[0].Code);
        Assert.Equal("INVALID_PAGE", errors.GetErrors()[1].Code);
    }
}
