import 'dart:convert';

import 'package:shared_preferences/shared_preferences.dart';

/// SharedPreferences を使用した JSON ストレージラッパー
class RealtimeStorage {
  SharedPreferences? _prefs;

  /// SharedPreferences を初期化する
  Future<void> init() async {
    _prefs ??= await SharedPreferences.getInstance();
  }

  /// JSON データを取得する
  T? get<T>(String key, T Function(Map<String, dynamic>) fromJson) {
    final raw = _prefs?.getString(key);
    if (raw == null) return null;
    try {
      return fromJson(jsonDecode(raw) as Map<String, dynamic>);
    } catch (_) {
      return null;
    }
  }

  /// 生の JSON 文字列を取得する
  String? getRaw(String key) {
    return _prefs?.getString(key);
  }

  /// JSON データを保存する
  Future<bool> set(String key, Object value) async {
    await init();
    return _prefs!.setString(key, jsonEncode(value));
  }

  /// データを削除する
  Future<bool> remove(String key) async {
    await init();
    return _prefs!.remove(key);
  }
}
