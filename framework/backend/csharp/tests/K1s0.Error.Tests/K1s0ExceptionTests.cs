using System.Net;
using FluentAssertions;

namespace K1s0.Error.Tests;

public class K1s0ExceptionTests
{
    [Fact]
    public void K1s0Exception_SetsAllProperties()
    {
        var ex = new K1s0Exception("svc.cat.reason", "something broke", HttpStatusCode.BadGateway);

        ex.ErrorCode.Should().Be("svc.cat.reason");
        ex.Message.Should().Be("something broke");
        ex.HttpStatus.Should().Be(HttpStatusCode.BadGateway);
    }

    [Fact]
    public void NotFoundException_HasStatus404()
    {
        var ex = new NotFoundException("user.query.not_found", "User not found");

        ex.HttpStatus.Should().Be(HttpStatusCode.NotFound);
        ex.ErrorCode.Should().Be("user.query.not_found");
    }

    [Fact]
    public void ValidationException_HasStatus400AndErrors()
    {
        var errors = new Dictionary<string, string[]>
        {
            ["name"] = ["Name is required"],
        };

        var ex = new ValidationException("user.input.invalid", "Validation failed", errors);

        ex.HttpStatus.Should().Be(HttpStatusCode.BadRequest);
        ex.Errors.Should().ContainKey("name");
    }

    [Fact]
    public void ConflictException_HasStatus409()
    {
        var ex = new ConflictException("user.create.conflict", "Already exists");
        ex.HttpStatus.Should().Be(HttpStatusCode.Conflict);
    }

    [Fact]
    public void UnauthorizedException_HasStatus401()
    {
        var ex = new UnauthorizedException("auth.token.expired", "Token expired");
        ex.HttpStatus.Should().Be(HttpStatusCode.Unauthorized);
    }

    [Fact]
    public void ForbiddenException_HasStatus403()
    {
        var ex = new ForbiddenException("auth.access.denied", "Access denied");
        ex.HttpStatus.Should().Be(HttpStatusCode.Forbidden);
    }

    [Fact]
    public void K1s0Exception_DefaultStatusIsInternalServerError()
    {
        var ex = new K1s0Exception("svc.cat.reason", "error");
        ex.HttpStatus.Should().Be(HttpStatusCode.InternalServerError);
    }

    [Fact]
    public void K1s0Exception_PreservesInnerException()
    {
        var inner = new InvalidOperationException("inner");
        var ex = new K1s0Exception("svc.cat.reason", "outer", innerException: inner);

        ex.InnerException.Should().BeSameAs(inner);
    }
}

public class ErrorCodeTests
{
    [Theory]
    [InlineData("auth.token.expired", "auth", "token", "expired")]
    [InlineData("user-mgmt.query.not_found", "user-mgmt", "query", "not_found")]
    public void Parse_ValidCode_ReturnsErrorCode(string input, string service, string category, string reason)
    {
        var code = ErrorCode.Parse(input);

        code.Service.Should().Be(service);
        code.Category.Should().Be(category);
        code.Reason.Should().Be(reason);
    }

    [Fact]
    public void Parse_InvalidCode_ThrowsFormatException()
    {
        var act = () => ErrorCode.Parse("invalid");
        act.Should().Throw<FormatException>();
    }

    [Theory]
    [InlineData("a.b.c", true)]
    [InlineData("abc", false)]
    [InlineData("a.b", false)]
    [InlineData("", false)]
    [InlineData(null, false)]
    [InlineData("A.B.C", false)]
    public void IsValid_ReturnsExpected(string? input, bool expected)
    {
        ErrorCode.IsValid(input).Should().Be(expected);
    }

    [Fact]
    public void ToString_ReturnsCanonicalForm()
    {
        var code = new ErrorCode("auth", "token", "expired");
        code.ToString().Should().Be("auth.token.expired");
    }

    [Fact]
    public void TryParse_InvalidInput_ReturnsFalse()
    {
        ErrorCode.TryParse("bad", out var result).Should().BeFalse();
        result.Should().BeNull();
    }
}
