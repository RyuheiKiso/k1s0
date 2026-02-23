import { describe, it, expect } from 'vitest';
import { BufferedAuditClient } from '../src/index.js';
import type { AuditEvent } from '../src/index.js';

function makeEvent(overrides: Partial<AuditEvent> = {}): AuditEvent {
  return {
    id: 'evt-1',
    tenantId: 'tenant-1',
    actorId: 'user-1',
    action: 'create',
    resourceType: 'document',
    resourceId: 'doc-1',
    timestamp: new Date().toISOString(),
    ...overrides,
  };
}

describe('BufferedAuditClient', () => {
  it('イベントを記録できる', async () => {
    const client = new BufferedAuditClient();
    await client.record(makeEvent());
    const events = await client.flush();
    expect(events).toHaveLength(1);
    expect(events[0].action).toBe('create');
  });

  it('flushでバッファがクリアされる', async () => {
    const client = new BufferedAuditClient();
    await client.record(makeEvent({ id: 'evt-1' }));
    await client.record(makeEvent({ id: 'evt-2' }));
    const first = await client.flush();
    expect(first).toHaveLength(2);
    const second = await client.flush();
    expect(second).toHaveLength(0);
  });

  it('複数イベントを順序通りに返す', async () => {
    const client = new BufferedAuditClient();
    await client.record(makeEvent({ action: 'create' }));
    await client.record(makeEvent({ action: 'update' }));
    await client.record(makeEvent({ action: 'delete' }));
    const events = await client.flush();
    expect(events.map((e) => e.action)).toEqual(['create', 'update', 'delete']);
  });

  it('空バッファでflushすると空配列を返す', async () => {
    const client = new BufferedAuditClient();
    const events = await client.flush();
    expect(events).toEqual([]);
  });
});
