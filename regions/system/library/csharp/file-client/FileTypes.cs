namespace K1s0.System.FileClient;

public record FileMetadata(
    string Path,
    long SizeBytes,
    string ContentType,
    string ETag,
    DateTimeOffset LastModified,
    IReadOnlyDictionary<string, string> Tags);

public record PresignedUrl(
    string Url,
    string Method,
    DateTimeOffset ExpiresAt,
    IReadOnlyDictionary<string, string> Headers);

public class FileClientException : Exception
{
    public string Code { get; }

    public FileClientException(string message, string code)
        : base(message)
    {
        Code = code;
    }
}
