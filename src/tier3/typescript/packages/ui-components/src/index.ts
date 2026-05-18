// k1s0 tier3 SPA 共通 UI コンポーネントパッケージ公開エントリポイント
// 全 10 コンポーネントを一括 re-export する
// React 18 + TypeScript strict / WCAG 2.1 AA 準拠

// ErrorBoundary コンポーネントと型を export する
export { ErrorBoundary } from "./ErrorBoundary";
// ErrorBoundary プロパティ型を export する
export type { ErrorBoundaryProps } from "./ErrorBoundary";

// BusinessErrorPanel コンポーネントと型を export する
export { BusinessErrorPanel } from "./BusinessErrorPanel";
// BusinessErrorPanel プロパティ型 / エラー型 / 重大度型を export する
export type {
  BusinessErrorPanelProps,
  BusinessError,
  BusinessErrorSeverity,
} from "./BusinessErrorPanel";

// LoadingSpinner コンポーネントと型を export する
export { LoadingSpinner } from "./LoadingSpinner";
// LoadingSpinner プロパティ型を export する
export type { LoadingSpinnerProps } from "./LoadingSpinner";

// OptimisticBadge コンポーネントと型を export する
export { OptimisticBadge } from "./OptimisticBadge";
// OptimisticBadge プロパティ型 / 状態型を export する
export type { OptimisticBadgeProps, OptimisticStatus } from "./OptimisticBadge";

// SearchList コンポーネントと型を export する
export { SearchList } from "./SearchList";
// SearchList プロパティ型 / アイテム型を export する
export type { SearchListProps, SearchListItem } from "./SearchList";

// Pagination コンポーネントと型を export する
export { Pagination } from "./Pagination";
// Pagination プロパティ型を export する
export type { PaginationProps } from "./Pagination";

// Filter コンポーネントと型を export する
export { Filter } from "./Filter";
// Filter プロパティ型 / オプション型を export する
export type { FilterProps, FilterOption } from "./Filter";

// StateTransitionIndicator コンポーネントと型を export する
export { StateTransitionIndicator } from "./StateTransitionIndicator";
// StateTransitionIndicator プロパティ型 / ステップ型を export する
export type {
  StateTransitionIndicatorProps,
  TransitionStep,
} from "./StateTransitionIndicator";

// PermissionGate コンポーネントと型を export する
export { PermissionGate } from "./PermissionGate";
// PermissionGate プロパティ型 / 権限型を export する
export type { PermissionGateProps, Permission } from "./PermissionGate";

// RequireRole コンポーネントと型を export する
export { RequireRole } from "./RequireRole";
// RequireRole プロパティ型 / ロール型を export する
export type { RequireRoleProps, Role } from "./RequireRole";
