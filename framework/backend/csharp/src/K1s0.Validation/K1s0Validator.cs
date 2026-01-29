using System.Text.RegularExpressions;
using FluentValidation;

namespace K1s0.Validation;

/// <summary>
/// Base validator with common rule helpers for k1s0 services.
/// </summary>
/// <typeparam name="T">The type being validated.</typeparam>
public abstract partial class K1s0Validator<T> : AbstractValidator<T>
{
    private static readonly Regex KebabCasePattern = KebabCaseRegex();
    private static readonly Regex ErrorCodePattern = ErrorCodeRegex();

    /// <summary>
    /// Validates that a string property is in kebab-case format (e.g., "user-management").
    /// </summary>
    protected IRuleBuilderOptions<T, string> RuleForKebabCase(
        System.Linq.Expressions.Expression<Func<T, string>> expression)
    {
        return RuleFor(expression)
            .NotEmpty()
            .Must(value => KebabCasePattern.IsMatch(value))
            .WithMessage("'{PropertyName}' must be in kebab-case format (e.g., 'my-service').");
    }

    /// <summary>
    /// Validates that a string property is a valid error code in "{service}.{category}.{reason}" format.
    /// </summary>
    protected IRuleBuilderOptions<T, string> RuleForErrorCode(
        System.Linq.Expressions.Expression<Func<T, string>> expression)
    {
        return RuleFor(expression)
            .NotEmpty()
            .Must(value => ErrorCodePattern.IsMatch(value))
            .WithMessage("'{PropertyName}' must be in '{service}.{category}.{reason}' format.");
    }

    /// <summary>
    /// Validates that a string property is not null, empty, or whitespace, and within a length range.
    /// </summary>
    protected IRuleBuilderOptions<T, string> RuleForRequiredString(
        System.Linq.Expressions.Expression<Func<T, string>> expression,
        int minLength = 1,
        int maxLength = 255)
    {
        return RuleFor(expression)
            .NotEmpty()
            .MinimumLength(minLength)
            .MaximumLength(maxLength);
    }

    [GeneratedRegex(@"^[a-z][a-z0-9]*(-[a-z0-9]+)*$", RegexOptions.Compiled)]
    private static partial Regex KebabCaseRegex();

    [GeneratedRegex(@"^[a-z][a-z0-9_-]*\.[a-z][a-z0-9_]*\.[a-z][a-z0-9_]*$", RegexOptions.Compiled)]
    private static partial Regex ErrorCodeRegex();
}
