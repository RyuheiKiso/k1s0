export interface PageRequest {
  page: number;
  perPage: number;
}

export interface PageResponse<T> {
  items: T[];
  total: number;
  page: number;
  perPage: number;
  totalPages: number;
}

export interface PaginationMeta {
  total: number;
  page: number;
  perPage: number;
  totalPages: number;
}

export interface CursorRequest {
  cursor?: string;
  limit: number;
}

export interface CursorMeta {
  nextCursor?: string;
  hasMore: boolean;
}

const MIN_PER_PAGE = 1;
const MAX_PER_PAGE = 100;

export class PerPageValidationError extends Error {
  constructor(value: number) {
    super(
      `invalid perPage: ${value} (must be between ${MIN_PER_PAGE} and ${MAX_PER_PAGE})`,
    );
    this.name = "PerPageValidationError";
  }
}

export function validatePerPage(perPage: number): number {
  if (perPage < MIN_PER_PAGE || perPage > MAX_PER_PAGE) {
    throw new PerPageValidationError(perPage);
  }
  return perPage;
}

export function createPageResponse<T>(
  items: T[],
  total: number,
  req: PageRequest,
): PageResponse<T> {
  return {
    items,
    total,
    page: req.page,
    perPage: req.perPage,
    totalPages: Math.ceil(total / req.perPage),
  };
}

const CURSOR_SEPARATOR = "|";

export function encodeCursor(sortKey: string, id: string): string {
  return btoa(`${sortKey}${CURSOR_SEPARATOR}${id}`);
}

export function defaultPageRequest(): PageRequest {
  return { page: 1, perPage: 20 };
}

export function pageOffset(req: PageRequest): number {
  return (req.page - 1) * req.perPage;
}

export function hasNextPage(req: PageRequest, total: number): boolean {
  return req.page * req.perPage < total;
}

export function decodeCursor(cursor: string): { sortKey: string; id: string } {
  const decoded = atob(cursor);
  const idx = decoded.indexOf(CURSOR_SEPARATOR);
  if (idx < 0) {
    throw new Error("invalid cursor: missing separator");
  }
  return {
    sortKey: decoded.substring(0, idx),
    id: decoded.substring(idx + 1),
  };
}
