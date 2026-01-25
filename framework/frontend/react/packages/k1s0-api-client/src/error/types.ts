import { z } from 'zod';

/**
 * RFC 7807 Problem Details 形式のスキーマ
 * バックエンドからのエラーレスポンスの標準形式
 */
export const ProblemDetailsSchema = z.object({
  /** エラー種別を示すURI */
  type: z.string().default('about:blank'),
  /** 人間可読なエラータイトル */
  title: z.string(),
  /** HTTPステータスコード */
  status: z.number().int().min(400).max(599),
  /** エラーの詳細説明 */
  detail: z.string().optional(),
  /** エラーが発生したリソースのURI */
  instance: z.string().optional(),
  /** k1s0標準: エラーコード（運用で一次判断に使用） */
  error_code: z.string(),
  /** k1s0標準: トレースID（ログ/トレース調査の入口） */
  trace_id: z.string().optional(),
  /** k1s0標準: フィールド単位のバリデーションエラー */
  errors: z
    .array(
      z.object({
        field: z.string(),
        message: z.string(),
        code: z.string().optional(),
      })
    )
    .optional(),
});

export type ProblemDetails = z.infer<typeof ProblemDetailsSchema>;

/**
 * APIエラーの分類（内部分類）
 * バックエンドのerror分類と対応
 */
export type ApiErrorKind =
  | 'validation' // 入力不備（400系）
  | 'authentication' // 認証エラー（401）
  | 'authorization' // 認可エラー（403）
  | 'not_found' // リソース不存在（404）
  | 'conflict' // 競合/重複（409）
  | 'dependency' // 依存先障害（502/503等）
  | 'temporary' // 一時障害/リトライ可能（503等）
  | 'rate_limit' // レート制限（429）
  | 'timeout' // タイムアウト
  | 'network' // ネットワークエラー
  | 'unknown'; // 不明なエラー

/**
 * HTTPステータスコードからエラー分類へのマッピング
 */
export function mapStatusToErrorKind(status: number): ApiErrorKind {
  if (status === 400) return 'validation';
  if (status === 401) return 'authentication';
  if (status === 403) return 'authorization';
  if (status === 404) return 'not_found';
  if (status === 409) return 'conflict';
  if (status === 429) return 'rate_limit';
  if (status === 502 || status === 503) return 'dependency';
  if (status === 504) return 'timeout';
  if (status >= 500) return 'temporary';
  return 'unknown';
}

/**
 * エラー分類がリトライ可能かどうかを判定
 */
export function isRetryableError(kind: ApiErrorKind): boolean {
  return kind === 'temporary' || kind === 'dependency' || kind === 'timeout';
}

/**
 * エラー分類に対応するユーザー向けデフォルトメッセージ
 */
export function getDefaultErrorMessage(kind: ApiErrorKind): string {
  switch (kind) {
    case 'validation':
      return '入力内容に問題があります。内容を確認してください。';
    case 'authentication':
      return '認証が必要です。ログインしてください。';
    case 'authorization':
      return 'この操作を行う権限がありません。';
    case 'not_found':
      return '指定されたリソースが見つかりません。';
    case 'conflict':
      return 'データが競合しています。最新の状態を確認してください。';
    case 'dependency':
      return 'サービスに一時的な問題が発生しています。';
    case 'temporary':
      return 'サービスが一時的に利用できません。しばらくしてから再試行してください。';
    case 'rate_limit':
      return 'リクエストが多すぎます。しばらくしてから再試行してください。';
    case 'timeout':
      return 'リクエストがタイムアウトしました。';
    case 'network':
      return 'ネットワークに接続できません。接続を確認してください。';
    case 'unknown':
    default:
      return '予期しないエラーが発生しました。';
  }
}
