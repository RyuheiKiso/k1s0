namespace K1s0.System.FileClient;

public class InMemoryFileClient : IFileClient
{
    private readonly Dictionary<string, FileMetadata> _files = new();

    public IReadOnlyDictionary<string, FileMetadata> StoredFiles =>
        new Dictionary<string, FileMetadata>(_files);

    public Task<PresignedUrl> GenerateUploadUrlAsync(
        string path,
        string contentType,
        TimeSpan expiresIn,
        CancellationToken ct = default)
    {
        _files[path] = new FileMetadata(
            path,
            0,
            contentType,
            string.Empty,
            DateTimeOffset.UtcNow,
            new Dictionary<string, string>());
        var url = new PresignedUrl(
            $"https://storage.example.com/upload/{path}",
            "PUT",
            DateTimeOffset.UtcNow.Add(expiresIn),
            new Dictionary<string, string>());
        return Task.FromResult(url);
    }

    public Task<PresignedUrl> GenerateDownloadUrlAsync(
        string path,
        TimeSpan expiresIn,
        CancellationToken ct = default)
    {
        if (!_files.ContainsKey(path))
        {
            throw new FileClientException($"File not found: {path}", "NOT_FOUND");
        }

        var url = new PresignedUrl(
            $"https://storage.example.com/download/{path}",
            "GET",
            DateTimeOffset.UtcNow.Add(expiresIn),
            new Dictionary<string, string>());
        return Task.FromResult(url);
    }

    public Task DeleteAsync(string path, CancellationToken ct = default)
    {
        if (!_files.Remove(path))
        {
            throw new FileClientException($"File not found: {path}", "NOT_FOUND");
        }

        return Task.CompletedTask;
    }

    public Task<FileMetadata> GetMetadataAsync(string path, CancellationToken ct = default)
    {
        if (!_files.TryGetValue(path, out var meta))
        {
            throw new FileClientException($"File not found: {path}", "NOT_FOUND");
        }

        return Task.FromResult(meta);
    }

    public Task<IReadOnlyList<FileMetadata>> ListAsync(string prefix, CancellationToken ct = default)
    {
        var result = _files.Values
            .Where(f => f.Path.StartsWith(prefix, StringComparison.Ordinal))
            .ToList();
        return Task.FromResult<IReadOnlyList<FileMetadata>>(result);
    }

    public Task CopyAsync(string src, string dst, CancellationToken ct = default)
    {
        if (!_files.TryGetValue(src, out var source))
        {
            throw new FileClientException($"File not found: {src}", "NOT_FOUND");
        }

        _files[dst] = source with { Path = dst };
        return Task.CompletedTask;
    }
}
