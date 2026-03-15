// GUI全体で使用される文言定数
// テストと実装の両方から参照し、文言の同期を保つ

// --- GeneratePage ---

// BFF関連の文言
export const BFF_LANGUAGE_VALIDATION_ERROR = 'BFF言語を選択してください。';
export const BFF_GENERATE_LABEL = 'BFF生成:';
export const BFF_GENERATE_YES = 'あり';
export const BFF_GENERATE_NO = 'なし';
export const BFF_GENERATE_UNAVAILABLE = '利用不可';
export const BFF_LANGUAGE_LABEL = 'BFF言語:';
export const BFF_LANGUAGE_NONE = '不要';

// BFF有効化ラジオボタンの文言
export const BFF_OPT_IN_YES = 'はい';
export const BFF_OPT_IN_NO = 'いいえ';

// --- TemplateMigratePage ---

// コンフリクト解決の文言
export const CONFLICT_RESOLUTION_HEADING = 'コンフリクト解決';
export const CONFLICT_USE_TEMPLATE = 'テンプレートを使用';
export const CONFLICT_KEEP_USER = 'ユーザーの変更を保持';
export const CONFLICT_SKIP_FILE = 'ファイルをスキップ';

// 移行結果の文言
export const MIGRATION_SUCCESS = 'テンプレート移行が正常に完了しました。';

// ラベルの文言
export const VERSION_LABEL_PREFIX = 'バージョン:';

// --- AuthPage ---

// 接続確認の文言テンプレート
// URLを引数に取り、エラーメッセージを生成する関数
export function connectionFailureMessage(url: string, error: string): string {
  return `${url} の解決に失敗しました。${error}`;
}
