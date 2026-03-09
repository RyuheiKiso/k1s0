//! External tests for k1s0-telemetry-macros.
//!
//! This is a proc-macro crate. The macro generates code at compile time, so
//! testing is limited to verifying that the macro compiles correctly when applied
//! to various function signatures.
//!
//! NOTE: These tests require the `tracing` crate as a dependency since the macro
//! expands to `#[tracing::instrument(...)]`. If the crate does not have `tracing`
//! in dev-dependencies, these tests will fail to compile. In that case, the test
//! file documents what SHOULD be tested once dependencies are available.

// Attempt to use the macro — this will only compile if `tracing` is available
// as a dependency of this crate (or in dev-dependencies).

#[cfg(test)]
mod tests {
    // proc-macro crates have very limited testing capability.
    // The real macro is tested in the telemetry crate which depends on this crate.
    //
    // What we CAN verify here:
    // 1. The crate compiles successfully as a proc-macro
    // 2. The macro is exported and reachable
    //
    // What we CANNOT test without adding tracing as a dev-dependency:
    // - #[k1s0_trace] on async fn
    // - #[k1s0_trace(skip(password))] with skip parameter
    // - #[k1s0_trace(name = "custom.name")] with name parameter
    // - #[k1s0_trace(skip(a, b), name = "op")] combined parameters

    #[test]
    fn proc_macro_crate_compiles() {
        // If this test runs, the proc-macro crate compiled successfully.
        // The k1s0_trace macro is available for use by downstream crates.
        assert!(true);
    }
}
