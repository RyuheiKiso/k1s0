// profiling.rs — k1s0 tier1 Library: プロファイリング L1+ facade trait
// Pyroscope 等の OSS 型を公開 API に露出しない（L1+ ラップ規約）。
// プロファイリングセッションはラベル付きで開始して end() で終了する。
// 全 trait は Send + Sync を要求する（スレッド安全性の強制）。

// Profiler は Pyroscope を L1+ ラップするプロファイリング facade trait。
// 公開 API シグネチャに OSS 型（pyroscope::PyroscopeAgent 等）を一切含まない。
pub trait Profiler: Send + Sync {
    // start_session はプロファイリングセッションを開始して Box<dyn ProfilingSession> を返す。
    // labels は識別用のメタデータ（例: &[("service", "tier1"), ("tenant_id", "t-001")]）。
    // セッションは end() を呼ぶまで継続する（end() 忘れはプロファイリングデータ損失の原因）。
    fn start_session(&self, labels: &[(&str, &str)]) -> Box<dyn ProfilingSession>;
}

// ProfilingSession は進行中のプロファイリングセッションを表す trait。
// start_session で開始したセッションは end() で明示的に終了すること。
pub trait ProfilingSession: Send + Sync {
    // end はプロファイリングセッションを終了してデータをバックエンドに送信する。
    // Box<Self> を消費するため、end() 後はセッションオブジェクトを使用できない。
    fn end(self: Box<Self>);
}
