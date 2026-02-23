import Testing
@testable import K1s0FileClient

@Suite("FileClient Tests")
struct FileClientTests {
    @Test("アップロードURLを生成できること")
    func testGenerateUploadUrl() async throws {
        let client = InMemoryFileClient()
        let url = try await client.generateUploadUrl(
            path: "uploads/test.png",
            contentType: "image/png",
            expiresIn: .seconds(3600)
        )
        #expect(url.url.contains("uploads/test.png"))
        #expect(url.method == "PUT")
    }

    @Test("ダウンロードURLを生成できること")
    func testGenerateDownloadUrl() async throws {
        let client = InMemoryFileClient()
        _ = try await client.generateUploadUrl(
            path: "uploads/test.png",
            contentType: "image/png",
            expiresIn: .seconds(3600)
        )
        let url = try await client.generateDownloadUrl(
            path: "uploads/test.png",
            expiresIn: .seconds(300)
        )
        #expect(url.url.contains("uploads/test.png"))
        #expect(url.method == "GET")
    }

    @Test("存在しないファイルのダウンロードURLでエラー")
    func testDownloadUrlNotFound() async {
        let client = InMemoryFileClient()
        await #expect(throws: FileClientError.self) {
            try await client.generateDownloadUrl(
                path: "nonexistent.txt",
                expiresIn: .seconds(300)
            )
        }
    }

    @Test("ファイルを削除できること")
    func testDelete() async throws {
        let client = InMemoryFileClient()
        _ = try await client.generateUploadUrl(
            path: "uploads/test.png",
            contentType: "image/png",
            expiresIn: .seconds(3600)
        )
        try await client.delete(path: "uploads/test.png")
        await #expect(throws: FileClientError.self) {
            try await client.getMetadata(path: "uploads/test.png")
        }
    }

    @Test("メタデータを取得できること")
    func testGetMetadata() async throws {
        let client = InMemoryFileClient()
        _ = try await client.generateUploadUrl(
            path: "uploads/test.png",
            contentType: "image/png",
            expiresIn: .seconds(3600)
        )
        let meta = try await client.getMetadata(path: "uploads/test.png")
        #expect(meta.path == "uploads/test.png")
        #expect(meta.contentType == "image/png")
    }

    @Test("プレフィックスで一覧取得できること")
    func testList() async throws {
        let client = InMemoryFileClient()
        _ = try await client.generateUploadUrl(path: "uploads/a.png", contentType: "image/png", expiresIn: .seconds(3600))
        _ = try await client.generateUploadUrl(path: "uploads/b.jpg", contentType: "image/jpeg", expiresIn: .seconds(3600))
        _ = try await client.generateUploadUrl(path: "other/c.txt", contentType: "text/plain", expiresIn: .seconds(3600))
        let files = try await client.list(prefix: "uploads/")
        #expect(files.count == 2)
    }

    @Test("ファイルをコピーできること")
    func testCopy() async throws {
        let client = InMemoryFileClient()
        _ = try await client.generateUploadUrl(
            path: "uploads/test.png",
            contentType: "image/png",
            expiresIn: .seconds(3600)
        )
        try await client.copy(src: "uploads/test.png", dst: "archive/test.png")
        let meta = try await client.getMetadata(path: "archive/test.png")
        #expect(meta.contentType == "image/png")
        #expect(meta.path == "archive/test.png")
    }

    @Test("存在しないファイルのコピーでエラー")
    func testCopyNotFound() async {
        let client = InMemoryFileClient()
        await #expect(throws: FileClientError.self) {
            try await client.copy(src: "nonexistent.txt", dst: "dest.txt")
        }
    }

    @Test("初期状態でファイルが空であること")
    func testStoredFilesEmpty() async {
        let client = InMemoryFileClient()
        let files = await client.storedFiles()
        #expect(files.isEmpty)
    }
}
