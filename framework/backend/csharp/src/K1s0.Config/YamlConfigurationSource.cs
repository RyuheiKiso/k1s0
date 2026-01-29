using Microsoft.Extensions.Configuration;

namespace K1s0.Config;

/// <summary>
/// An <see cref="IConfigurationSource"/> that reads YAML files.
/// </summary>
public class YamlConfigurationSource : IConfigurationSource
{
    /// <summary>
    /// The path to the YAML file.
    /// </summary>
    public required string Path { get; init; }

    /// <summary>
    /// Whether the file is optional. Defaults to false.
    /// </summary>
    public bool Optional { get; init; }

    /// <inheritdoc />
    public IConfigurationProvider Build(IConfigurationBuilder builder)
    {
        return new YamlConfigurationProvider(this);
    }
}
