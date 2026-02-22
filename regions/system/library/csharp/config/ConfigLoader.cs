using System.IO;
using YamlDotNet.RepresentationModel;
using YamlDotNet.Serialization;
using YamlDotNet.Serialization.NamingConventions;

namespace K1s0.System.Config;

public static class ConfigLoader
{
    private static readonly IDeserializer _deserializer = new DeserializerBuilder()
        .WithNamingConvention(UnderscoredNamingConvention.Instance)
        .Build();

    public static AppConfig Load(string basePath, string? envPath = null)
    {
        string baseYaml;
        try
        {
            baseYaml = File.ReadAllText(basePath);
        }
        catch (IOException ex)
        {
            throw new ConfigException(
                ConfigErrorCodes.ReadFile,
                $"Failed to read config file: {basePath}",
                ex);
        }

        if (envPath is not null)
        {
            string envYaml;
            try
            {
                envYaml = File.ReadAllText(envPath);
            }
            catch (IOException ex)
            {
                throw new ConfigException(
                    ConfigErrorCodes.ReadFile,
                    $"Failed to read env config file: {envPath}",
                    ex);
            }

            baseYaml = MergeYamlStrings(baseYaml, envYaml);
        }

        try
        {
            var config = _deserializer.Deserialize<AppConfig>(baseYaml);
            ConfigValidator.Validate(config);
            return config;
        }
        catch (YamlDotNet.Core.YamlException ex)
        {
            throw new ConfigException(
                ConfigErrorCodes.ParseYaml,
                "Failed to parse YAML config",
                ex);
        }
    }

    private static string MergeYamlStrings(string baseYaml, string overlayYaml)
    {
        var baseStream = new YamlStream();
        baseStream.Load(new StringReader(baseYaml));

        var overlayStream = new YamlStream();
        overlayStream.Load(new StringReader(overlayYaml));

        if (baseStream.Documents.Count == 0)
        {
            return overlayYaml;
        }

        if (overlayStream.Documents.Count == 0)
        {
            return baseYaml;
        }

        var baseRoot = baseStream.Documents[0].RootNode as YamlMappingNode
            ?? new YamlMappingNode();
        var overlayRoot = overlayStream.Documents[0].RootNode as YamlMappingNode
            ?? new YamlMappingNode();

        var merged = MergeHelper.DeepMerge(baseRoot, overlayRoot);

        var resultStream = new YamlStream(new YamlDocument(merged));
        using var writer = new StringWriter();
        resultStream.Save(writer);
        return writer.ToString();
    }
}
