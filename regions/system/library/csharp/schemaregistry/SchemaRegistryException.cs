namespace K1s0.System.SchemaRegistry;

public class SchemaRegistryException : Exception
{
    public string Code { get; }

    public SchemaRegistryException(string code, string message)
        : base(message)
    {
        Code = code;
    }

    public SchemaRegistryException(string code, string message, Exception innerException)
        : base(message, innerException)
    {
        Code = code;
    }

    public static class ErrorCodes
    {
        public const string RegistrationFailed = "REGISTRATION_FAILED";
        public const string SchemaNotFound = "SCHEMA_NOT_FOUND";
        public const string CompatibilityCheckFailed = "COMPATIBILITY_CHECK_FAILED";
        public const string ConnectionFailed = "CONNECTION_FAILED";
    }
}
