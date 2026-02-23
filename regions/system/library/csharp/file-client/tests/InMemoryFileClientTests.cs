using K1s0.System.FileClient;

namespace K1s0.System.FileClient.Tests;

public class InMemoryFileClientTests
{
    [Fact]
    public async Task GenerateUploadUrl_ReturnsUrl()
    {
        var client = new InMemoryFileClient();
        var url = await client.GenerateUploadUrlAsync("uploads/test.png", "image/png", TimeSpan.FromHours(1));
        Assert.Contains("uploads/test.png", url.Url);
        Assert.Equal("PUT", url.Method);
    }

    [Fact]
    public async Task GenerateDownloadUrl_ReturnsUrl()
    {
        var client = new InMemoryFileClient();
        await client.GenerateUploadUrlAsync("uploads/test.png", "image/png", TimeSpan.FromHours(1));
        var url = await client.GenerateDownloadUrlAsync("uploads/test.png", TimeSpan.FromMinutes(5));
        Assert.Contains("uploads/test.png", url.Url);
        Assert.Equal("GET", url.Method);
    }

    [Fact]
    public async Task GenerateDownloadUrl_ThrowsForNonExistent()
    {
        var client = new InMemoryFileClient();
        await Assert.ThrowsAsync<FileClientException>(
            () => client.GenerateDownloadUrlAsync("nonexistent.txt", TimeSpan.FromMinutes(5)));
    }

    [Fact]
    public async Task Delete_RemovesFile()
    {
        var client = new InMemoryFileClient();
        await client.GenerateUploadUrlAsync("uploads/test.png", "image/png", TimeSpan.FromHours(1));
        await client.DeleteAsync("uploads/test.png");
        await Assert.ThrowsAsync<FileClientException>(
            () => client.GetMetadataAsync("uploads/test.png"));
    }

    [Fact]
    public async Task GetMetadata_ReturnsMetadata()
    {
        var client = new InMemoryFileClient();
        await client.GenerateUploadUrlAsync("uploads/test.png", "image/png", TimeSpan.FromHours(1));
        var meta = await client.GetMetadataAsync("uploads/test.png");
        Assert.Equal("uploads/test.png", meta.Path);
        Assert.Equal("image/png", meta.ContentType);
    }

    [Fact]
    public async Task List_ReturnsMatchingFiles()
    {
        var client = new InMemoryFileClient();
        await client.GenerateUploadUrlAsync("uploads/a.png", "image/png", TimeSpan.FromHours(1));
        await client.GenerateUploadUrlAsync("uploads/b.jpg", "image/jpeg", TimeSpan.FromHours(1));
        await client.GenerateUploadUrlAsync("other/c.txt", "text/plain", TimeSpan.FromHours(1));
        var files = await client.ListAsync("uploads/");
        Assert.Equal(2, files.Count);
    }

    [Fact]
    public async Task Copy_CopiesFile()
    {
        var client = new InMemoryFileClient();
        await client.GenerateUploadUrlAsync("uploads/test.png", "image/png", TimeSpan.FromHours(1));
        await client.CopyAsync("uploads/test.png", "archive/test.png");
        var meta = await client.GetMetadataAsync("archive/test.png");
        Assert.Equal("image/png", meta.ContentType);
        Assert.Equal("archive/test.png", meta.Path);
    }

    [Fact]
    public async Task Copy_ThrowsForNonExistent()
    {
        var client = new InMemoryFileClient();
        await Assert.ThrowsAsync<FileClientException>(
            () => client.CopyAsync("nonexistent.txt", "dest.txt"));
    }

    [Fact]
    public void StoredFiles_InitiallyEmpty()
    {
        var client = new InMemoryFileClient();
        Assert.Empty(client.StoredFiles);
    }
}
