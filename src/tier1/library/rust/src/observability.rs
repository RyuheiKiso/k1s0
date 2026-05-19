// observability.rs — k1s0 tier1 Library: 可観測性 L1+ facade trait（5 signal class 対応）
// OTel SDK 等の OSS 型を公開 API に露出しない（L1+ ラップ規約）。
// 5 signal class: Trace / Metric / Log / Profile / Continuous-Profiling に対応する。
// 全 trait は Send + Sync を要求する（スレッド安全性の強制）。

// Tracer は OTel SDK を L1+ ラップするスパン操作 facade trait。
// 公開 API シグネチャに OSS 型（opentelemetry::trace::Tracer 等）を一切含まない。
// 親スパン ID は context（リクエストスコープの伝播機構）経由で伝播する（引数として受け取らない）。
pub trait Tracer: Send + Sync {
    // start_span は新しいスパンを開始して Box<dyn Span> を返す。
    // name はスパン名（操作名、例: "db.query" / "rpc.call"）。
    // 親スパン ID は OTel context propagation 経由で自動的に設定される。
    fn start_span(&self, name: &str) -> Box<dyn Span>;
}

// Span は単一の操作スパンを表す trait。
// スパンは属性設定後に end() を呼んで終了すること（end() 呼び出し忘れ禁止）。
pub trait Span: Send + Sync {
    // set_attribute はスパンに key-value 属性を設定する。
    // OTel Semantic Conventions の属性名（例: "db.system" / "http.method"）を推奨する。
    fn set_attribute(&mut self, key: &str, value: &str);

    // end はスパンを終了してバックエンドにエクスポートする（Box<Self> を消費する）。
    // end() を呼ばないとスパンがドロップされ、テレメトリが失われる。
    fn end(self: Box<Self>);
}

// MetricRecorder は OTel Metrics SDK を L1+ ラップするメトリクス記録 facade trait。
// 公開 API シグネチャに OSS 型（opentelemetry::metrics::Meter 等）を一切含まない。
pub trait MetricRecorder: Send + Sync {
    // increment_counter はカウンタ値を value だけ増加させる（単調増加カウンタ）。
    // labels は追加の次元ラベル（例: &[("status", "200"), ("method", "GET")]）。
    fn increment_counter(&self, name: &str, value: u64, labels: &[(&str, &str)]);

    // record_histogram はヒストグラムに value を記録する（レイテンシ / サイズ計測用）。
    // labels は追加の次元ラベル（例: &[("operation", "query")]）。
    fn record_histogram(&self, name: &str, value: f64, labels: &[(&str, &str)]);
}

// LogEmitter は OTel Logging SDK を L1+ ラップするログ出力 facade trait。
// 公開 API シグネチャに OSS 型（tracing::Logger 等）を一切含まない。
pub trait LogEmitter: Send + Sync {
    // emit は構造化ログを指定レベルで出力する。
    // attributes は追加の構造化フィールド（例: &[("user_id", "u-123"), ("op", "write")]）。
    fn emit(&self, level: LogLevel, message: &str, attributes: &[(&str, &str)]);
}

// LogLevel はログの重要度レベルを表す enum。
// OTel Logging の SeverityNumber に対応する 5 レベルを定義する。
pub enum LogLevel {
    // Trace: 最詳細なデバッグ情報（開発時のみ使用する）
    Trace,
    // Debug: デバッグ情報（開発・テスト環境で使用する）
    Debug,
    // Info: 一般的な情報ログ（通常のオペレーション記録）
    Info,
    // Warn: 警告ログ（非致命的な異常状態を記録する）
    Warn,
    // Error: エラーログ（処理失敗・例外を記録する）
    Error,
}
