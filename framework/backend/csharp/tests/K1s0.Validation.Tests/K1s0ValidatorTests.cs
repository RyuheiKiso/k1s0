using FluentAssertions;
using FluentValidation.TestHelper;

namespace K1s0.Validation.Tests;

public record SampleCommand(string Name, string ServiceId, string ErrorCode);

public class SampleCommandValidator : K1s0Validator<SampleCommand>
{
    public SampleCommandValidator()
    {
        RuleForRequiredString(x => x.Name, 2, 50);
        RuleForKebabCase(x => x.ServiceId);
        RuleForErrorCode(x => x.ErrorCode);
    }
}

public class K1s0ValidatorTests
{
    private readonly SampleCommandValidator _validator = new();

    [Fact]
    public void Valid_Input_PassesAllRules()
    {
        var command = new SampleCommand("Alice", "user-management", "auth.token.expired");

        var result = _validator.TestValidate(command);

        result.ShouldNotHaveAnyValidationErrors();
    }

    [Theory]
    [InlineData("")]
    [InlineData("A")]
    public void Name_TooShortOrEmpty_Fails(string name)
    {
        var command = new SampleCommand(name, "svc", "a.b.c");
        var result = _validator.TestValidate(command);
        result.ShouldHaveValidationErrorFor(x => x.Name);
    }

    [Theory]
    [InlineData("UserManagement")]
    [InlineData("user_management")]
    [InlineData("")]
    public void ServiceId_NotKebabCase_Fails(string serviceId)
    {
        var command = new SampleCommand("Alice", serviceId, "a.b.c");
        var result = _validator.TestValidate(command);
        result.ShouldHaveValidationErrorFor(x => x.ServiceId);
    }

    [Theory]
    [InlineData("user-mgmt")]
    [InlineData("a")]
    [InlineData("my-long-service-123")]
    public void ServiceId_ValidKebabCase_Passes(string serviceId)
    {
        var command = new SampleCommand("Alice", serviceId, "a.b.c");
        var result = _validator.TestValidate(command);
        result.ShouldNotHaveValidationErrorFor(x => x.ServiceId);
    }

    [Theory]
    [InlineData("invalid")]
    [InlineData("a.b")]
    [InlineData("")]
    public void ErrorCode_InvalidFormat_Fails(string errorCode)
    {
        var command = new SampleCommand("Alice", "svc", errorCode);
        var result = _validator.TestValidate(command);
        result.ShouldHaveValidationErrorFor(x => x.ErrorCode);
    }

    [Fact]
    public void ErrorCode_ValidFormat_Passes()
    {
        var command = new SampleCommand("Alice", "svc", "auth.token.expired");
        var result = _validator.TestValidate(command);
        result.ShouldNotHaveValidationErrorFor(x => x.ErrorCode);
    }
}
