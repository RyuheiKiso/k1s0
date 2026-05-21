// k1s0 tier3 conflicts screen — 4 subtype 分岐 UI の index
// 11_クライアント状態適合仕様.md §5 CI 整合: subtype と UI 分岐 1:1 固定
// ui-components パッケージの 4 subtype component を re-export する
// (tier3 CLAUDE.md §業務 entity の独自再宣言型禁止 — ui-components 経由のみ使用)

// ThreeWayMergeUi: lost_update subtype の 3-way merge UI
export { ThreeWayMergeUi } from "@k1s0/ui-components";
// BusinessErrorPanel: stale_write / concurrent_edit subtype の error panel
export { BusinessErrorPanel } from "@k1s0/ui-components";
// ConcurrentEditDialog: concurrent_edit subtype の user choice dialog
export { ConcurrentEditDialog } from "@k1s0/ui-components";
// AutoResendBadge: auto-resend with chained idempotency key バッジ
export { AutoResendBadge } from "@k1s0/ui-components";
