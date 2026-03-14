/// building_blocks ライブラリのエントリポイント。
/// 各コンポーネント（PubSub、StateStore、SecretStore、Binding）の
/// インターフェース定義とインメモリ実装をまとめてエクスポートする。
library building_blocks;

/// コンポーネントの基底インターフェースと共通型を提供する。
export 'src/component.dart';

/// PubSub（発行/購読）コンポーネントのインターフェースを提供する。
export 'src/pubsub.dart';

/// StateStore（状態ストア）コンポーネントのインターフェースを提供する。
export 'src/statestore.dart';

/// SecretStore（シークレットストア）コンポーネントのインターフェースを提供する。
export 'src/secretstore.dart';

/// Binding（入出力バインディング）コンポーネントのインターフェースを提供する。
export 'src/binding.dart';

/// コンポーネント共通のエラー型を提供する。
export 'src/errors.dart';

/// コンポーネントの設定型を提供する。
export 'src/config.dart';

/// テスト・開発用のインメモリ PubSub 実装を提供する。
export 'src/inmemory_pubsub.dart';

/// テスト・開発用のインメモリ StateStore 実装を提供する。
export 'src/inmemory_statestore.dart';

/// テスト・開発用のインメモリ SecretStore 実装を提供する。
export 'src/inmemory_secretstore.dart';

/// テスト・開発用のインメモリ Binding 実装を提供する。
export 'src/inmemory_binding.dart';

/// コンポーネントの登録・管理を行う ComponentRegistry を提供する。
export 'src/registry.dart';
