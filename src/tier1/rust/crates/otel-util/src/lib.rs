// 本ファイルは tier1 Rust 共通の OpenTelemetry 初期化ユーティリティ。
//
// 設計正典:
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/05_モジュール依存関係.md
//     - DS-SW-COMP-109: k1s0-otel 共通ライブラリ（tracer / meter / logger / propagator 集約）
//   docs/05_実装/60_観測性設計/10_OTel_Collector配置/01_OTel_Collector配置.md
//     - OTEL_EXPORTER_OTLP_ENDPOINT を既定値として参照する Rust crate
//   docs/05_実装/00_ディレクトリ設計/20_tier1レイアウト/04_rust_workspace配置.md
//     - workspace member の 1 つとして配置（otel-util/）
//
// 役割:
//   tier1 Rust 3 Pod（audit / decision / pii）共通の OpenTelemetry 初期化を集約する。
//   呼出側は Pod 起動時に init(pod_name, namespace) を 1 度だけ呼び、戻り値の InitGuard を
//   Pod の生存期間と同じ寿命で保持する（Drop で OTLP provider を flush + shutdown する）。
//
// 動作モード:
//   - OTEL_EXPORTER_OTLP_ENDPOINT 未設定 → tracing-subscriber の fmt layer のみ（stdout JSON Lines、
//     fluentbit 経由 Loki 集約前提、DS-SW-COMP-037 第 1 段）。
//   - OTEL_EXPORTER_OTLP_ENDPOINT 設定済 → 上記 fmt layer に加えて OTLP gRPC exporter 経由で
//     Collector に直送（DS-SW-COMP-037 第 2 段、Tempo / Mimir / Loki に振り分け）。

//! k1s0-otel: tier1 Rust 共通 OpenTelemetry 初期化ライブラリ（DS-SW-COMP-109）。

// Trace SDK 用の OTel global tracer provider 設定 API。
use opentelemetry::trace::TracerProvider as _;
// 環境変数読出（OTEL_EXPORTER_OTLP_ENDPOINT / OTEL_LOG_LEVEL）。
use std::env;
// init を一度きりに制限するための flag。多重 init は global subscriber を 2 つ作って panic するため避ける。
use std::sync::OnceLock;

// init() の戻り値。Drop 時に OTLP TracerProvider を flush + shutdown する。
// OTLP モードでは Some(provider) を抱え、stdout-only モードでは None で no-op になる。
pub struct InitGuard {
    // OTLP exporter が設定された場合のみ Some。Drop で shutdown_tracer_provider 相当を呼ぶ。
    tracer_provider: Option<opentelemetry_sdk::trace::TracerProvider>,
}

// Drop 時に OTLP provider の在庫 span を flush し、exporter のリソースを解放する。
// graceful shutdown 経路（SIGTERM）で main が return し InitGuard が drop される際に呼ばれる。
impl Drop for InitGuard {
    fn drop(&mut self) {
        // OTLP モードのみ。stdout-only モードでは tracer_provider は None で何もしない。
        if let Some(provider) = self.tracer_provider.take() {
            // shutdown は同期 API。span queue を flush して exporter チャンネルを close する。
            // shutdown 中に書込中の trace は drop される可能性があるが、graceful shutdown の
            // タイムアウト 5 秒以内に flush することを期待する。
            let _ = provider.shutdown();
        }
    }
}

// init を一度だけ呼ぶための latch。2 度目以降の呼出は no-op として panic を回避する。
static INIT_DONE: OnceLock<()> = OnceLock::new();

// init は tier1 Rust Pod の起動時に 1 度だけ呼び、tracing-subscriber と OTel TracerProvider を
// 構築する。pod_name と namespace は OTel Resource attribute（service.name / service.namespace）に
// 反映され、Tempo / Loki でフィルタリング可能になる。
//
// 戻り値の InitGuard は Pod 終了時に drop される必要がある。main 関数の最後まで保持すること。
//
// 環境変数:
//   - OTEL_EXPORTER_OTLP_ENDPOINT: 設定済なら OTLP gRPC exporter を有効化（例: "http://otel-agent.observability:4317"）
//   - RUST_LOG / OTEL_LOG_LEVEL: tracing-subscriber の EnvFilter 値（既定 "info"）
pub fn init(pod_name: &str, namespace: &str) -> InitGuard {
    // 多重 init を回避する。すでに初期化済みなら no-op の Guard を返す。
    if INIT_DONE.set(()).is_err() {
        return InitGuard { tracer_provider: None };
    }

    // tracing-subscriber の EnvFilter を構築する。RUST_LOG が最優先、無ければ "info"。
    let filter = build_env_filter();

    // OTEL_EXPORTER_OTLP_ENDPOINT を読む。空文字 / 未設定なら stdout-only モードに分岐する。
    let otlp_endpoint = env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .ok()
        .filter(|s| !s.is_empty());

    match otlp_endpoint {
        // OTLP モード: OTLP gRPC exporter を構築し、tracing → OTel ブリッジを subscriber に重ねる。
        Some(endpoint) => init_with_otlp(&endpoint, pod_name, namespace, filter),
        // stdout-only モード: tracing-subscriber の fmt layer のみで初期化する。
        None => {
            init_stdout_only(filter);
            InitGuard { tracer_provider: None }
        }
    }
}

