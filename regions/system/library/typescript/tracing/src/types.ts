export interface TraceContext {
  traceId: string;
  parentId: string;
  flags: number;
}

export function toTraceparent(ctx: TraceContext): string {
  return `00-${ctx.traceId}-${ctx.parentId}-${ctx.flags.toString(16).padStart(2, '0')}`;
}

export function fromTraceparent(s: string): TraceContext | null {
  const parts = s.split('-');
  if (parts.length !== 4) return null;
  if (parts[0] !== '00') return null;
  if (parts[1].length !== 32) return null;
  if (parts[2].length !== 16) return null;
  if (parts[3].length !== 2) return null;

  const flags = parseInt(parts[3], 16);
  if (isNaN(flags)) return null;

  return {
    traceId: parts[1],
    parentId: parts[2],
    flags,
  };
}
