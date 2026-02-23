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

export function encodeCursor(id: string): string {
  return btoa(id);
}

export function decodeCursor(cursor: string): string {
  return atob(cursor);
}
