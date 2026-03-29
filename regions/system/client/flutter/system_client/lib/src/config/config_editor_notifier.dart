import 'package:dio/dio.dart';
import 'package:flutter_riverpod/legacy.dart';

import 'config_interpreter.dart';
import 'config_types.dart';

/// 設定エディタの画面状態
/// ローディング・エラー・データ・保存中・コンフリクトの状態を不変オブジェクトで保持する
class ConfigEditorState {
  const ConfigEditorState({
    this.data,
    this.selectedCategoryId,
    this.isLoading = true,
    this.error,
    this.isSaving = false,
    this.hasConflict = false,
    // FE-08 対応: 部分失敗フラグ（一部の PUT に成功し残りが失敗した状態を示す）
    this.hasPartialFailure = false,
  });

  /// 設定データ（ロード完了後に設定される）
  final ConfigData? data;
  /// 現在選択中のカテゴリ ID
  final String? selectedCategoryId;
  /// ローディング中かどうか
  final bool isLoading;
  /// エラーメッセージ（エラー発生時のみ）
  final String? error;
  /// 保存中かどうか
  final bool isSaving;
  /// コンフリクトが検出されたかどうか
  final bool hasConflict;
  /// 逐次 PUT の途中でエラーが発生し部分的にしか保存できなかったかどうか
  final bool hasPartialFailure;

  /// 現在選択中のカテゴリの状態を返す
  ConfigCategoryState? get selectedCategory {
    if (data == null || selectedCategoryId == null) return null;
    return data!.categories
        .where((category) => category.schema.id == selectedCategoryId)
        .firstOrNull;
  }

  /// 変更されたフィールドが存在するかどうかを返す
  bool get hasDirtyFields => (data?.dirtyCount ?? 0) > 0;

  /// バリデーションエラーが存在するかどうかを返す
  bool get hasValidationErrors =>
      data?.categories.any(
        (category) => category.fields.any((field) => field.error != null),
      ) ??
      false;

  /// 一部のフィールドを更新した新しいインスタンスを返す
  ConfigEditorState copyWith({
    ConfigData? data,
    String? selectedCategoryId,
    bool? isLoading,
    String? error,
    bool? isSaving,
    bool? hasConflict,
    bool? hasPartialFailure,
    bool clearError = false,
    bool clearSelectedCategoryId = false,
  }) {
    return ConfigEditorState(
      data: data ?? this.data,
      selectedCategoryId: clearSelectedCategoryId
          ? null
          : (selectedCategoryId ?? this.selectedCategoryId),
      isLoading: isLoading ?? this.isLoading,
      error: clearError ? null : (error ?? this.error),
      isSaving: isSaving ?? this.isSaving,
      hasConflict: hasConflict ?? this.hasConflict,
      hasPartialFailure: hasPartialFailure ?? this.hasPartialFailure,
    );
  }
}

/// 設定エディタの状態管理を行う StateNotifier
/// ロード・フィールド変更・バリデーション・保存・リセットのロジックを集約する
class ConfigEditorNotifier extends StateNotifier<ConfigEditorState> {
  ConfigEditorNotifier({
    required this.dio,
    required this.serviceName,
  }) : super(const ConfigEditorState());

  final Dio dio;
  final String serviceName;

  /// API から設定スキーマと現在値を読み込む
  Future<void> load() async {
    state = state.copyWith(isLoading: true, clearError: true);

    try {
      final interpreter = ConfigInterpreter(dio: dio);
      final data = await interpreter.build(serviceName);
      state = state.copyWith(
        data: data,
        selectedCategoryId: data.categories.firstOrNull?.schema.id,
        isLoading: false,
      );
    } on DioException catch (e) {
      state = state.copyWith(
        error: e.message ?? 'Failed to load config',
        isLoading: false,
      );
    }
  }

  /// カテゴリを選択する
  void selectCategory(String id) {
    state = state.copyWith(selectedCategoryId: id);
  }

  /// フィールドのバリデーションエラーを更新する
  void onFieldValidationChanged(String key, String? error) {
    if (state.data == null || state.selectedCategoryId == null) return;

    final categories = state.data!.categories.map((category) {
      if (category.schema.id != state.selectedCategoryId) {
        return category;
      }

      return category.copyWith(
        fields: category.fields.map((field) {
          if (field.key != key) return field;
          if (error == null) {
            return field.copyWith(clearError: true);
          }
          return field.copyWith(error: error);
        }).toList(),
      );
    }).toList();

    state = state.copyWith(
      data: state.data!.copyWith(categories: categories),
      hasConflict: false,
    );
  }

