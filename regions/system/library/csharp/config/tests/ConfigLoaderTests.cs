using Xunit;

namespace K1s0.System.Config.Tests;

public class ConfigLoaderTests : IDisposable
{
    private readonly string _tempDir;

    public ConfigLoaderTests()
    {
        _tempDir = Path.Combine(Path.GetTempPath(), Guid.NewGuid().ToString());
        Directory.CreateDirectory(_tempDir);
    }

    public void Dispose()
    {
        if (Directory.Exists(_tempDir))
        {
            Directory.Delete(_tempDir, true);
        }
    }

    private string WriteYaml(string filename, string content)
    {
        string path = Path.Combine(_tempDir, filename);
        File.WriteAllText(path, content);
        return path;
    }

    [Fact]
    public void Load_ValidConfig_ReturnsAppConfig()
    {
        string path = WriteYaml("config.yaml", ValidBaseYaml);

        var config = ConfigLoader.Load(path);

        Assert.Equal("test-server", config.App.Name);
        Assert.Equal("1.0.0", config.App.Version);
        Assert.Equal("system", config.App.Tier);
        Assert.Equal("dev", config.App.Environment);
        Assert.Equal("0.0.0.0", config.Server.Host);
        Assert.Equal(8080, config.Server.Port);
    }

    [Fact]
    public void Load_WithEnvOverride_MergesCorrectly()
    {
        string basePath = WriteYaml("config.yaml", ValidBaseYaml);
        string envPath = WriteYaml("config.prod.yaml", @"
app:
  environment: prod
server:
  port: 9090
");

        var config = ConfigLoader.Load(basePath, envPath);

        Assert.Equal("prod", config.App.Environment);
        Assert.Equal(9090, config.Server.Port);
        Assert.Equal("test-server", config.App.Name);
    }

    [Fact]
    public void Load_FileNotFound_ThrowsConfigException()
    {
        var ex = Assert.Throws<ConfigException>(
            () => ConfigLoader.Load("/nonexistent/config.yaml"));
        Assert.Equal(ConfigErrorCodes.ReadFile, ex.Code);
    }

    [Fact]
    public void Load_InvalidYaml_ThrowsConfigException()
    {
        string path = WriteYaml("bad.yaml", "not: [valid: yaml: {{");

        var ex = Assert.Throws<ConfigException>(
            () => ConfigLoader.Load(path));
        Assert.Equal(ConfigErrorCodes.ParseYaml, ex.Code);
    }

    [Fact]
    public void Load_MissingRequiredField_ThrowsConfigException()
    {
        string path = WriteYaml("incomplete.yaml", @"
app:
  name: ''
  version: '1.0'
  tier: system
  environment: dev
server:
  host: '0.0.0.0'
  port: 8080
");

        var ex = Assert.Throws<ConfigException>(
            () => ConfigLoader.Load(path));
        Assert.Equal(ConfigErrorCodes.Validation, ex.Code);
    }

    private const string ValidBaseYaml = @"
app:
  name: test-server
  version: '1.0.0'
  tier: system
  environment: dev
server:
  host: '0.0.0.0'
  port: 8080
observability:
  log:
    level: debug
    format: json
auth:
  jwt:
    issuer: 'http://localhost:8180/realms/k1s0'
    audience: 'k1s0-api'
";
}
