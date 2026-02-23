class TenantClientConfig {
  final String serverUrl;
  final Duration cacheTtl;
  final int cacheMaxCapacity;

  const TenantClientConfig({
    required this.serverUrl,
    this.cacheTtl = const Duration(minutes: 5),
    this.cacheMaxCapacity = 1000,
  });
}
