// pagination.ts — cursor-based pagination helper 関数（forms パッケージ独立モジュール）
// tier3 フォーム系画面のリスト取得で使用する cursor pagination を提供する
// offset/limit ベースのページングではなく cursor ベースを採用することで整合性を保証する
// tenant_id をクエリパラメータ / 型に含めることを禁止する（tier3/CLAUDE.md 規約）

// CursorPageRequest: cursor-based pagination のリクエスト型
export interface CursorPageRequest {
  // 取得開始カーソル（前ページの最終レコードの cursor 値。初回は undefined）
  readonly after?: string;
  // 取得件数（1 以上の正の整数。最大は API 側で制限する）
  readonly limit: number;
}

// CursorPageResponse: cursor-based pagination のレスポンス型
// T: ページ内のアイテム型（tenant_id を含まない業務エンティティ）
export interface CursorPageResponse<T> {
  // ページ内のアイテム一覧
  readonly items: readonly T[];
  // 次ページの先頭カーソル（次ページが存在しない場合は null）
  readonly nextCursor: string | null;
  // 前ページの先頭カーソル（前ページが存在しない場合は null）
  readonly prevCursor: string | null;
  // 次ページが存在するかどうか
  readonly hasNextPage: boolean;
  // 前ページが存在するかどうか
  readonly hasPrevPage: boolean;
}

// CursorPaginationState: cursor pagination の状態管理型
// cursor の履歴をスタックで保持することで前ページ遷移を実現する
export interface CursorPaginationState {
  // 現在のページのカーソル（undefined は先頭ページ）
  readonly currentCursor: string | undefined;
  // カーソル履歴スタック（前ページ遷移に使用する。LIFO 順）
  readonly cursorStack: readonly string[];
  // 1 ページあたりの取得件数
  readonly pageSize: number;
}

// createInitialPaginationState: 初期 cursor pagination 状態を生成する
// pageSize: 1 ページあたりの取得件数（デフォルト 20）
export function createInitialPaginationState(
  // 1 ページあたりの取得件数を受け取る（省略時は 20）
  pageSize = 20
): CursorPaginationState {
  // 初期状態を生成して返す（先頭ページから開始する）
  return {
    // 現在のカーソルは先頭ページなので undefined
    currentCursor: undefined,
    // カーソル履歴スタックは空
    cursorStack: [],
    // 1 ページあたりの取得件数を設定する
    pageSize,
  };
}

// buildPageRequest: 現在の pagination 状態から CursorPageRequest を生成する
// state: 現在の cursor pagination 状態
export function buildPageRequest(state: CursorPaginationState): CursorPageRequest {
  // 現在のカーソルと件数からリクエストを生成して返す
  return {
    // 現在のカーソルを after パラメータに設定する（先頭ページは undefined）
    after: state.currentCursor,
    // 1 ページあたりの取得件数を設定する
    limit: state.pageSize,
  };
}

// advanceToNextPage: 次ページに進む状態を返す
// state: 現在の cursor pagination 状態
// nextCursor: レスポンスの nextCursor 値
export function advanceToNextPage(
  // 現在の状態を受け取る
  state: CursorPaginationState,
  // 次ページのカーソルを受け取る（null の場合は次ページなし）
  nextCursor: string | null
): CursorPaginationState {
  // 次ページが存在しない場合は現在の状態を返す
  if (nextCursor === null) {
    // 次ページが存在しないため現在の状態を返す
    return state;
  }
  // 現在のカーソルをスタックに push して次ページへ進む
  return {
    // 次ページのカーソルを現在のカーソルに設定する
    currentCursor: nextCursor,
    // 現在のカーソルをスタックに積む（前ページ遷移のため）
    cursorStack: state.currentCursor !== undefined
      // 現在のカーソルが存在する場合はスタックに追加する
      ? [...state.cursorStack, state.currentCursor]
      // 先頭ページ（currentCursor が undefined）の場合はスタックに積まない
      : state.cursorStack,
    // 1 ページあたりの取得件数は変更しない
    pageSize: state.pageSize,
  };
}

// retreatToPrevPage: 前ページに戻る状態を返す
// state: 現在の cursor pagination 状態
export function retreatToPrevPage(
  // 現在の状態を受け取る
  state: CursorPaginationState
): CursorPaginationState {
  // スタックが空の場合は先頭ページに戻る（戻れない場合は現在の状態を返す）
  if (state.cursorStack.length === 0) {
    // 先頭ページなので現在の状態を返す（それ以上戻れない）
    return state;
  }
  // スタックから最後のカーソルを取り出して前ページに戻る
  const prevCursorStack = state.cursorStack.slice(0, -1);
  // スタックの最後の要素が前ページのカーソル
  const prevCursor = state.cursorStack[state.cursorStack.length - 1];
  // 前ページに戻った状態を返す
  return {
    // スタックから取り出したカーソルを現在のカーソルに設定する
    currentCursor: prevCursor,
    // スタックから最後の要素を取り除く
    cursorStack: prevCursorStack,
    // 1 ページあたりの取得件数は変更しない
    pageSize: state.pageSize,
  };
}

// resetToFirstPage: 先頭ページに戻る状態を返す
// state: 現在の cursor pagination 状態
export function resetToFirstPage(
  // 現在の状態を受け取る
  state: CursorPaginationState
): CursorPaginationState {
  // 先頭ページに戻った状態を返す
  return {
    // 先頭ページのカーソルは undefined
    currentCursor: undefined,
    // カーソル履歴スタックをクリアする
    cursorStack: [],
    // 1 ページあたりの取得件数は変更しない
    pageSize: state.pageSize,
  };
}

// isFirstPage: 現在のページが先頭ページかどうかを返す
// state: 現在の cursor pagination 状態
export function isFirstPage(state: CursorPaginationState): boolean {
  // currentCursor が undefined かつスタックが空のとき先頭ページと判定する
  return state.currentCursor === undefined && state.cursorStack.length === 0;
}
