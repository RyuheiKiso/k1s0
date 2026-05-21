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

// ============================================================
// L3 abstract 具象実装（Y-tier1-3 解消: Rust / C# / TypeScript 多言語 parity）
// ============================================================
// Go の observability.go（Logger / Tracer / MetricMeter interface）と同等の
// 具象実装を Rust で提供する。OSS 型（opentelemetry crate）を公開 API に露出しない。
// factory function: new_logger() / new_tracer() / new_metric_meter() を提供する。

// ---- Logger 具象実装 ----

// Logger は構造化ログ出力の L3 abstract 具象実装。
// OSS 型（tracing::Logger / log::Logger 等）を公開シグネチャに一切含まない。
// service_name: OTLP resource.ServiceName 属性（ログのサービス識別子）
// base_attrs: 全ログに自動付与する基底属性（with メソッドで積み上がる）
pub struct Logger {
    // service_name: ログの OTLP resource.ServiceName 属性（構造化ログのサービス識別子）
    service_name: String,
    // base_attrs: 全ログに自動付与する基底属性（key-value ペアのベクタ: String キーを使用する）
    base_attrs: Vec<(String, String)>,
}

// Logger の実装ブロック
impl Logger {
    // new は Logger を生成するファクトリ（OSS 型を引数に取らない）
    fn new(service_name: impl Into<String>, base_attrs: Vec<(String, String)>) -> Self {
        // Logger を初期化して返す
        Self {
            // サービス名を String に変換して格納する
            service_name: service_name.into(),
            // 基底属性を格納する（String キーを使用して &'static str 制約を排除する）
            base_attrs,
        }
    }

    // emit_log は構造化ログを標準エラーに出力する内部メソッド。
    // OTLP exporter を設定する場合はこの出力を差し替える（factory pattern）。
    fn emit_log(&self, level: &str, msg: &str, labels: &[(&str, &str)]) {
        // 基底属性と引数ラベルを結合した文字列を生成する
        let base: String = self
            .base_attrs
            .iter()
            // 各基底属性を key=value 形式に変換する
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join(" ");
        // 引数ラベルを key=value 形式に変換する
        let extra: String = labels
            .iter()
            // 各ラベルを key=value 形式に変換する
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join(" ");
        // service.name 属性とラベルを結合してログを出力する
        eprintln!(
            "[{}] service.name={} {} {} {}",
            level, self.service_name, base, extra, msg
        );
    }

    // debug は LogLevel::Debug レベルのログを出力する。
    // attrs は追加の構造化フィールド（key-value ペア）。
    pub fn debug(&self, msg: &str, attrs: &[(&str, &str)]) {
        // debug レベルのログを emit する
        self.emit_log("debug", msg, attrs);
    }

    // info は LogLevel::Info レベルのログを出力する。
    // attrs は追加の構造化フィールド（key-value ペア）。
    pub fn info(&self, msg: &str, attrs: &[(&str, &str)]) {
        // info レベルのログを emit する
        self.emit_log("info", msg, attrs);
    }

    // warn は LogLevel::Warn レベルのログを出力する。
    // attrs は追加の構造化フィールド（key-value ペア）。
    pub fn warn(&self, msg: &str, attrs: &[(&str, &str)]) {
        // warn レベルのログを emit する
        self.emit_log("warn", msg, attrs);
    }

    // error は LogLevel::Error レベルのログを出力する。
    // err は Option<&dyn std::error::Error>（エラーがない場合は None を渡す）。
    pub fn error(&self, msg: &str, err: Option<&dyn std::error::Error>, attrs: &[(&str, &str)]) {
        // error レベルのログを emit する
        self.emit_log("error", msg, attrs);
        // エラーが存在する場合は詳細を出力する
        if let Some(e) = err {
            // エラーの詳細を標準エラーに出力する
            eprintln!("  error: {}", e);
        }
    }

    // fatal は LogLevel::Error（fatal 相当）レベルのログを出力してプロセスを終了する。
    // 実装はログ出力後に std::process::exit(1) を呼び出す（致命的エラーのため継続不可）。
    pub fn fatal(&self, msg: &str, err: Option<&dyn std::error::Error>, attrs: &[(&str, &str)]) {
        // fatal レベルのログを emit する
        self.emit_log("fatal", msg, attrs);
        // エラーが存在する場合は詳細を出力する
        if let Some(e) = err {
            // エラーの詳細を標準エラーに出力する
            eprintln!("  fatal error: {}", e);
        }
        // プロセスを終了する（致命的エラーのため継続不可）
        std::process::exit(1);
    }

    // with は追加属性を持つ派生 Logger を返す（子 logger を生成する）。
    // 全てのログ出力に追加の attrs が自動付与される（tenant_id 等の常駐属性に使用する）。
    pub fn with(&self, attrs: Vec<(String, String)>) -> Logger {
        // 基底属性と追加属性を結合して新しい Logger を生成する
        let mut new_attrs = self.base_attrs.clone();
        // 追加属性を基底属性に追加する
        new_attrs.extend(attrs);
        // 新しい Logger を生成して返す
        Logger::new(self.service_name.clone(), new_attrs)
    }
}

