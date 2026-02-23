class VaultClientConfig {
  final String serverUrl;
  final Duration cacheTtl;
  final int cacheMaxCapacity;

  const VaultClientConfig({
    required this.serverUrl,
    this.cacheTtl = const Duration(minutes: 10),
    this.cacheMaxCapacity = 500,
  });
}
