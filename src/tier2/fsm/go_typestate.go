// k1s0 tier2 FSM: Go typestate パターンによる状態遷移型エンコーディング
// fsm_spec.yaml の OrderStatus / BatchStatus を Go の typestate パターンで表現する
// Go のジェネリクス + interface 制約で不正な状態遷移を compile-time に制限する
// （状態遷移パターン 13）

// パッケージ名: fsm
package fsm

// ============================================================
// OrderStatus FSM: オーダーの状態遷移
// ============================================================

// OrderState: オーダー状態のマーカーインターフェース
// sealed interface 相当（このパッケージ外で実装できない）
type OrderState interface {
	// orderState メソッドでパッケージ内に閉じた型制約を実現する
	orderState()
}

// DraftState: Draft 状態（下書き、未確定）を表す型
type DraftState struct{}

// orderState メソッドで OrderState インターフェースを実装する（ゼロコスト）
func (DraftState) orderState() {}

// SubmittedState: Submitted 状態（提出済み、確定）を表す型
type SubmittedState struct{}

// orderState メソッドで OrderState インターフェースを実装する
func (SubmittedState) orderState() {}

// ProcessingState: Processing 状態（処理中）を表す型
type ProcessingState struct{}

// orderState メソッドで OrderState インターフェースを実装する
func (ProcessingState) orderState() {}

// CompletedState: Completed 状態（完了、終端）を表す型
type CompletedState struct{}

// orderState メソッドで OrderState インターフェースを実装する
func (CompletedState) orderState() {}

// CancelledState: Cancelled 状態（中止、終端）を表す型
type CancelledState struct{}

// orderState メソッドで OrderState インターフェースを実装する
func (CancelledState) orderState() {}

// Order[S]: 状態を型パラメータで保持するオーダー型
// S は OrderState を実装する型に制限される（Go のジェネリクス型制約）
type Order[S OrderState] struct {
	// オーダーの主キー
	ID string
	// 現在の状態（型パラメータで静的に保証する）
	State S
}

// NewOrder: Draft 状態のオーダーを生成するファクトリ関数
func NewOrder(id string) Order[DraftState] {
	// DraftState で Order を生成して返す
	return Order[DraftState]{ID: id, State: DraftState{}}
}

// Submit: Draft → Submitted の状態遷移（Submit イベント）
// Order[DraftState] からのみ呼べる（コンパイル時に型で制約する）
func Submit(o Order[DraftState]) Order[SubmittedState] {
	// 同一 ID で SubmittedState の Order を返す
	return Order[SubmittedState]{ID: o.ID, State: SubmittedState{}}
}

// CancelFromDraft: Draft → Cancelled の状態遷移（Cancel イベント）
func CancelFromDraft(o Order[DraftState]) Order[CancelledState] {
	// 同一 ID で CancelledState の Order を返す
	return Order[CancelledState]{ID: o.ID, State: CancelledState{}}
}

// StartProcessing: Submitted → Processing の状態遷移（StartProcessing イベント）
// Order[SubmittedState] からのみ呼べる
func StartProcessing(o Order[SubmittedState]) Order[ProcessingState] {
	// 同一 ID で ProcessingState の Order を返す
	return Order[ProcessingState]{ID: o.ID, State: ProcessingState{}}
}

// CancelFromSubmitted: Submitted → Cancelled の状態遷移
func CancelFromSubmitted(o Order[SubmittedState]) Order[CancelledState] {
	// 同一 ID で CancelledState の Order を返す
	return Order[CancelledState]{ID: o.ID, State: CancelledState{}}
}

// Complete: Processing → Completed の状態遷移（Complete イベント）
// Order[ProcessingState] からのみ呼べる
func Complete(o Order[ProcessingState]) Order[CompletedState] {
	// 同一 ID で CompletedState の Order を返す
	return Order[CompletedState]{ID: o.ID, State: CompletedState{}}
}

// CancelFromProcessing: Processing → Cancelled の状態遷移
func CancelFromProcessing(o Order[ProcessingState]) Order[CancelledState] {
	// 同一 ID で CancelledState の Order を返す
	return Order[CancelledState]{ID: o.ID, State: CancelledState{}}
}

// ============================================================
// BatchStatus FSM: バッチ処理の状態遷移
// ============================================================

// BatchState: バッチ状態のマーカーインターフェース
// sealed interface 相当（このパッケージ外で実装できない）
type BatchState interface {
	// batchState メソッドでパッケージ内に閉じた型制約を実現する
	batchState()
}

// RegisteredState: Registered 状態（登録済み）を表す型
type RegisteredState struct{}

// batchState メソッドで BatchState インターフェースを実装する
func (RegisteredState) batchState() {}

// RunningState: Running 状態（実行中）を表す型
type RunningState struct{}

// batchState メソッドで BatchState インターフェースを実装する
func (RunningState) batchState() {}

// PausedState: Paused 状態（一時停止）を表す型
type PausedState struct{}

// batchState メソッドで BatchState インターフェースを実装する
func (PausedState) batchState() {}

// SucceededState: Succeeded 状態（成功、終端）を表す型
type SucceededState struct{}

// batchState メソッドで BatchState インターフェースを実装する
func (SucceededState) batchState() {}

// FailedState: Failed 状態（失敗、終端）を表す型
type FailedState struct{}

// batchState メソッドで BatchState インターフェースを実装する
func (FailedState) batchState() {}

// Batch[S]: バッチ処理の状態を型パラメータで保持する型
type Batch[S BatchState] struct {
	// バッチの主キー
	ID string
	// 現在の状態（型パラメータで静的に保証する）
	State S
}

// NewBatch: Registered 状態のバッチを生成するファクトリ関数
func NewBatch(id string) Batch[RegisteredState] {
	// RegisteredState で Batch を生成して返す
	return Batch[RegisteredState]{ID: id, State: RegisteredState{}}
}

// StartBatch: Registered → Running の状態遷移（Start イベント）
func StartBatch(b Batch[RegisteredState]) Batch[RunningState] {
	// 同一 ID で RunningState の Batch を返す
	return Batch[RunningState]{ID: b.ID, State: RunningState{}}
}

// PauseBatch: Running → Paused の状態遷移（Pause イベント）
func PauseBatch(b Batch[RunningState]) Batch[PausedState] {
	// 同一 ID で PausedState の Batch を返す
	return Batch[PausedState]{ID: b.ID, State: PausedState{}}
}

// ResumeBatch: Paused → Running の状態遷移（Resume イベント）
func ResumeBatch(b Batch[PausedState]) Batch[RunningState] {
	// 同一 ID で RunningState の Batch を返す
	return Batch[RunningState]{ID: b.ID, State: RunningState{}}
}

// SucceedBatch: Running → Succeeded の状態遷移（Succeed イベント）
func SucceedBatch(b Batch[RunningState]) Batch[SucceededState] {
	// 同一 ID で SucceededState の Batch を返す
	return Batch[SucceededState]{ID: b.ID, State: SucceededState{}}
}

// FailBatchFromRunning: Running → Failed の状態遷移（Fail イベント）
func FailBatchFromRunning(b Batch[RunningState]) Batch[FailedState] {
	// 同一 ID で FailedState の Batch を返す
	return Batch[FailedState]{ID: b.ID, State: FailedState{}}
}

// FailBatchFromPaused: Paused → Failed の状態遷移（Fail イベント）
func FailBatchFromPaused(b Batch[PausedState]) Batch[FailedState] {
	// 同一 ID で FailedState の Batch を返す
	return Batch[FailedState]{ID: b.ID, State: FailedState{}}
}