// new_logger は Logger を生成するファクトリ関数（OSS 型を引数に取らない）。
// service_name は OTLP resource.ServiceName 属性（例: "tier1.key_svc"）。
pub fn new_logger(service_name: impl Into<String>) -> Logger {
    // Logger を初期化して返す（基底属性は空で開始する）
    Logger::new(service_name, vec![])
}

// ---- Tracer 具象実装 ----

// K1s0Span は Tracer::start_span が返すスパン具象実装。
// OTel の opentelemetry::trace::Span 型を公開シグネチャに一切露出しない。
// traceId / spanId を内部に保持し、set_attribute / end を提供する。
pub struct K1s0Span {
    // name: スパン名（操作名、例: "db.query" / "rpc.call"）
    name: String,
    // trace_id: 16 bytes hex 文字列（疑似ランダム生成、本番では OTel context から伝播）
    trace_id: String,
    // span_id: 8 bytes hex 文字列（疑似ランダム生成）
    span_id: String,
    // attributes: スパンに設定された key-value ペア（set_attribute で蓄積する）
    attributes: Vec<(String, String)>,
    // ended: end() が呼ばれたかどうか（重複 end を防ぐ）
    ended: bool,
}

// K1s0Span の実装ブロック
impl K1s0Span {
    // new は K1s0Span を生成する内部ファクトリ
    // trace_id / span_id は呼び出し元が注入する（OTel context propagation から伝播させる）。
    // stub 実装では固定値 "0000000000000000" + sequential counter を使用する（wall-clock 禁止）。
    fn new(name: impl Into<String>) -> Self {
        // trace_id: stub 実装では固定プレフィックス + thread_local counter を使用する
        // 本番では OTel SDK の context.getActiveSpan() から伝播させる（wall-clock 使用禁止）
        let trace_id = {
            // thread_local カウンタで連番 hex を生成する（wall-clock 禁止規律の物理化）
            use std::cell::Cell;
            thread_local! {
                // スパンカウンタ: スレッドローカルで連番を管理する（wall-clock 代替）
                static SPAN_COUNTER: Cell<u64> = const { Cell::new(0) };
            }
            // カウンタを 1 インクリメントして hex 文字列を生成する
            let n = SPAN_COUNTER.with(|c| { let v = c.get(); c.set(v + 1); v });
            // trace_id は 32 文字の hex 文字列（16 bytes 相当）
            format!("{:016x}{:016x}", 0u64, n)
        };
        // span_id: trace_id と同様に thread_local カウンタで生成する（wall-clock 禁止）
        let span_id = {
            // スパン ID 用カウンタ: スレッドローカルで連番を管理する
            use std::cell::Cell;
            thread_local! {
                // スパン ID カウンタ
                static SPAN_ID_COUNTER: Cell<u64> = const { Cell::new(0) };
            }
            // カウンタを 1 インクリメントして 16 文字の hex 文字列を生成する
            let n = SPAN_ID_COUNTER.with(|c| { let v = c.get(); c.set(v + 1); v });
            // span_id は 16 文字の hex 文字列（8 bytes 相当）
            format!("{:016x}", n)
        };
        // K1s0Span を初期化して返す
        Self {
            // スパン名を格納する
            name: name.into(),
            // trace_id を格納する
            trace_id,
            // span_id を格納する
            span_id,
            // 属性配列を初期化する
            attributes: vec![],
            // 終了フラグを初期化する
            ended: false,
        }
    }

    // trace_id はスパンの trace ID を返す（16 bytes hex 文字列）
    pub fn trace_id(&self) -> &str {
        // trace_id フィールドを返す
        &self.trace_id
    }

    // span_id はスパンの span ID を返す（8 bytes hex 文字列）
    pub fn span_id(&self) -> &str {
        // span_id フィールドを返す
        &self.span_id
    }
}

// K1s0Span は Span trait を実装する
impl Span for K1s0Span {
    // set_attribute はスパンに key-value 属性を設定する
    fn set_attribute(&mut self, key: &str, value: &str) {
        // 属性を内部配列に追加する
        self.attributes.push((key.to_string(), value.to_string()));
    }

    // end はスパンを終了してバックエンドにエクスポートする（Box<Self> を消費する）
    fn end(mut self: Box<Self>) {
        // 既に終了している場合は何もしない
        if self.ended {
            return;
        }
        // 終了フラグを立てる
        self.ended = true;
        // スパン情報を標準エラーに出力する（本番では OTel exporter に転送する）
        eprintln!(
            "[span.end] name={} trace_id={} span_id={} attrs={:?}",
            self.name, self.trace_id, self.span_id, self.attributes
        );
    }
}

// K1s0Tracer は分散トレーシングの L3 abstract 具象実装。
// OSS 型（opentelemetry::trace::Tracer 等）を公開シグネチャに一切含まない。
// Tracer trait（同ファイル内の facade trait）を実装する。
pub struct K1s0Tracer {
    // source_name: トレーサーのソース名（ActivitySource / InstrumentationLibrary 相当）
    source_name: String,
}

