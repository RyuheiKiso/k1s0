using FluentAssertions;
using Microsoft.Extensions.Configuration;

namespace K1s0.Config.Tests;

public class YamlConfigurationTests : IDisposable
{
    private readonly string _tempDir;

    public YamlConfigurationTests()
    {
        _tempDir = Path.Combine(Path.GetTempPath(), $"k1s0_config_test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(Path.Combine(_tempDir, "config"));
    }

    public void Dispose()
    {
        if (Directory.Exists(_tempDir))
        {
            Directory.Delete(_tempDir, true);
        }

        GC.SuppressFinalize(this);
    }

    [Fact]
    public void YamlProvider_LoadsFlatKeys()
    {
        string yamlPath = Path.Combine(_tempDir, "config", "default.yaml");
        File.WriteAllText(yamlPath, "server:\n  port: 8080\n  host: localhost\n");

        var provider = new YamlConfigurationProvider(new YamlConfigurationSource { Path = yamlPath });
        provider.Load();

        provider.TryGet("server:port", out string? port).Should().BeTrue();
        port.Should().Be("8080");
        provider.TryGet("server:host", out string? host).Should().BeTrue();
        host.Should().Be("localhost");
    }

    [Fact]
    public void YamlProvider_OptionalMissingFile_DoesNotThrow()
    {
        var provider = new YamlConfigurationProvider(
            new YamlConfigurationSource { Path = "/nonexistent.yaml", Optional = true });

        var act = () => provider.Load();

        act.Should().NotThrow();
    }

    [Fact]
    public void YamlProvider_RequiredMissingFile_Throws()
    {
        var provider = new YamlConfigurationProvider(
            new YamlConfigurationSource { Path = "/nonexistent.yaml", Optional = false });

        var act = () => provider.Load();

        act.Should().Throw<FileNotFoundException>();
    }

    [Fact]
    public void AddK1s0YamlConfig_LoadsDefaultYaml()
    {
        string yamlPath = Path.Combine(_tempDir, "config", "default.yaml");
        File.WriteAllText(yamlPath, "app:\n  name: test-service\n");

        string originalDir = Directory.GetCurrentDirectory();
        try
        {
            Directory.SetCurrentDirectory(_tempDir);

            var config = new ConfigurationBuilder()
                .AddK1s0YamlConfig([])
                .Build();

            config["app:name"].Should().Be("test-service");
        }
        finally
        {
            Directory.SetCurrentDirectory(originalDir);
        }
    }

    [Fact]
    public void AddK1s0YamlConfig_EnvOverridesDefault()
    {
        File.WriteAllText(
            Path.Combine(_tempDir, "config", "default.yaml"),
            "db:\n  host: localhost\n  port: 5432\n");
        File.WriteAllText(
            Path.Combine(_tempDir, "config", "dev.yaml"),
            "db:\n  host: dev-db.internal\n");

        string originalDir = Directory.GetCurrentDirectory();
        try
        {
            Directory.SetCurrentDirectory(_tempDir);

            var config = new ConfigurationBuilder()
                .AddK1s0YamlConfig(["--env", "dev"])
                .Build();

            config["db:host"].Should().Be("dev-db.internal");
            config["db:port"].Should().Be("5432");
        }
        finally
        {
            Directory.SetCurrentDirectory(originalDir);
        }
    }

    [Fact]
    public void AddK1s0YamlConfig_LoadsSecrets()
    {
        File.WriteAllText(
            Path.Combine(_tempDir, "config", "default.yaml"),
            "app:\n  name: test\n");

        string secretsDir = Path.Combine(_tempDir, "secrets");
        Directory.CreateDirectory(secretsDir);
        File.WriteAllText(Path.Combine(secretsDir, "db_password"), "s3cret");

        string originalDir = Directory.GetCurrentDirectory();
        try
        {
            Directory.SetCurrentDirectory(_tempDir);

            var config = new ConfigurationBuilder()
                .AddK1s0YamlConfig(["--secrets-dir", secretsDir])
                .Build();

            config["db_password"].Should().Be("s3cret");
        }
        finally
        {
            Directory.SetCurrentDirectory(originalDir);
        }
    }

    [Fact]
    public void YamlProvider_HandlesSequences()
    {
        string yamlPath = Path.Combine(_tempDir, "config", "default.yaml");
        File.WriteAllText(yamlPath, "tags:\n  - alpha\n  - beta\n");

        var provider = new YamlConfigurationProvider(new YamlConfigurationSource { Path = yamlPath });
        provider.Load();

        provider.TryGet("tags:0", out string? first).Should().BeTrue();
        first.Should().Be("alpha");
        provider.TryGet("tags:1", out string? second).Should().BeTrue();
        second.Should().Be("beta");
    }
}
