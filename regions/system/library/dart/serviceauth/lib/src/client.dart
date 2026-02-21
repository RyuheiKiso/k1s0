import 'dart:convert';

import 'package:http/http.dart' as http;

import 'types.dart';
import 'token.dart';
import 'config.dart';
import 'error.dart';

abstract class ServiceAuthClient {
  Future<ServiceToken> getToken();
  Future<String> getCachedToken();
  SpiffeId validateSpiffeIdCheck(String uri, String expectedNamespace);
}

class HttpServiceAuthClient implements ServiceAuthClient {
  final ServiceAuthConfig config;
  final http.Client _httpClient;
  ServiceToken? _cachedToken;

  HttpServiceAuthClient(this.config, {http.Client? httpClient})
      : _httpClient = httpClient ?? http.Client();

  @override
  Future<ServiceToken> getToken() async {
    final body = {
      'grant_type': 'client_credentials',
      'client_id': config.clientId,
      'client_secret': config.clientSecret,
    };

    final response = await _httpClient.post(
      Uri.parse(config.tokenEndpoint),
      headers: {'Content-Type': 'application/x-www-form-urlencoded'},
      body: body,
    );

    if (response.statusCode != 200) {
      throw ServiceAuthError(
        'token request failed (status ${response.statusCode}): ${response.body}',
      );
    }

    final json = jsonDecode(response.body) as Map<String, dynamic>;
    final expiresIn = json['expires_in'] as int;

    return ServiceToken(
      accessToken: json['access_token'] as String,
      tokenType: json['token_type'] as String,
      expiresAt: DateTime.now().add(Duration(seconds: expiresIn)),
      scope: json['scope'] as String?,
    );
  }

  @override
  Future<String> getCachedToken() async {
    if (_cachedToken == null || shouldRefresh(_cachedToken!)) {
      _cachedToken = await getToken();
    }
    return bearerHeader(_cachedToken!);
  }

  @override
  SpiffeId validateSpiffeIdCheck(String uri, String expectedNamespace) {
    return validateSpiffeId(uri, expectedNamespace);
  }
}
