namespace K1s0.System.Config;

public class ConfigException : Exception
{
    public string Code { get; }

    public ConfigException(string code, string message, Exception? inner = null)
        : base(message, inner)
    {
        Code = code;
    }
}

public static class ConfigErrorCodes
{
    public const string ReadFile = "READ_FILE_ERROR";
    public const string ParseYaml = "PARSE_YAML_ERROR";
    public const string Validation = "VALIDATION_ERROR";
}
