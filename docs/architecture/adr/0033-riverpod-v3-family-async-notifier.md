# ADR-0033: Riverpod v3 FamilyNotifier → FamilyAsyncNotifier 移行

## ステータス

採用済み (2026-03-25)

## 背景

Riverpod v3 において、非同期状態を管理する `Notifier` パターンに関するAPIが整理された。
`FamilyNotifier<AsyncValue<T>, Arg>` というパターンは Riverpod v3 では非推奨のアンチパターンとなり、
`FamilyAsyncNotifier<T, Arg>` への移行が推奨される。

影響を受けたファイル:
- `regions/service/board/client/flutter/board/lib/providers/board_provider.dart`

### 旧パターン（アンチパターン）

```dart
class BoardColumnListNotifier
    extends FamilyNotifier<AsyncValue<List<BoardColumn>>, String> {
  @override
  AsyncValue<List<BoardColumn>> build(String projectId) {
    load(projectId);
    return const AsyncValue.loading(); // 手動ラップが必要
  }
}

final boardColumnListProvider = NotifierProviderFamily<
    BoardColumnListNotifier, AsyncValue<List<BoardColumn>>, String>(
  BoardColumnListNotifier.new,
);
```

### 新パターン（推奨）

```dart
class BoardColumnListNotifier
    extends FamilyAsyncNotifier<List<BoardColumn>, String> {
  @override
  FutureOr<List<BoardColumn>> build(String projectId) async {
    return await _repository.listColumns(projectId); // 自動ラップ
  }
}

final boardColumnListProvider = AsyncNotifierProviderFamily<
    BoardColumnListNotifier, List<BoardColumn>, String>(
  BoardColumnListNotifier.new,
);
```

## 決定内容

`FamilyNotifier<AsyncValue<T>, Arg>` パターンを廃止し、`FamilyAsyncNotifier<T, Arg>` に完全移行する。

## 理由

1. **コードの簡潔性**: `build()` メソッドが `FutureOr<T>` を返すことで、手動での `AsyncValue` ラップが不要になる
2. **型安全性**: Riverpod が自動で `AsyncValue<T>` にラップするため、型の不整合が起きにくい
3. **エラーハンドリング**: `AsyncValue.guard()` による一貫したエラーハンドリングパターンが利用可能
4. **後方互換性**: コンシューマー側（`ref.watch()` で `AsyncValue<T>` を受け取る部分）はそのまま動作する

## 影響

- **コンシューマーコードへの影響なし**: `ref.watch(boardColumnListProvider(id))` は依然として `AsyncValue<List<BoardColumn>>` を返す
- `NotifierProviderFamily` → `AsyncNotifierProviderFamily` への変更が必要
- `FamilyNotifier` → `FamilyAsyncNotifier` への継承変更が必要
- `build()` の戻り値型が `AsyncValue<T>` → `FutureOr<T>` に変更

## 代替案

- **旧パターンを維持する**: Riverpod v3 のlintルールで警告が出るため却下
- **StateNotifier を使用する**: Riverpod v2 以前のパターンであり、v3 では非推奨のため却下

## 参考資料

- [Riverpod v3 migration guide](https://riverpod.dev/docs/migration/from_riverpod_0_14_to_riverpod_1)
- 影響ファイル: `regions/service/board/client/flutter/board/lib/providers/board_provider.dart`
