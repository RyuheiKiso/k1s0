namespace K1s0.System.Config;

public static class ConfigValidator
{
    private static readonly string[] _validTiers = ["system", "business", "service"];
    private static readonly string[] _validEnvironments = ["dev", "staging", "prod"];

    public static void Validate(AppConfig config)
    {
        if (string.IsNullOrWhiteSpace(config.App.Name))
        {
            throw new ConfigException(
                ConfigErrorCodes.Validation,
                "app.name is required");
        }

        if (string.IsNullOrWhiteSpace(config.App.Version))
        {
            throw new ConfigException(
                ConfigErrorCodes.Validation,
                "app.version is required");
        }

        if (!_validTiers.Contains(config.App.Tier))
        {
            throw new ConfigException(
                ConfigErrorCodes.Validation,
                "app.tier must be system, business, or service");
        }

        if (!_validEnvironments.Contains(config.App.Environment))
        {
            throw new ConfigException(
                ConfigErrorCodes.Validation,
                "app.environment must be dev, staging, or prod");
        }

        if (string.IsNullOrWhiteSpace(config.Server.Host))
        {
            throw new ConfigException(
                ConfigErrorCodes.Validation,
                "server.host is required");
        }

        if (config.Server.Port <= 0)
        {
            throw new ConfigException(
                ConfigErrorCodes.Validation,
                "server.port must be > 0");
        }

        if (config.Auth is not null)
        {
            if (string.IsNullOrWhiteSpace(config.Auth.Jwt.Issuer))
            {
                throw new ConfigException(
                    ConfigErrorCodes.Validation,
                    "auth.jwt.issuer is required");
            }

            if (string.IsNullOrWhiteSpace(config.Auth.Jwt.Audience))
            {
                throw new ConfigException(
                    ConfigErrorCodes.Validation,
                    "auth.jwt.audience is required");
            }
        }
    }
}
