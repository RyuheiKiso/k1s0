import 'package:test/test.dart';
import 'package:k1s0_building_blocks/building_blocks.dart';

// InMemorySecretStore のテストエントリポイント。
void main() {
  // InMemorySecretStore のテスト: シークレットの保存・取得・一括取得・上書きの動作を検証する。
  group('InMemorySecretStore', () {
    late InMemorySecretStore store;

    setUp(() {
      store = InMemorySecretStore();
    });

    test('初期状態は uninitialized', () async {
      expect(await store.status(), ComponentStatus.uninitialized);
    });

    test('init 後は ready になる', () async {
      await store.init();
      expect(await store.status(), ComponentStatus.ready);
    });

    test('close 後は closed になりシークレットがクリアされる', () async {
      await store.init();
      store.put('k', 'v');
      await store.close();
      expect(await store.status(), ComponentStatus.closed);
      expect(
        () => store.getSecret('k'),
        throwsA(isA<ComponentError>()),
      );
    });

    test('デフォルト name は inmemory-secretstore', () {
      expect(store.name, 'inmemory-secretstore');
      expect(store.componentType, 'secretstore');
    });

    test('コンストラクタで name を指定できる', () {
      final named = InMemorySecretStore(name: 'custom-secrets');
      expect(named.name, 'custom-secrets');
    });

    test('metadata は backend=memory を返す', () {
      expect(store.metadata(), {'backend': 'memory'});
    });

    test('put したシークレットを getSecret で取得できる', () async {
      await store.init();
      store.put('db-password', 'secret123');

      final secret = await store.getSecret('db-password');

      expect(secret.key, 'db-password');
      expect(secret.value, 'secret123');
    });

    test('存在しないキーの getSecret は ComponentError をスローする', () async {
      await store.init();
      expect(
        () => store.getSecret('missing'),
        throwsA(isA<ComponentError>()),
      );
    });

    test('bulkGet で複数シークレットを一括取得できる', () async {
      await store.init();
      store.put('k1', 'v1');
      store.put('k2', 'v2');

      final result = await store.bulkGet(['k1', 'k2']);

      expect(result['k1']!.value, 'v1');
      expect(result['k2']!.value, 'v2');
    });

    test('bulkGet でいずれかのキーが存在しない場合は ComponentError をスローする', () async {
      await store.init();
      store.put('k1', 'v1');

      expect(
        () => store.bulkGet(['k1', 'missing']),
        throwsA(isA<ComponentError>()),
      );
    });

    test('put で既存の値を上書きできる', () async {
      await store.init();
      store.put('key', 'old');
      store.put('key', 'new');

      final secret = await store.getSecret('key');
      expect(secret.value, 'new');
    });
  });
}
