import 'dart:async';

import 'secret.dart';
import 'config.dart';
import 'error.dart';

abstract class VaultClient {
  Future<Secret> getSecret(String path);
  Future<String> getSecretValue(String path, String key);
  Future<List<String>> listSecrets(String pathPrefix);
  Stream<SecretRotatedEvent> watchSecret(String path);
}

class InMemoryVaultClient implements VaultClient {
  final VaultClientConfig config;
  final Map<String, Secret> _store = {};

  InMemoryVaultClient(this.config);

  void putSecret(Secret secret) {
    _store[secret.path] = secret;
  }

  @override
  Future<Secret> getSecret(String path) async {
    final secret = _store[path];
    if (secret == null) {
      throw VaultError(VaultErrorCode.notFound, path);
    }
    return secret;
  }

  @override
  Future<String> getSecretValue(String path, String key) async {
    final secret = await getSecret(path);
    final value = secret.data[key];
    if (value == null) {
      throw VaultError(VaultErrorCode.notFound, '$path/$key');
    }
    return value;
  }

  @override
  Future<List<String>> listSecrets(String pathPrefix) async {
    return _store.keys
        .where((k) => k.startsWith(pathPrefix))
        .toList();
  }

  @override
  Stream<SecretRotatedEvent> watchSecret(String path) {
    return const Stream.empty();
  }
}
