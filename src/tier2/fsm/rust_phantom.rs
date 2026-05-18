// k1s0 tier2 FSM: Rust phantom type による状態遷移型エンコーディング
// fsm_spec.yaml の OrderStatus / BatchStatus を Rust の phantom type で表現する
// 不正な状態遷移を compile-time エラーで検出する（状態遷移パターン 13）

// PhantomData を使って状態を型パラメータで表現する
use std::marker::PhantomData;

// ============================================================
// OrderStatus FSM: オーダーの状態遷移
// ============================================================

// Draft 状態（下書き、未確定）を表すゼロサイズ型
pub struct Draft;
// Submitted 状態（提出済み、確定）を表すゼロサイズ型
pub struct Submitted;
// Processing 状態（処理中）を表すゼロサイズ型
pub struct Processing;
// Completed 状態（完了、終端）を表すゼロサイズ型
pub struct Completed;
// Cancelled 状態（中止、終端）を表すゼロサイズ型
pub struct Cancelled;

// Order[State]: 状態を型パラメータで保持するオーダー型
// PhantomData[State] によって状態が型レベルで管理される
pub struct Order<State> {
    // オーダーの主キー
    pub id: String,
    // PhantomData で State を型パラメータとして保持する（runtime では 0 バイト）
    _state: PhantomData<State>,
}

// Order<Draft> の固有メソッド（Draft 状態からのみ呼べる遷移）
impl Order<Draft> {
    // 新規オーダーを Draft 状態で生成する
    pub fn new(id: String) -> Self {
        // PhantomData でゼロコストの状態追跡を行う
        Self { id, _state: PhantomData }
    }

    // Draft → Submitted: 提出操作（Submit イベント）
    // 戻り値は Order<Submitted>（型が変わる = 状態遷移を型で表現する）
    pub fn submit(self) -> Order<Submitted> {
        // 同一 id で Submitted 状態の Order を返す（状態遷移を型で表現する）
        Order { id: self.id, _state: PhantomData }
    }

    // Draft → Cancelled: キャンセル操作（Cancel イベント）
    // 戻り値は Order<Cancelled>（終端状態）
    pub fn cancel(self) -> Order<Cancelled> {
        // 同一 id で Cancelled 状態の Order を返す
        Order { id: self.id, _state: PhantomData }
    }
}

// Order<Submitted> の固有メソッド（Submitted 状態からのみ呼べる遷移）
impl Order<Submitted> {
    // Submitted → Processing: 処理開始操作（StartProcessing イベント）
    // 戻り値は Order<Processing>（状態遷移を型で表現する）
    pub fn start_processing(self) -> Order<Processing> {
        // 同一 id で Processing 状態の Order を返す
        Order { id: self.id, _state: PhantomData }
    }

    // Submitted → Cancelled: 処理開始前のキャンセル操作
    // 戻り値は Order<Cancelled>（終端状態）
    pub fn cancel(self) -> Order<Cancelled> {
        // 同一 id で Cancelled 状態の Order を返す
        Order { id: self.id, _state: PhantomData }
    }
}

// Order<Processing> の固有メソッド（Processing 状態からのみ呼べる遷移）
impl Order<Processing> {
    // Processing → Completed: 処理完了操作（Complete イベント）
    // 戻り値は Order<Completed>（終端状態）
    pub fn complete(self) -> Order<Completed> {
        // 同一 id で Completed 状態の Order を返す
        Order { id: self.id, _state: PhantomData }
    }

    // Processing → Cancelled: 処理中キャンセル操作（Cancel イベント）
    // 戻り値は Order<Cancelled>（終端状態）
    pub fn cancel(self) -> Order<Cancelled> {
        // 同一 id で Cancelled 状態の Order を返す
        Order { id: self.id, _state: PhantomData }
    }
}

// ============================================================
// BatchStatus FSM: バッチ処理の状態遷移
// ============================================================

// Registered 状態（登録済み）を表すゼロサイズ型
pub struct Registered;
// Running 状態（実行中）を表すゼロサイズ型
pub struct Running;
// Paused 状態（一時停止）を表すゼロサイズ型
pub struct Paused;
// Succeeded 状態（成功、終端）を表すゼロサイズ型
pub struct Succeeded;
// Failed 状態（失敗、終端）を表すゼロサイズ型
pub struct Failed;

