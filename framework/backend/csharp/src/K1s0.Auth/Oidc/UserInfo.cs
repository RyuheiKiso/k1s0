using System.Text.Json.Serialization;

namespace K1s0.Auth.Oidc;

/// <summary>
/// Represents user information retrieved from the OIDC userinfo endpoint.
/// </summary>
/// <param name="Sub">The subject identifier.</param>
/// <param name="Name">The user's full name.</param>
/// <param name="Email">The user's email address.</param>
/// <param name="EmailVerified">Whether the email address has been verified.</param>
public record UserInfo(
    [property: JsonPropertyName("sub")] string Sub,
    [property: JsonPropertyName("name")] string? Name,
    [property: JsonPropertyName("email")] string? Email,
    [property: JsonPropertyName("email_verified")] bool EmailVerified);
