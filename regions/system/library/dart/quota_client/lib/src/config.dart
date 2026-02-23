class QuotaClientConfig {
  final String serverUrl;
  final Duration timeout;
  final Duration policyCacheTtl;

  const QuotaClientConfig({
    required this.serverUrl,
    this.timeout = const Duration(seconds: 5),
    this.policyCacheTtl = const Duration(seconds: 60),
  });
}
