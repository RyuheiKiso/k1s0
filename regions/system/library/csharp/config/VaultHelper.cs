namespace K1s0.System.Config;

public static class VaultHelper
{
    public static AppConfig MergeVaultSecrets(
        AppConfig config,
        IDictionary<string, string> secrets)
    {
        var result = config;

        if (secrets.TryGetValue("database.password", out string? dbPassword)
            && result.Database is not null)
        {
            result = result with
            {
                Database = result.Database with { Password = dbPassword },
            };
        }

        if (secrets.TryGetValue("redis.password", out string? redisPassword)
            && result.Redis is not null)
        {
            result = result with
            {
                Redis = result.Redis with { Password = redisPassword },
            };
        }

        if (secrets.TryGetValue("redis_session.password", out string? redisSessionPassword)
            && result.RedisSession is not null)
        {
            result = result with
            {
                RedisSession = result.RedisSession with { Password = redisSessionPassword },
            };
        }

        if (secrets.TryGetValue("kafka.sasl.username", out string? kafkaUser)
            && result.Kafka?.Sasl is not null)
        {
            result = result with
            {
                Kafka = result.Kafka with
                {
                    Sasl = result.Kafka.Sasl with { Username = kafkaUser },
                },
            };
        }

        if (secrets.TryGetValue("kafka.sasl.password", out string? kafkaPassword)
            && result.Kafka?.Sasl is not null)
        {
            result = result with
            {
                Kafka = result.Kafka with
                {
                    Sasl = result.Kafka.Sasl with { Password = kafkaPassword },
                },
            };
        }

        if (secrets.TryGetValue("auth.oidc.client_secret", out string? oidcSecret)
            && result.Auth?.Oidc is not null)
        {
            result = result with
            {
                Auth = result.Auth with
                {
                    Oidc = result.Auth.Oidc with { ClientSecret = oidcSecret },
                },
            };
        }

        return result;
    }
}
