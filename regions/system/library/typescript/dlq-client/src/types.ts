/** DLQ メッセージステータス。 */
export type DlqStatus = 'PENDING' | 'RETRYING' | 'RESOLVED' | 'DEAD';

/** DLQ メッセージ。 */
export interface DlqMessage {
  id: string;
  originalTopic: string;
  errorMessage: string;
  retryCount: number;
  maxRetries: number;
  payload: unknown;
  status: DlqStatus;
  createdAt: string;
  lastRetryAt: string | null;
}

/** DLQ メッセージ一覧取得レスポンス。 */
export interface ListDlqMessagesResponse {
  messages: DlqMessage[];
  total: number;
  page: number;
}

/** DLQ メッセージ再処理レスポンス。 */
export interface RetryDlqMessageResponse {
  messageId: string;
  status: DlqStatus;
}
