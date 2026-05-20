// index.ts — @k1s0/hlc-lib のパブリック API エクスポート定義
// TypeScript 利用者はこのファイルのみを import する（内部実装への直接参照禁止）。
// Rust 実装（src/client/hlc_lib/rust/src/lib.rs）と同等の公開 API を再エクスポートする。

// HlcTimestamp クラスと HlcClock クラスを hlc.ts からエクスポートする
export { HlcTimestamp, HlcClock } from './hlc.js';