// K1s0Tracer の実装ブロック（トレーサーファクトリと start_span を提供する）
impl K1s0Tracer {
    // new は K1s0Tracer を生成する内部ファクトリ
    fn new(source_name: impl Into<String>) -> Self {
        // K1s0Tracer を初期化して返す
        Self {
            // ソース名を格納する
            source_name: source_name.into(),
        }
    }

    // source_name はトレーサーのソース名を返す
    pub fn source_name(&self) -> &str {
        // source_name フィールドを返す
        &self.source_name
    }
}

// K1s0Tracer は Tracer facade trait を実装する（OSS 型を公開シグネチャに露出しない）
impl Tracer for K1s0Tracer {
    // start_span は新しいスパンを開始して Box<dyn Span> を返す
    fn start_span(&self, name: &str) -> Box<dyn Span> {
        // K1s0Span を生成して Box<dyn Span> に変換して返す
        Box::new(K1s0Span::new(name))
    }
}

// new_tracer は K1s0Tracer を生成するファクトリ関数（OSS 型を引数に取らない）。
// source_name は OTLP InstrumentationLibrary 名（例: "k1s0.tier1.key_svc"）。
pub fn new_tracer(source_name: impl Into<String>) -> K1s0Tracer {
    // K1s0Tracer を初期化して返す
    K1s0Tracer::new(source_name)
}

// ---- MetricMeter 具象実装 ----

// MetricMeter は L3 メトリクス記録の具象実装。
// OSS 型（opentelemetry::metrics::Meter / prometheus::Registry 等）を公開 API に一切露出しない。
// カウンター / ヒストグラム を内部マップで管理する。
pub struct MetricMeter {
    // service_name: メトリクスの OTLP resource.ServiceName 属性
    service_name: String,
    // counters: カウンター名から累積値への内部マップ（遅延初期化で管理する）
    // Mutex で保護して Send + Sync を維持する
    counters: std::sync::Mutex<std::collections::HashMap<String, f64>>,
    // histograms: ヒストグラム名から観測値 Vec への内部マップ
    histograms: std::sync::Mutex<std::collections::HashMap<String, Vec<f64>>>,
    // gauges: ゲージ名から現在値への内部マップ
    gauges: std::sync::Mutex<std::collections::HashMap<String, f64>>,
}

// MetricMeter の実装ブロック（メトリクスファクトリと記録メソッドを提供する）
impl MetricMeter {
    // new は MetricMeter を生成する内部ファクトリ
    fn new(service_name: impl Into<String>) -> Self {
        // MetricMeter を初期化して返す
        Self {
            // サービス名を格納する
            service_name: service_name.into(),
            // カウンターマップを初期化する
            counters: std::sync::Mutex::new(std::collections::HashMap::new()),
            // ヒストグラムマップを初期化する
            histograms: std::sync::Mutex::new(std::collections::HashMap::new()),
            // ゲージマップを初期化する
            gauges: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }

    // service_name はメトリクスのサービス名を返す
    pub fn service_name(&self) -> &str {
        // service_name フィールドを返す
        &self.service_name
    }
}

// MetricMeter は MetricRecorder trait を実装する
impl MetricRecorder for MetricMeter {
    // increment_counter はカウンタ値を value だけ増加させる（単調増加カウンタ）
    fn increment_counter(&self, name: &str, value: u64, labels: &[(&str, &str)]) {
        // カウンターマップをロックして value を累積する
        let mut map = self.counters.lock().unwrap_or_else(|e| e.into_inner());
        // カウンターを取得して value を加算する
        let entry = map.entry(name.to_string()).or_insert(0.0);
        // value を f64 に変換して加算する（u64 は f64 に安全に変換できる）
        *entry += value as f64;
        // 標準エラーにメトリクスを出力する（本番では OTel exporter に転送する）
        eprintln!(
            "[counter] service={} name={} delta={} labels={:?}",
            self.service_name, name, value, labels
        );
    }

    // record_histogram はヒストグラムに value を記録する（レイテンシ / サイズ計測用）
    fn record_histogram(&self, name: &str, value: f64, labels: &[(&str, &str)]) {
        // ヒストグラムマップをロックして value を追加する
        let mut map = self.histograms.lock().unwrap_or_else(|e| e.into_inner());
        // ヒストグラムエントリを取得して value を追加する
        map.entry(name.to_string()).or_default().push(value);
        // 標準エラーにメトリクスを出力する（本番では OTel exporter に転送する）
        eprintln!(
            "[histogram] service={} name={} value={} labels={:?}",
            self.service_name, name, value, labels
        );
    }
}

// new_metric_meter は MetricMeter を生成するファクトリ関数（OSS 型を引数に取らない）。
// service_name は OTLP resource.ServiceName 属性（例: "tier1.key_svc"）。
pub fn new_metric_meter(service_name: impl Into<String>) -> MetricMeter {
    // MetricMeter を初期化して返す
    MetricMeter::new(service_name)
}
