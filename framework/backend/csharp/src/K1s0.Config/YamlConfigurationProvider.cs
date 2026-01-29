using Microsoft.Extensions.Configuration;
using YamlDotNet.RepresentationModel;

namespace K1s0.Config;

/// <summary>
/// An <see cref="IConfigurationProvider"/> that reads configuration from YAML files,
/// flattening nested keys with the ":" separator used by Microsoft.Extensions.Configuration.
/// </summary>
public class YamlConfigurationProvider : ConfigurationProvider
{
    private readonly YamlConfigurationSource _source;

    /// <summary>
    /// Creates a new <see cref="YamlConfigurationProvider"/>.
    /// </summary>
    /// <param name="source">The YAML configuration source.</param>
    public YamlConfigurationProvider(YamlConfigurationSource source)
    {
        _source = source;
    }

    /// <inheritdoc />
    public override void Load()
    {
        if (!File.Exists(_source.Path))
        {
            if (_source.Optional)
            {
                Data = new Dictionary<string, string?>(StringComparer.OrdinalIgnoreCase);
                return;
            }

            throw new FileNotFoundException($"Configuration file not found: {_source.Path}", _source.Path);
        }

        using var reader = new StreamReader(_source.Path);
        var yaml = new YamlStream();
        yaml.Load(reader);

        var data = new Dictionary<string, string?>(StringComparer.OrdinalIgnoreCase);

        if (yaml.Documents.Count > 0 && yaml.Documents[0].RootNode is YamlMappingNode root)
        {
            VisitNode(root, string.Empty, data);
        }

        Data = data;
    }

    private static void VisitNode(YamlNode node, string prefix, Dictionary<string, string?> data)
    {
        switch (node)
        {
            case YamlMappingNode mapping:
                foreach (var entry in mapping.Children)
                {
                    string key = ((YamlScalarNode)entry.Key).Value!;
                    string fullKey = string.IsNullOrEmpty(prefix) ? key : $"{prefix}:{key}";
                    VisitNode(entry.Value, fullKey, data);
                }
                break;

            case YamlSequenceNode sequence:
                for (int i = 0; i < sequence.Children.Count; i++)
                {
                    string fullKey = $"{prefix}:{i}";
                    VisitNode(sequence.Children[i], fullKey, data);
                }
                break;

            case YamlScalarNode scalar:
                data[prefix] = scalar.Value;
                break;
        }
    }
}
