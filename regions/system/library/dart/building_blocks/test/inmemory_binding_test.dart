import 'dart:typed_data';
import 'package:test/test.dart';
import 'package:k1s0_building_blocks/building_blocks.dart';

void main() {
  group('InMemoryInputBinding', () {
    late InMemoryInputBinding binding;

    setUp(() {
      binding = InMemoryInputBinding();
    });

    test('初期状態は uninitialized', () async {
      expect(await binding.status(), ComponentStatus.uninitialized);
    });

    test('init 後は ready になる', () async {
      await binding.init();
      expect(await binding.status(), ComponentStatus.ready);
    });

    test('close 後は closed になる', () async {
      await binding.init();
      await binding.close();
      expect(await binding.status(), ComponentStatus.closed);
    });

    test('デフォルト name は inmemory-input-binding', () {
      expect(binding.name, 'inmemory-input-binding');
      expect(binding.componentType, 'binding.input');
    });

    test('コンストラクタで name を指定できる', () {
      final named = InMemoryInputBinding(name: 'custom-input');
      expect(named.name, 'custom-input');
    });

    test('metadata は backend=memory と direction=input を返す', () {
      expect(binding.metadata(), {'backend': 'memory', 'direction': 'input'});
    });

    test('push したデータを read で FIFO 順に取得できる', () async {
      await binding.init();
      binding.push(BindingData(data: Uint8List.fromList([1]), metadata: {'seq': '1'}));
      binding.push(BindingData(data: Uint8List.fromList([2]), metadata: {'seq': '2'}));

      final first = await binding.read();
      final second = await binding.read();

      expect(first.data, Uint8List.fromList([1]));
      expect(first.metadata['seq'], '1');
      expect(second.data, Uint8List.fromList([2]));
    });

    test('キューが空のときに read すると ComponentError をスローする', () async {
      await binding.init();
      expect(
        () => binding.read(),
        throwsA(isA<ComponentError>()),
      );
    });
  });

  group('InMemoryOutputBinding', () {
    late InMemoryOutputBinding binding;

    setUp(() {
      binding = InMemoryOutputBinding();
    });

    test('初期状態は uninitialized', () async {
      expect(await binding.status(), ComponentStatus.uninitialized);
    });

    test('init 後は ready になる', () async {
      await binding.init();
      expect(await binding.status(), ComponentStatus.ready);
    });

    test('close 後は closed になり呼び出し履歴がクリアされる', () async {
      await binding.init();
      await binding.invoke('op', Uint8List.fromList([1]));
      await binding.close();
      expect(await binding.status(), ComponentStatus.closed);
      expect(binding.lastInvocation(), isNull);
    });

    test('デフォルト name は inmemory-output-binding', () {
      expect(binding.name, 'inmemory-output-binding');
      expect(binding.componentType, 'binding.output');
    });

    test('コンストラクタで name を指定できる', () {
      final named = InMemoryOutputBinding(name: 'custom-output');
      expect(named.name, 'custom-output');
    });

    test('metadata は backend=memory と direction=output を返す', () {
      expect(binding.metadata(), {'backend': 'memory', 'direction': 'output'});
    });

    test('invoke 前は lastInvocation が null', () async {
      await binding.init();
      expect(binding.lastInvocation(), isNull);
    });

    test('invoke が呼び出し履歴を記録する', () async {
      await binding.init();
      await binding.invoke(
        'send',
        Uint8List.fromList([1, 2]),
        metadata: {'key': 'val'},
      );

      final inv = binding.lastInvocation();
      expect(inv, isNotNull);
      expect(inv!.operation, 'send');
      expect(inv.data, Uint8List.fromList([1, 2]));
      expect(inv.metadata['key'], 'val');
    });

    test('invoke はデフォルトで入力データをそのまま返す', () async {
      await binding.init();
      final resp = await binding.invoke('op', Uint8List.fromList([42]));
      expect(resp.data, Uint8List.fromList([42]));
    });

    test('setResponse でモックレスポンスを設定できる', () async {
      await binding.init();
      binding.setResponse(response: BindingResponse(data: Uint8List.fromList([99]), metadata: {}));
      final resp = await binding.invoke('op', Uint8List.fromList([1]));
      expect(resp.data, Uint8List.fromList([99]));
    });

    test('setResponse でモックエラーを設定できる', () async {
      await binding.init();
      final mockError = Exception('invoke error');
      binding.setResponse(error: mockError);
      expect(
        () => binding.invoke('op', Uint8List.fromList([1])),
        throwsA(equals(mockError)),
      );
    });

    test('allInvocations で全履歴を取得できる', () async {
      await binding.init();
      await binding.invoke('op1', Uint8List.fromList([1]));
      await binding.invoke('op2', Uint8List.fromList([2]));

      final all = binding.allInvocations();
      expect(all, hasLength(2));
      expect(all[0].operation, 'op1');
      expect(all[1].operation, 'op2');
    });

    test('reset で履歴とモック設定をクリアできる', () async {
      await binding.init();
      binding.setResponse(response: BindingResponse(data: Uint8List.fromList([99]), metadata: {}));
      await binding.invoke('op', Uint8List.fromList([1]));

      binding.reset();

      expect(binding.lastInvocation(), isNull);
      // reset 後はデフォルト動作（入力をそのまま返す）に戻る
      final resp = await binding.invoke('op', Uint8List.fromList([7]));
      expect(resp.data, Uint8List.fromList([7]));
    });
  });
}
