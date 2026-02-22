using Xunit;

namespace K1s0.System.Config.Tests;

public class ConfigValidatorTests
{
    [Fact]
    public void Validate_ValidConfig_DoesNotThrow()
    {
        var config = CreateValidConfig();
        ConfigValidator.Validate(config);
    }

    [Fact]
    public void Validate_EmptyAppName_Throws()
    {
        var config = CreateValidConfig() with
        {
            App = CreateValidConfig().App with { Name = string.Empty },
        };

        var ex = Assert.Throws<ConfigException>(
            () => ConfigValidator.Validate(config));
        Assert.Equal(ConfigErrorCodes.Validation, ex.Code);
        Assert.Contains("app.name", ex.Message);
    }

    [Fact]
    public void Validate_EmptyAppVersion_Throws()
    {
        var config = CreateValidConfig() with
        {
            App = CreateValidConfig().App with { Version = string.Empty },
        };

        var ex = Assert.Throws<ConfigException>(
            () => ConfigValidator.Validate(config));
        Assert.Contains("app.version", ex.Message);
    }

    [Fact]
    public void Validate_InvalidTier_Throws()
    {
        var config = CreateValidConfig() with
        {
            App = CreateValidConfig().App with { Tier = "invalid" },
        };

        var ex = Assert.Throws<ConfigException>(
            () => ConfigValidator.Validate(config));
        Assert.Contains("app.tier", ex.Message);
    }

    [Fact]
    public void Validate_InvalidEnvironment_Throws()
    {
        var config = CreateValidConfig() with
        {
            App = CreateValidConfig().App with { Environment = "test" },
        };

        var ex = Assert.Throws<ConfigException>(
            () => ConfigValidator.Validate(config));
        Assert.Contains("app.environment", ex.Message);
    }

    [Fact]
    public void Validate_EmptyServerHost_Throws()
    {
        var config = CreateValidConfig() with
        {
            Server = CreateValidConfig().Server with { Host = string.Empty },
        };

        var ex = Assert.Throws<ConfigException>(
            () => ConfigValidator.Validate(config));
        Assert.Contains("server.host", ex.Message);
    }

    [Fact]
    public void Validate_ZeroServerPort_Throws()
    {
        var config = CreateValidConfig() with
        {
            Server = CreateValidConfig().Server with { Port = 0 },
        };

        var ex = Assert.Throws<ConfigException>(
            () => ConfigValidator.Validate(config));
        Assert.Contains("server.port", ex.Message);
    }

    [Fact]
    public void Validate_EmptyJwtIssuer_Throws()
    {
        var config = CreateValidConfig() with
        {
            Auth = new AuthSection
            {
                Jwt = new JwtConfig { Issuer = string.Empty, Audience = "k1s0-api" },
            },
        };

        var ex = Assert.Throws<ConfigException>(
            () => ConfigValidator.Validate(config));
        Assert.Contains("auth.jwt.issuer", ex.Message);
    }

    [Fact]
    public void Validate_EmptyJwtAudience_Throws()
    {
        var config = CreateValidConfig() with
        {
            Auth = new AuthSection
            {
                Jwt = new JwtConfig { Issuer = "http://localhost", Audience = string.Empty },
            },
        };

        var ex = Assert.Throws<ConfigException>(
            () => ConfigValidator.Validate(config));
        Assert.Contains("auth.jwt.audience", ex.Message);
    }

    private static AppConfig CreateValidConfig() => new()
    {
        App = new AppSection
        {
            Name = "test-server",
            Version = "1.0.0",
            Tier = "system",
            Environment = "dev",
        },
        Server = new ServerSection
        {
            Host = "0.0.0.0",
            Port = 8080,
        },
        Auth = new AuthSection
        {
            Jwt = new JwtConfig
            {
                Issuer = "http://localhost:8180/realms/k1s0",
                Audience = "k1s0-api",
            },
        },
    };
}
