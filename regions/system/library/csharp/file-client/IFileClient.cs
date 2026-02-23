namespace K1s0.System.FileClient;

public interface IFileClient
{
    Task<PresignedUrl> GenerateUploadUrlAsync(
        string path,
        string contentType,
        TimeSpan expiresIn,
        CancellationToken ct = default);

    Task<PresignedUrl> GenerateDownloadUrlAsync(
        string path,
        TimeSpan expiresIn,
        CancellationToken ct = default);

    Task DeleteAsync(string path, CancellationToken ct = default);

    Task<FileMetadata> GetMetadataAsync(string path, CancellationToken ct = default);

    Task<IReadOnlyList<FileMetadata>> ListAsync(string prefix, CancellationToken ct = default);

    Task CopyAsync(string src, string dst, CancellationToken ct = default);
}
