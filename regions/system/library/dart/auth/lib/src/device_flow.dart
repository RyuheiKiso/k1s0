/// Device Authorization Grant フロー（RFC 8628）のクライアント実装。
/// CLI やスマートデバイス等、ブラウザリダイレクトが困難な環境向け。

import 'dart:convert';

import 'package:http/http.dart' as http;

/// DeviceCodeResponse はデバイス認可リクエストのレスポンス。
class DeviceCodeResponse {
  final String deviceCode;
  final String userCode;
  final String verificationUri;
  final String? verificationUriComplete;
  final int expiresIn;
  final int interval;

  DeviceCodeResponse({
    required this.deviceCode,
    required this.userCode,
    required this.verificationUri,
    this.verificationUriComplete,
    required this.expiresIn,
    required this.interval,
  });

  factory DeviceCodeResponse.fromJson(Map<String, dynamic> json) =>
      DeviceCodeResponse(
        deviceCode: json['device_code'] as String,
        userCode: json['user_code'] as String,
        verificationUri: json['verification_uri'] as String,
        verificationUriComplete: json['verification_uri_complete'] as String?,
        expiresIn: json['expires_in'] as int,
        interval: json['interval'] as int,
      );
}

/// DeviceTokenResponse はトークンエンドポイントのレスポンス。
class DeviceTokenResponse {
  final String accessToken;
  final String? refreshToken;
  final String tokenType;
  final int expiresIn;

  DeviceTokenResponse({
    required this.accessToken,
    this.refreshToken,
    required this.tokenType,
    required this.expiresIn,
  });

  factory DeviceTokenResponse.fromJson(Map<String, dynamic> json) =>
      DeviceTokenResponse(
        accessToken: json['access_token'] as String,
        refreshToken: json['refresh_token'] as String?,
        tokenType: json['token_type'] as String,
        expiresIn: json['expires_in'] as int,
      );
}

/// DeviceFlowError は Device Authorization Grant フローのエラー。
class DeviceFlowError implements Exception {
  final String errorCode;
  final String? description;

  DeviceFlowError(this.errorCode, [this.description]);

  @override
  String toString() {
    if (description != null) {
      return 'DeviceFlowError: $errorCode ($description)';
    }
    return 'DeviceFlowError: $errorCode';
  }
}

/// デバイスコードコールバック。
typedef DeviceCodeCallback = void Function(DeviceCodeResponse resp);

/// HTTP クライアントの抽象化（テスト用に注入可能）。
typedef DeviceFlowHttpPost = Future<http.Response> Function(
  Uri url, {
  Map<String, String>? headers,
  Object? body,
});

/// DeviceAuthClient のオプション。
class DeviceAuthClientOptions {
  final String deviceEndpoint;
  final String tokenEndpoint;
  final DeviceFlowHttpPost? httpPost;

  DeviceAuthClientOptions({
    required this.deviceEndpoint,
    required this.tokenEndpoint,
    this.httpPost,
  });
}

/// DeviceAuthClient は Device Authorization Grant フロー（RFC 8628）のクライアント。
class DeviceAuthClient {
  final String _deviceEndpoint;
  final String _tokenEndpoint;
  final DeviceFlowHttpPost _httpPost;

  DeviceAuthClient(DeviceAuthClientOptions options)
      : _deviceEndpoint = options.deviceEndpoint,
        _tokenEndpoint = options.tokenEndpoint,
        _httpPost = options.httpPost ?? _defaultPost;

  static Future<http.Response> _defaultPost(
    Uri url, {
    Map<String, String>? headers,
    Object? body,
  }) {
    return http.post(url, headers: headers, body: body);
  }

  /// デバイス認可リクエストを送信し、デバイスコード情報を返す。
  Future<DeviceCodeResponse> requestDeviceCode(
    String clientId, [
    String? scope,
  ]) async {
    final body = <String, String>{
      'client_id': clientId,
    };
    if (scope != null) {
      body['scope'] = scope;
    }

    final resp = await _httpPost(
      Uri.parse(_deviceEndpoint),
      headers: {'Content-Type': 'application/x-www-form-urlencoded'},
      body: body,
    );

    if (resp.statusCode != 200) {
      throw DeviceFlowError(
        'request_failed',
        'Device code request failed: ${resp.statusCode}',
      );
    }

    return DeviceCodeResponse.fromJson(
      jsonDecode(resp.body) as Map<String, dynamic>,
    );
  }

  /// device_code を使ってトークンエンドポイントをポーリングする。
  /// interval が 0 の場合はデフォルトの 5 秒を使用する。
  /// [cancelled] を完了させるとポーリングを中止できる。
  Future<DeviceTokenResponse> pollToken(
    String clientId,
    String deviceCode,
    int interval, {
    Future<void>? cancelled,
  }) async {
    var intervalSecs = interval <= 0 ? 5 : interval;

    while (true) {
      final resp = await _httpPost(
        Uri.parse(_tokenEndpoint),
        headers: {'Content-Type': 'application/x-www-form-urlencoded'},
        body: {
          'grant_type': 'urn:ietf:params:oauth:grant-type:device_code',
          'device_code': deviceCode,
          'client_id': clientId,
        },
      );

      if (resp.statusCode == 200) {
        return DeviceTokenResponse.fromJson(
          jsonDecode(resp.body) as Map<String, dynamic>,
        );
      }

      final errBody =
          jsonDecode(resp.body) as Map<String, dynamic>;
      final errorCode = errBody['error'] as String;

      switch (errorCode) {
        case 'authorization_pending':
          break;
        case 'slow_down':
          intervalSecs += 5;
          break;
        case 'expired_token':
          throw DeviceFlowError('expired_token', 'Device code has expired');
        case 'access_denied':
          throw DeviceFlowError(
            'access_denied',
            'User denied the authorization request',
          );
        default:
          throw DeviceFlowError(
            errorCode,
            errBody['error_description'] as String?,
          );
      }

      // interval 待機（キャンセル対応）
      if (cancelled != null) {
        final result = await Future.any([
          Future.delayed(Duration(seconds: intervalSecs)).then((_) => false),
          cancelled.then((_) => true),
        ]);
        if (result) {
          throw DeviceFlowError('cancelled', 'Polling was cancelled');
        }
      } else {
        await Future.delayed(Duration(seconds: intervalSecs));
      }
    }
  }

  /// Device Authorization Grant フロー全体を実行する統合メソッド。
  /// [onUserCode] コールバックでユーザーにデバイスコード情報を通知する。
  Future<DeviceTokenResponse> deviceFlow(
    String clientId,
    String? scope,
    DeviceCodeCallback onUserCode, {
    Future<void>? cancelled,
  }) async {
    final deviceResp = await requestDeviceCode(clientId, scope);
    onUserCode(deviceResp);
    return pollToken(
      clientId,
      deviceResp.deviceCode,
      deviceResp.interval,
      cancelled: cancelled,
    );
  }
}
