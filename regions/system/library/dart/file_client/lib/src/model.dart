class FileMetadata {
  final String path;
  final int sizeBytes;
  final String contentType;
  final String etag;
  final DateTime lastModified;
  final Map<String, String> tags;

  const FileMetadata({
    required this.path,
    required this.sizeBytes,
    required this.contentType,
    required this.etag,
    required this.lastModified,
    required this.tags,
  });
}

class PresignedUrl {
  final String url;
  final String method;
  final DateTime expiresAt;
  final Map<String, String> headers;

  const PresignedUrl({
    required this.url,
    required this.method,
    required this.expiresAt,
    required this.headers,
  });
}

class FileClientError implements Exception {
  final String message;
  final String code;

  const FileClientError(this.message, this.code);

  @override
  String toString() => 'FileClientError($code): $message';
}