  /// フィールドの値を更新し、変更状態とバリデーションを再計算する
  void onFieldChanged(String key, ConfigValue value) {
    if (state.data == null || state.selectedCategoryId == null) return;

    final categories = state.data!.categories.map((category) {
      if (category.schema.id != state.selectedCategoryId) {
        return category;
      }

      return category.copyWith(
        fields: category.fields.map((field) {
          if (field.key != key) return field;
          return updateFieldState(field, value);
        }).toList(),
      );
    }).toList();

    state = state.copyWith(
      data: state.data!.copyWith(
        categories: categories,
        dirtyCount: countDirtyFields(categories),
      ),
      hasConflict: false,
    );
  }

  /// フィールドをスキーマのデフォルト値にリセットする
  void resetFieldToDefault(String key) {
    if (state.data == null || state.selectedCategoryId == null) return;

    final categories = state.data!.categories.map((category) {
      if (category.schema.id != state.selectedCategoryId) {
        return category;
      }

      return category.copyWith(
        fields: category.fields.map((field) {
          if (field.key != key) return field;
          /// デフォルト値がない場合は空文字列にフォールバックする
          final defaultValue =
              field.schema.defaultValue ?? const StringConfigValue('');
          return updateFieldState(field, defaultValue);
        }).toList(),
      );
    }).toList();

    state = state.copyWith(
      data: state.data!.copyWith(
        categories: categories,
        dirtyCount: countDirtyFields(categories),
      ),
    );
  }

  /// 全フィールドの変更を破棄して元の値に戻す
  void discard() {
    if (state.data == null) return;
    state = state.copyWith(
      data: resetConfigData(state.data!),
      hasConflict: false,
    );
  }

  /// 変更されたフィールドを API に保存する
  /// FE-08 対応: 逐次 PUT の部分失敗時にロールバックを実行する
  /// - 保存開始前に変更前の状態をスナップショットとして保存する
  /// - PUT 中にエラーが発生した場合、成功済み分も含めてスナップショットに戻す
  /// - コンフリクト（409）発生時は hasConflict フラグを設定する
  /// - その他エラーで部分失敗が発生した場合は hasPartialFailure フラグをセットしてユーザーに通知する
  Future<bool> save() async {
    if (state.data == null ||
        !state.hasDirtyFields ||
        state.hasValidationErrors) {
      return false;
    }

    final dirtyFields = state.data!.categories
        .expand((category) => category.fields)
        .where((field) => field.isDirty)
        .toList();

    // FE-08 対応: ロールバック用スナップショットを保存開始前に取得する
    // PUT が途中で失敗した場合はこのスナップショットで全体を復元する
    final originalData = state.data!;

    state = state.copyWith(isSaving: true);

    // 成功した PUT の件数を追跡し、部分失敗の判定に使用する
    var successCount = 0;

    try {
      for (final field in dirtyFields) {
        await dio.put(
          '/api/v1/config/${Uri.encodeComponent(field.namespace)}/${Uri.encodeComponent(field.key)}',
          data: {
            'value': field.value.toJson(),
            'version': field.version,
          },
        );
        successCount++;
      }

      await load();
      state = state.copyWith(hasConflict: false, hasPartialFailure: false);
      return true;
    } on DioException catch (e) {
      if (e.response?.statusCode == 409) {
        // コンフリクト: ロールバック不要（サーバー側は変更されていない）
        state = state.copyWith(hasConflict: true, isSaving: false);
        return false;
      }

      // 部分失敗: 1件以上 PUT が成功していた場合はサーバー側が中途半端な状態になっている
      // クライアント側をスナップショットに戻し、ユーザーに再確認を促す
      final isPartialFailure = successCount > 0;
      state = state.copyWith(
        data: isPartialFailure ? originalData : state.data,
        error: isPartialFailure
            ? '$successCount / ${dirtyFields.length} 件の保存に成功しましたが、残りの保存に失敗しました。'
              ' ページを再読み込みしてサーバーの状態を確認してください。'
            : (e.message ?? 'Failed to save config'),
        isSaving: false,
        hasPartialFailure: isPartialFailure,
      );
      return false;
    }
  }
}

/// ConfigEditorNotifier の Provider を生成するファクトリ
/// 各サービスごとに独立した Notifier インスタンスを提供する
StateNotifierProvider<ConfigEditorNotifier, ConfigEditorState>
    configEditorProvider(Dio dio, String serviceName) {
  return StateNotifierProvider<ConfigEditorNotifier, ConfigEditorState>(
    (ref) {
      final notifier = ConfigEditorNotifier(
        dio: dio,
        serviceName: serviceName,
      );
      /// 初回ロードを自動的に実行する
      notifier.load();
      return notifier;
    },
  );
}
