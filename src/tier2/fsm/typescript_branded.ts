/**
 * k1s0 tier2 FSM: TypeScript branded type による状態遷移型エンコーディング
 * fsm_spec.yaml の OrderStatus / BatchStatus を TypeScript の branded type で表現する
 * 不正な状態遷移は TypeScript の型チェックで compile-time エラーとして検出する
 * （状態遷移パターン 13）
 */

// ============================================================
// OrderStatus FSM: オーダーの状態遷移
// ============================================================

/**
 * OrderState: オーダーの状態を表す brand 付き型
 * 各状態を distinct な型として定義し、不正遷移を型エラーで検出する
 */
// OrderStateBrand: 型の brand 定義（nominal typing を実現する）
declare const orderStateBrand: unique symbol;

// Draft 状態の branded type
type OrderDraft = { readonly [orderStateBrand]: "Draft" };
// Submitted 状態の branded type
type OrderSubmitted = { readonly [orderStateBrand]: "Submitted" };
// Processing 状態の branded type
type OrderProcessing = { readonly [orderStateBrand]: "Processing" };
// Completed 状態の branded type（終端状態）
type OrderCompleted = { readonly [orderStateBrand]: "Completed" };
// Cancelled 状態の branded type（終端状態）
type OrderCancelled = { readonly [orderStateBrand]: "Cancelled" };

/**
 * Order<TState>: 状態を型パラメータで保持するオーダー型
 * TState は OrderState の union 型のメンバーに制限する
 */
// Order インターフェース定義（状態を型パラメータで管理する）
export interface Order<TState> {
  // オーダーの主キー
  readonly id: string;
  // 現在の状態（型パラメータで静的に保証する、実際の値は使わない）
  readonly _state: TState;
}

/**
 * createOrder: Draft 状態のオーダーを生成するファクトリ関数
 */
// createOrder 関数（Draft 状態の Order を生成する）
export function createOrder(id: string): Order<OrderDraft> {
  // Draft 状態で Order を生成する（型を明示的にキャストする）
  return { id, _state: {} as OrderDraft };
}

/**
 * submitOrder: Draft → Submitted の状態遷移（Submit イベント）
 * Order<OrderDraft> からのみ呼べる型制約を持つ
 */
// submitOrder 関数（Draft → Submitted の遷移）
export function submitOrder(order: Order<OrderDraft>): Order<OrderSubmitted> {
  // 同一 id で Submitted 状態の Order を返す
  return { id: order.id, _state: {} as OrderSubmitted };
}

/**
 * cancelOrderFromDraft: Draft → Cancelled の状態遷移（Cancel イベント）
 */
// cancelOrderFromDraft 関数（Draft → Cancelled の遷移）
export function cancelOrderFromDraft(order: Order<OrderDraft>): Order<OrderCancelled> {
  // 同一 id で Cancelled 状態の Order を返す
  return { id: order.id, _state: {} as OrderCancelled };
}

/**
 * startProcessing: Submitted → Processing の状態遷移（StartProcessing イベント）
 * Order<OrderSubmitted> からのみ呼べる型制約を持つ
 */
// startProcessing 関数（Submitted → Processing の遷移）
export function startProcessing(order: Order<OrderSubmitted>): Order<OrderProcessing> {
  // 同一 id で Processing 状態の Order を返す
  return { id: order.id, _state: {} as OrderProcessing };
}

/**
 * cancelOrderFromSubmitted: Submitted → Cancelled の状態遷移
 */
// cancelOrderFromSubmitted 関数（Submitted → Cancelled の遷移）
export function cancelOrderFromSubmitted(order: Order<OrderSubmitted>): Order<OrderCancelled> {
  // 同一 id で Cancelled 状態の Order を返す
  return { id: order.id, _state: {} as OrderCancelled };
}

/**
 * completeOrder: Processing → Completed の状態遷移（Complete イベント）
 * Order<OrderProcessing> からのみ呼べる型制約を持つ
 */
// completeOrder 関数（Processing → Completed の遷移）
export function completeOrder(order: Order<OrderProcessing>): Order<OrderCompleted> {
  // 同一 id で Completed 状態の Order を返す
  return { id: order.id, _state: {} as OrderCompleted };
}

