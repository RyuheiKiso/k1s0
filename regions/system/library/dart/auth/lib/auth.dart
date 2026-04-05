/// k1s0 OAuth2 PKCE 認証ライブラリ
///
/// クライアント側で OAuth2 Authorization Code + PKCE フローを実装する。
/// H-010 監査対応: TokenStorage 抽象インターフェースを公開し、
/// Flutter/pure Dart 両環境で依存を注入できるようにする。
library;

export 'src/auth_client.dart';
export 'src/device_flow.dart';
export 'src/pkce.dart';
export 'src/storage/token_storage.dart';
export 'src/token_store.dart';
export 'src/types.dart';
