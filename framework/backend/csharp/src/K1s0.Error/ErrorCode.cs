using System.Diagnostics.CodeAnalysis;
using System.Text.RegularExpressions;

namespace K1s0.Error;

/// <summary>
/// Represents a structured error code in the format "{service}.{category}.{reason}".
/// </summary>
/// <param name="Service">The service that produced the error.</param>
/// <param name="Category">The error category within the service.</param>
/// <param name="Reason">The specific reason for the error.</param>
public sealed partial record ErrorCode(string Service, string Category, string Reason)
{
    private static readonly Regex Pattern = ErrorCodeRegex();

    /// <summary>
    /// Returns the canonical string representation "{service}.{category}.{reason}".
    /// </summary>
    public override string ToString() => $"{Service}.{Category}.{Reason}";

    /// <summary>
    /// Parses an error code string in the format "{service}.{category}.{reason}".
    /// </summary>
    /// <param name="value">The string to parse.</param>
    /// <returns>A valid <see cref="ErrorCode"/> instance.</returns>
    /// <exception cref="FormatException">Thrown when the value does not match the expected format.</exception>
    public static ErrorCode Parse(string value)
    {
        if (!TryParse(value, out var result))
        {
            throw new FormatException(
                $"Invalid error code format '{value}'. Expected '{{service}}.{{category}}.{{reason}}' using lowercase letters, digits, and underscores.");
        }

        return result;
    }

    /// <summary>
    /// Attempts to parse an error code string.
    /// </summary>
    /// <param name="value">The string to parse.</param>
    /// <param name="result">The parsed error code, or null if parsing failed.</param>
    /// <returns>True if parsing succeeded; otherwise, false.</returns>
    public static bool TryParse(string? value, [NotNullWhen(true)] out ErrorCode? result)
    {
        result = null;

        if (string.IsNullOrWhiteSpace(value))
        {
            return false;
        }

        var match = Pattern.Match(value);
        if (!match.Success)
        {
            return false;
        }

        result = new ErrorCode(
            match.Groups["service"].Value,
            match.Groups["category"].Value,
            match.Groups["reason"].Value);

        return true;
    }

    /// <summary>
    /// Validates whether the given string is a well-formed error code.
    /// </summary>
    /// <param name="value">The string to validate.</param>
    /// <returns>True if the string matches the error code format; otherwise, false.</returns>
    public static bool IsValid(string? value) => !string.IsNullOrWhiteSpace(value) && Pattern.IsMatch(value);

    [GeneratedRegex(@"^(?<service>[a-z][a-z0-9_-]*)\.(?<category>[a-z][a-z0-9_]*)\.(?<reason>[a-z][a-z0-9_]*)$", RegexOptions.Compiled)]
    private static partial Regex ErrorCodeRegex();
}
