using System.IdentityModel.Tokens.Jwt;
using System.Security.Claims;
using System.Text;
using Microsoft.IdentityModel.Tokens;

namespace K1s0.System.TestHelper.Jwt;

/// <summary>テスト用 JWT トークン生成ヘルパー。</summary>
public class JwtTestHelper
{
    private readonly string _secret;
    private readonly string _algorithm;

    public JwtTestHelper(string secret, string algorithm = "HS256")
    {
        _secret = secret;
        _algorithm = algorithm;
    }

    /// <summary>管理者トークンを生成する。</summary>
    public string CreateAdminToken()
    {
        return CreateToken(new TestClaims("admin", new[] { "admin" }));
    }

    /// <summary>ユーザートークンを生成する。</summary>
    public string CreateUserToken(string userId, params string[] roles)
    {
        return CreateToken(new TestClaims(userId, roles));
    }

    /// <summary>カスタムクレームでトークンを生成する。</summary>
    public string CreateToken(TestClaims claims)
    {
        var key = new SymmetricSecurityKey(Encoding.UTF8.GetBytes(_secret.PadRight(32, '0')));
        var credentials = new SigningCredentials(key, SecurityAlgorithms.HmacSha256);

        var tokenClaims = new List<Claim>
        {
            new(JwtRegisteredClaimNames.Sub, claims.Sub),
        };

        foreach (var role in claims.Roles)
        {
            tokenClaims.Add(new Claim("role", role));
        }

        if (claims.TenantId is not null)
        {
            tokenClaims.Add(new Claim("tenant_id", claims.TenantId));
        }

        var expiry = claims.Expiry ?? TimeSpan.FromHours(1);

        var token = new JwtSecurityToken(
            claims: tokenClaims,
            expires: DateTime.UtcNow.Add(expiry),
            signingCredentials: credentials);

        return new JwtSecurityTokenHandler().WriteToken(token);
    }

    /// <summary>トークンのペイロードをデコードしてクレームを返す。</summary>
    public TestClaims? DecodeClaims(string token)
    {
        try
        {
            var handler = new JwtSecurityTokenHandler();
            var jwt = handler.ReadJwtToken(token);
            var sub = jwt.Claims.FirstOrDefault(c => c.Type == JwtRegisteredClaimNames.Sub)?.Value ?? string.Empty;
            var roles = jwt.Claims.Where(c => c.Type == "role").Select(c => c.Value).ToList();
            var tenantId = jwt.Claims.FirstOrDefault(c => c.Type == "tenant_id")?.Value;
            return new TestClaims(sub, roles, tenantId);
        }
        catch
        {
            return null;
        }
    }
}
