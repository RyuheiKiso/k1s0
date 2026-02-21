class ServiceAuthConfig {
  final String tokenEndpoint;
  final String clientId;
  final String clientSecret;

  const ServiceAuthConfig({
    required this.tokenEndpoint,
    required this.clientId,
    required this.clientSecret,
  });
}