/**
 * cancelOrderFromProcessing: Processing → Cancelled の状態遷移
 */
// cancelOrderFromProcessing 関数（Processing → Cancelled の遷移）
export function cancelOrderFromProcessing(order: Order<OrderProcessing>): Order<OrderCancelled> {
  // 同一 id で Cancelled 状態の Order を返す
  return { id: order.id, _state: {} as OrderCancelled };
}

// ============================================================
// BatchStatus FSM: バッチ処理の状態遷移
// ============================================================

/**
 * BatchState: バッチの状態を表す brand 付き型
 */
// BatchStateBrand: 型の brand 定義
declare const batchStateBrand: unique symbol;

// Registered 状態の branded type
type BatchRegistered = { readonly [batchStateBrand]: "Registered" };
// Running 状態の branded type
type BatchRunning = { readonly [batchStateBrand]: "Running" };
// Paused 状態の branded type
type BatchPaused = { readonly [batchStateBrand]: "Paused" };
// Succeeded 状態の branded type（終端状態）
type BatchSucceeded = { readonly [batchStateBrand]: "Succeeded" };
// Failed 状態の branded type（終端状態）
type BatchFailed = { readonly [batchStateBrand]: "Failed" };

/**
 * Batch<TState>: バッチ処理の状態を型パラメータで保持する型
 */
// Batch インターフェース定義
export interface Batch<TState> {
  // バッチの主キー
  readonly id: string;
  // 現在の状態（型パラメータで静的に保証する）
  readonly _state: TState;
}

/**
 * createBatch: Registered 状態のバッチを生成するファクトリ関数
 */
// createBatch 関数（Registered 状態の Batch を生成する）
export function createBatch(id: string): Batch<BatchRegistered> {
  // Registered 状態で Batch を生成する
  return { id, _state: {} as BatchRegistered };
}

/**
 * startBatch: Registered → Running の状態遷移（Start イベント）
 */
// startBatch 関数（Registered → Running の遷移）
export function startBatch(batch: Batch<BatchRegistered>): Batch<BatchRunning> {
  // 同一 id で Running 状態の Batch を返す
  return { id: batch.id, _state: {} as BatchRunning };
}

/**
 * pauseBatch: Running → Paused の状態遷移（Pause イベント）
 */
// pauseBatch 関数（Running → Paused の遷移）
export function pauseBatch(batch: Batch<BatchRunning>): Batch<BatchPaused> {
  // 同一 id で Paused 状態の Batch を返す
  return { id: batch.id, _state: {} as BatchPaused };
}

/**
 * resumeBatch: Paused → Running の状態遷移（Resume イベント）
 */
// resumeBatch 関数（Paused → Running の遷移）
export function resumeBatch(batch: Batch<BatchPaused>): Batch<BatchRunning> {
  // 同一 id で Running 状態の Batch を返す
  return { id: batch.id, _state: {} as BatchRunning };
}

/**
 * succeedBatch: Running → Succeeded の状態遷移（Succeed イベント）
 */
// succeedBatch 関数（Running → Succeeded の遷移）
export function succeedBatch(batch: Batch<BatchRunning>): Batch<BatchSucceeded> {
  // 同一 id で Succeeded 状態の Batch を返す
  return { id: batch.id, _state: {} as BatchSucceeded };
}

/**
 * failBatchFromRunning: Running → Failed の状態遷移（Fail イベント）
 */
// failBatchFromRunning 関数（Running → Failed の遷移）
export function failBatchFromRunning(batch: Batch<BatchRunning>): Batch<BatchFailed> {
  // 同一 id で Failed 状態の Batch を返す
  return { id: batch.id, _state: {} as BatchFailed };
}

/**
 * failBatchFromPaused: Paused → Failed の状態遷移（Fail イベント）
 */
// failBatchFromPaused 関数（Paused → Failed の遷移）
export function failBatchFromPaused(batch: Batch<BatchPaused>): Batch<BatchFailed> {
  // 同一 id で Failed 状態の Batch を返す
  return { id: batch.id, _state: {} as BatchFailed };
}
