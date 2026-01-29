using System.Net;
using FluentAssertions;

namespace K1s0.Error.Tests;

public class ProblemDetailsFactoryTests
{
    [Fact]
    public void Create_FromNotFoundException_ReturnsProblemDetails()
    {
        var ex = new NotFoundException("user.query.not_found", "User 123 was not found");

        var details = ProblemDetailsFactory.Create(ex);

        details["status"].Should().Be(404);
        details["title"].Should().Be("Not Found");
        details["detail"].Should().Be("User 123 was not found");
        details["error_code"].Should().Be("user.query.not_found");
    }

    [Fact]
    public void Create_FromValidationException_IncludesErrors()
    {
        var errors = new Dictionary<string, string[]>
        {
            ["email"] = ["Invalid email format"],
        };
        var ex = new ValidationException("user.input.invalid", "Validation failed", errors);

        var details = ProblemDetailsFactory.Create(ex);

        details.Should().ContainKey("errors");
        details["status"].Should().Be(400);
    }

    [Fact]
    public void Create_FromBasicException_OmitsTraceIdWhenNull()
    {
        var ex = new K1s0Exception("svc.cat.reason", "error");

        var details = ProblemDetailsFactory.Create(ex);

        details.Should().NotContainKey("trace_id");
    }

    [Fact]
    public void Create_NullException_ThrowsArgumentNullException()
    {
        var act = () => ProblemDetailsFactory.Create(null!);
        act.Should().Throw<ArgumentNullException>();
    }
}