// Batch[State]: バッチ処理の状態を型パラメータで保持する型
pub struct Batch<State> {
    // バッチの主キー
    pub id: String,
    // PhantomData で State を型パラメータとして保持する
    _state: PhantomData<State>,
}

// Batch<Registered> の固有メソッド
impl Batch<Registered> {
    // 新規バッチを Registered 状態で生成する
    pub fn new(id: String) -> Self {
        // PhantomData でゼロコストの状態追跡を行う
        Self { id, _state: PhantomData }
    }

    // Registered → Running: バッチ実行開始操作（Start イベント）
    pub fn start(self) -> Batch<Running> {
        // 同一 id で Running 状態の Batch を返す
        Batch { id: self.id, _state: PhantomData }
    }
}

// Batch<Running> の固有メソッド
impl Batch<Running> {
    // Running → Paused: 一時停止操作（Pause イベント）
    pub fn pause(self) -> Batch<Paused> {
        // 同一 id で Paused 状態の Batch を返す
        Batch { id: self.id, _state: PhantomData }
    }

    // Running → Succeeded: 正常終了操作（Succeed イベント）
    pub fn succeed(self) -> Batch<Succeeded> {
        // 同一 id で Succeeded 状態の Batch を返す
        Batch { id: self.id, _state: PhantomData }
    }

    // Running → Failed: 異常終了操作（Fail イベント）
    pub fn fail(self) -> Batch<Failed> {
        // 同一 id で Failed 状態の Batch を返す
        Batch { id: self.id, _state: PhantomData }
    }
}

// Batch<Paused> の固有メソッド
impl Batch<Paused> {
    // Paused → Running: 再開操作（Resume イベント）
    pub fn resume(self) -> Batch<Running> {
        // 同一 id で Running 状態の Batch を返す
        Batch { id: self.id, _state: PhantomData }
    }

    // Paused → Failed: 一時停止中の異常終了（Fail イベント）
    pub fn fail(self) -> Batch<Failed> {
        // 同一 id で Failed 状態の Batch を返す
        Batch { id: self.id, _state: PhantomData }
    }
}

#[cfg(test)]
mod tests {
    // テストモジュール内部でのみ使用するインポート
    use super::*;

    #[test]
    // OrderStatus FSM: 正常遷移パスを確認する
    fn test_order_normal_path() {
        // Draft 状態から開始する
        let order = Order::<Draft>::new("order-001".to_string());
        // Draft → Submitted: 提出操作
        let submitted = order.submit();
        // Submitted → Processing: 処理開始操作
        let processing = submitted.start_processing();
        // Processing → Completed: 処理完了操作
        let completed = processing.complete();
        // 完了後の id が変わらないことを確認する
        assert_eq!(completed.id, "order-001");
    }

    #[test]
    // OrderStatus FSM: キャンセルパスを確認する
    fn test_order_cancel_from_draft() {
        // Draft 状態から直接キャンセルする
        let order = Order::<Draft>::new("order-002".to_string());
        // Draft → Cancelled: キャンセル操作
        let cancelled = order.cancel();
        // キャンセル後の id が変わらないことを確認する
        assert_eq!(cancelled.id, "order-002");
    }

    #[test]
    // BatchStatus FSM: 正常遷移パスを確認する
    fn test_batch_normal_path() {
        // Registered 状態から開始する
        let batch = Batch::<Registered>::new("batch-001".to_string());
        // Registered → Running: 実行開始操作
        let running = batch.start();
        // Running → Succeeded: 正常終了操作
        let succeeded = running.succeed();
        // 成功後の id が変わらないことを確認する
        assert_eq!(succeeded.id, "batch-001");
    }

    #[test]
    // BatchStatus FSM: Pause → Resume パスを確認する
    fn test_batch_pause_resume_path() {
        // Running 状態から一時停止して再開する
        let batch = Batch::<Registered>::new("batch-002".to_string());
        let running = batch.start();
        // Running → Paused: 一時停止操作
        let paused = running.pause();
        // Paused → Running: 再開操作
        let resumed = paused.resume();
        // 再開後の id が変わらないことを確認する
        assert_eq!(resumed.id, "batch-002");
    }
}
