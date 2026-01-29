using Microsoft.Extensions.Configuration;

namespace K1s0.Config;

/// <summary>
/// Extension methods for adding k1s0 YAML configuration to <see cref="IConfigurationBuilder"/>.
/// </summary>
public static class K1s0ConfigurationExtensions
{
    /// <summary>
    /// Adds k1s0 YAML configuration files to the configuration builder.
    /// Loads config/default.yaml, then config/{env}.yaml based on the --env argument.
    /// If --secrets-dir is provided, loads secret files as key-value pairs.
    /// </summary>
    /// <param name="builder">The configuration builder.</param>
    /// <param name="args">Command-line arguments.</param>
    /// <returns>The configuration builder for chaining.</returns>
    public static IConfigurationBuilder AddK1s0YamlConfig(this IConfigurationBuilder builder, string[] args)
    {
        ArgumentNullException.ThrowIfNull(builder);
        ArgumentNullException.ThrowIfNull(args);

        string basePath = Directory.GetCurrentDirectory();
        string env = ParseArgValue(args, "--env") ?? "default";
        string? secretsDir = ParseArgValue(args, "--secrets-dir");

        string defaultPath = Path.Combine(basePath, "config", "default.yaml");
        builder.Add(new YamlConfigurationSource { Path = defaultPath, Optional = false });

        if (!string.Equals(env, "default", StringComparison.OrdinalIgnoreCase))
        {
            string envPath = Path.Combine(basePath, "config", $"{env}.yaml");
            builder.Add(new YamlConfigurationSource { Path = envPath, Optional = true });
        }

        if (secretsDir is not null && Directory.Exists(secretsDir))
        {
            LoadSecrets(builder, secretsDir);
        }

        return builder;
    }

    private static void LoadSecrets(IConfigurationBuilder builder, string secretsDir)
    {
        var secretData = new Dictionary<string, string?>(StringComparer.OrdinalIgnoreCase);

        foreach (string file in Directory.GetFiles(secretsDir))
        {
            string key = Path.GetFileName(file);
            string value = File.ReadAllText(file).Trim();
            secretData[key] = value;
        }

        builder.AddInMemoryCollection(secretData);
    }

    private static string? ParseArgValue(string[] args, string name)
    {
        for (int i = 0; i < args.Length - 1; i++)
        {
            if (string.Equals(args[i], name, StringComparison.OrdinalIgnoreCase))
            {
                return args[i + 1];
            }
        }

        return null;
    }
}
