/**
 * workflow.ts — k1s0 tier1 Library TypeScript 実装: Workflow / Long-running Saga の L1+ interface
 * 16_ワークフロー適合仕様.md §WorkflowClient（Temporal L1+ 深耕）に準拠する。
 * Temporal の full API を Library 独自語彙で表現しつつ、AuthContext 伝播と tenant 分離を強制する。
 * OSS 型（@temporalio/client 等）を公開シグネチャに一切含まない。
 */

/**
 * WorkflowStatus はワークフロー実行の状態を宣言する enum。
 * Temporal の workflow execution status に準拠した Library 独自語彙とする。
 */
// WorkflowStatus 列挙型定義
export const enum WorkflowStatus {
  // Running: 実行中
  Running = "running",
  // Completed: 正常完了
  Completed = "completed",
  // Failed: 失敗（リトライ上限超過）
  Failed = "failed",
  // Canceled: キャンセルされた
  Canceled = "canceled",
  // TimedOut: タイムアウト
  TimedOut = "timed_out",
  // ContinuedAsNew: 継続（ContinueAsNew パターン）
  ContinuedAsNew = "continued_as_new",
  // Terminated: 強制終了
  Terminated = "terminated",
}

/**
 * WorkflowOptions はワークフロー開始オプションを宣言する型。
 * Temporal の StartWorkflowOptions を Library 独自語彙に翻訳する。
 */
// WorkflowOptions 型定義
export interface WorkflowOptions {
  // workflowId: ワークフロー識別子（idempotency key として使用する）
  readonly workflowId: string;
  // taskQueue: 実行するタスクキュー名（Worker が listen するキュー）
  readonly taskQueue: string;
  // maxRetries: ワークフロー全体のリトライ上限回数（0 = リトライなし）
  readonly maxRetries?: number | undefined;
  // retentionDays: ワークフロー履歴の保持日数（0 = デフォルト設定を使用する）
  readonly retentionDays?: number | undefined;
  // searchAttributes: Temporal Search Attribute（ダッシュボード検索用のインデックス対象属性）
  readonly searchAttributes?: Readonly<Record<string, unknown>> | undefined;
  // memo: ワークフローメモ（非インデックスの補足情報）
  readonly memo?: Readonly<Record<string, unknown>> | undefined;
}

/**
 * WorkflowExecution はワークフロー実行の識別子を宣言する型。
 */
// WorkflowExecution 型定義
export interface WorkflowExecution {
  // workflowId: ワークフロー識別子
  readonly workflowId: string;
  // runId: 実行 ID（同一 workflowId のリトライ・再実行を区別する）
  readonly runId: string;
  // tenantId: ワークフローの所属テナント識別子
  readonly tenantId: string;
}

/**
 * WorkflowRun はワークフロー実行ハンドルを宣言する interface。
 * Temporal の WorkflowHandle を Library 独自語彙で表現する。
 */
// WorkflowRun インターフェース定義
export interface WorkflowRun {
  /** workflowId はワークフロー識別子を返す。 */
  // workflowId プロパティ
  readonly workflowId: string;

  /** runId は実行 ID を返す。 */
  // runId プロパティ
  readonly runId: string;

  /**
   * result はワークフローの完了を待機して結果を取得する。
   * signal で待機をキャンセルする（AbortSignal）。
   */
  // result メソッド: ワークフロー完了を待機して結果を取得する
  result<T>(signal?: AbortSignal): Promise<T>;

  /**
   * status はワークフローの現在の状態を返す。
   */
  // status メソッド: 現在の状態を返す
  status(): Promise<WorkflowStatus>;
}

/**
 * WorkflowDescription はワークフロー実行の詳細情報を宣言する型。
 */
// WorkflowDescription 型定義
export interface WorkflowDescription {
  // execution: ワークフロー実行の識別子
  readonly execution: WorkflowExecution;
  // status: 現在の状態
  readonly status: WorkflowStatus;
  // workflowType: ワークフロータイプ名
  readonly workflowType: string;
  // startTimeMs: 開始時刻（Unix ミリ秒）
  readonly startTimeMs: number;
  // closeTimeMs: 終了時刻（Unix ミリ秒: 実行中の場合は 0）
  readonly closeTimeMs: number;
  // searchAttributes: 検索属性
  readonly searchAttributes: Readonly<Record<string, unknown>>;
  // memo: ワークフローメモ
  readonly memo: Readonly<Record<string, unknown>>;
}

/**
 * WorkflowClient は Workflow / Long-running Saga の L1+ 抽象 interface を宣言する。
 * Temporal の full API を Library 独自語彙で表現する。
 * OSS 型（@temporalio/client 等）を一切含まない。
 * tenantId を必須として AuthContext 伝播を強制する（tenant 分離必須）。
 */
// WorkflowClient インターフェース定義
export interface WorkflowClient {
  /**
   * startWorkflow はワークフローを開始して WorkflowRun を返す。
   * workflowType はワークフロー登録名。
   * args はワークフロー開始引数（protobuf の JSON シリアライズ値を推奨する）。
   */
  // startWorkflow メソッド: ワークフローを開始する
  startWorkflow(
    workflowType: string,
    opts: WorkflowOptions,
    ...args: unknown[]
  ): Promise<WorkflowRun>;

  /**
   * signalWorkflow は実行中のワークフローにシグナルを送信する。
   * runId = "" で最新の実行に送信する。
   */
  // signalWorkflow メソッド: シグナルを送信する
  signalWorkflow(
    tenantId: string,
    workflowId: string,
    runId: string,
    signalName: string,
    arg?: unknown,
  ): Promise<void>;

  /**
   * queryWorkflow は実行中のワークフローにクエリを送信して結果を取得する。
   */
  // queryWorkflow メソッド: クエリを送信する
  queryWorkflow<T>(
    tenantId: string,
    workflowId: string,
    runId: string,
    queryType: string,
    ...args: unknown[]
  ): Promise<T>;

  /**
   * cancelWorkflow は実行中のワークフローをキャンセルする（グレースフルキャンセル）。
   */
  // cancelWorkflow メソッド: ワークフローをキャンセルする
  cancelWorkflow(tenantId: string, workflowId: string, runId: string): Promise<void>;

  /**
   * terminateWorkflow は実行中のワークフローを強制終了する（理由を指定する）。
   */
  // terminateWorkflow メソッド: ワークフローを強制終了する
  terminateWorkflow(
    tenantId: string,
    workflowId: string,
    runId: string,
    reason: string,
  ): Promise<void>;

  /**
   * describeWorkflow はワークフローの詳細情報を取得する。
   */
  // describeWorkflow メソッド: ワークフローの詳細情報を取得する
  describeWorkflow(
    tenantId: string,
    workflowId: string,
    runId: string,
  ): Promise<WorkflowDescription>;

  /**
   * close はクライアントのリソースを解放する。
   */
  // close メソッド: クライアントのリソースを解放する
  close(): void;
}
