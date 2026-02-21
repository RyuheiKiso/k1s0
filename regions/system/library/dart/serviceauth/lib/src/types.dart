import 'error.dart';

class ServiceClaims {
  final String sub;
  final String iss;
  final String? scope;
  final int? exp;

  const ServiceClaims({
    required this.sub,
    required this.iss,
    this.scope,
    this.exp,
  });
}

/// SPIFFE URI: spiffe://<trust-domain>/ns/<namespace>/sa/<service-account>
class SpiffeId {
  final String trustDomain;
  final String namespace;
  final String serviceAccount;
  final String uri;

  const SpiffeId({
    required this.trustDomain,
    required this.namespace,
    required this.serviceAccount,
    required this.uri,
  });

  @override
  String toString() =>
      'spiffe://$trustDomain/ns/$namespace/sa/$serviceAccount';
}

SpiffeId parseSpiffeId(String uri) {
  if (!uri.startsWith('spiffe://')) {
    throw ServiceAuthError('invalid SPIFFE ID: must start with spiffe://');
  }

  final rest = uri.substring('spiffe://'.length);
  final slashIndex = rest.indexOf('/');
  if (slashIndex < 0) {
    throw ServiceAuthError('invalid SPIFFE ID format: $uri');
  }

  final trustDomain = rest.substring(0, slashIndex);
  final path = rest.substring(slashIndex + 1);
  final segments = path.split('/');

  // segments: ["ns", "<ns>", "sa", "<sa>"]
  if (segments.length < 4 || segments[0] != 'ns' || segments[2] != 'sa') {
    throw ServiceAuthError(
        'invalid SPIFFE ID path (expected /ns/<ns>/sa/<sa>): /$path');
  }

  return SpiffeId(
    trustDomain: trustDomain,
    namespace: segments[1],
    serviceAccount: segments[3],
    uri: uri,
  );
}

SpiffeId validateSpiffeId(String uri, String expectedNamespace) {
  final spiffe = parseSpiffeId(uri);
  if (spiffe.namespace != expectedNamespace) {
    throw ServiceAuthError(
        'namespace mismatch: expected $expectedNamespace, got ${spiffe.namespace}');
  }
  return spiffe;
}