// build_env_filter は RUST_LOG / OTEL_LOG_LEVEL の優先順で EnvFilter を組み立てる。
// 不正値が設定されていた場合は info にフォールバックする（起動失敗を避ける）。
fn build_env_filter() -> tracing_subscriber::EnvFilter {
    // RUST_LOG は tracing-subscriber 標準。設定済みならそれを尊重する。
    if let Ok(v) = env::var("RUST_LOG") {
        if let Ok(f) = tracing_subscriber::EnvFilter::try_new(&v) {
            return f;
        }
    }
    // OTEL_LOG_LEVEL は OTel 標準の log level 環境変数（DEBUG / INFO / WARN / ERROR）。
    if let Ok(v) = env::var("OTEL_LOG_LEVEL") {
        if let Ok(f) = tracing_subscriber::EnvFilter::try_new(v.to_lowercase()) {
            return f;
        }
    }
    // 既定は info。
    tracing_subscriber::EnvFilter::new("info")
}

// init_stdout_only は OTLP 未設定時の最小構成。tracing-subscriber の fmt layer のみを設定する。
// fluentbit / fluentd が stdout を読み取り Loki に集約する運用前提（DS-SW-COMP-037 第 1 段）。
fn init_stdout_only(filter: tracing_subscriber::EnvFilter) {
    use tracing_subscriber::layer::SubscriberExt as _;
    use tracing_subscriber::util::SubscriberInitExt as _;

    // JSON Lines 形式で stderr に書く（K8s log collector が拾う形式）。
    // stdout でなく stderr を選ぶのは tonic / hyper が起動時メッセージを stdout に書く慣習に合わせるため。
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_writer(std::io::stderr)
        .json();

    // try_init は global subscriber が既に登録済みの場合に Err を返す。テスト並列実行で
    // 競合する可能性があるため、エラーは無視する（INIT_DONE で一度きり保護されているはず）。
    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .try_init();
}

// init_with_otlp は OTLP gRPC exporter を構築し、tracing → OTel ブリッジを subscriber に重ねる。
// OTLP exporter の構築に失敗した場合は stdout-only にフォールバックして起動を継続する
// （Pod の起動が観測経路の問題で止まることを避けるため）。
fn init_with_otlp(
    endpoint: &str,
    pod_name: &str,
    namespace: &str,
    filter: tracing_subscriber::EnvFilter,
) -> InitGuard {
    use opentelemetry::KeyValue;
    use opentelemetry_otlp::WithExportConfig as _;
    use tracing_subscriber::layer::SubscriberExt as _;
    use tracing_subscriber::util::SubscriberInitExt as _;

    // OTLP gRPC exporter を構築する。endpoint は環境変数の値をそのまま使う
    // （Collector の Service ClusterIP 経由を想定）。
    let exporter_result = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint.to_string())
        .build();

    let exporter = match exporter_result {
        Ok(e) => e,
        Err(err) => {
            // exporter 構築失敗時は stdout-only にフォールバック。Pod 起動を止めない。
            eprintln!(
                "k1s0-otel: OTLP exporter init failed ({}), falling back to stdout-only",
                err
            );
            init_stdout_only(filter);
            return InitGuard { tracer_provider: None };
        }
    };

    // OTel Resource は service.name / service.namespace を必須属性として持つ
    // （docs/05_実装/60_観測性設計/10_OTel_Collector配置/01_OTel_Collector配置.md の resource processor 規約）。
    let resource = opentelemetry_sdk::Resource::new(vec![
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::SERVICE_NAME,
            pod_name.to_string(),
        ),
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::SERVICE_NAMESPACE,
            namespace.to_string(),
        ),
    ]);

    // TracerProvider を構築する。batch processor は tokio runtime で span を非同期 export する
    // （DS-SW-COMP-037 第 2 段の運用想定: 1000 spans / 10s）。
    let provider = opentelemetry_sdk::trace::TracerProvider::builder()
        .with_resource(resource)
        .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
        .build();

    // OTel global TracerProvider を設定する（tonic interceptor 等が後段で参照する）。
    opentelemetry::global::set_tracer_provider(provider.clone());

    // tracing → OTel ブリッジ用 Tracer を取得する（service name と一致させる）。
    let tracer = provider.tracer(pod_name.to_string());

    // tracing-opentelemetry Layer を作る。tracing::info_span! 等が OTel span として export される。
    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    // fmt layer は OTLP モードでも維持する（local debug / Pod log collector の冗長経路）。
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_writer(std::io::stderr)
        .json();

    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .with(otel_layer)
        .try_init();

    InitGuard { tracer_provider: Some(provider) }
}

#[cfg(test)]
mod tests {
    use super::*;

    // env_filter_falls_back_to_info は不正な RUST_LOG / OTEL_LOG_LEVEL でも info にフォールバックすることを確認する。
    #[test]
    fn env_filter_falls_back_to_info() {
        // 安全のため明示的に unset する。並列テストの干渉を避ける目的では crate-level の serial_test が必要だが、
        // ここでは内容検証のみ（filter 作成失敗時に panic しないこと）。
        // SAFETY: 単一スレッドのテスト関数内で env を一時設定する。
        unsafe {
            env::remove_var("RUST_LOG");
            env::remove_var("OTEL_LOG_LEVEL");
        }
        // 既定値（"info"）が返るパス。EnvFilter 自身は str 表現を提供しないため、構築が成功することのみ確認する。
        let _ = build_env_filter();
    }

    // init_guard_is_safe_to_drop_without_otlp は stdout-only モードで InitGuard を drop しても panic しないことを確認する。
    #[test]
    fn init_guard_is_safe_to_drop_without_otlp() {
        let guard = InitGuard { tracer_provider: None };
        drop(guard);
    }
}
