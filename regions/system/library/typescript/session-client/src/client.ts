import type { Session, CreateSessionRequest, RefreshSessionRequest } from './types.js';

export interface SessionClient {
  create(req: CreateSessionRequest): Promise<Session>;
  get(id: string): Promise<Session | null>;
  refresh(req: RefreshSessionRequest): Promise<Session>;
  revoke(id: string): Promise<void>;
  listUserSessions(userId: string): Promise<Session[]>;
  revokeAll(userId: string): Promise<number>;
}

export class InMemorySessionClient implements SessionClient {
  private sessions = new Map<string, Session>();

  async create(req: CreateSessionRequest): Promise<Session> {
    const now = new Date();
    const session: Session = {
      id: crypto.randomUUID(),
      userId: req.userId,
      token: crypto.randomUUID(),
      expiresAt: new Date(now.getTime() + req.ttlSeconds * 1000),
      createdAt: now,
      revoked: false,
      metadata: req.metadata ?? {},
    };
    this.sessions.set(session.id, session);
    return session;
  }

  async get(id: string): Promise<Session | null> {
    return this.sessions.get(id) ?? null;
  }

  async refresh(req: RefreshSessionRequest): Promise<Session> {
    const session = this.sessions.get(req.id);
    if (!session) {
      throw new Error(`Session not found: ${req.id}`);
    }
    session.expiresAt = new Date(Date.now() + req.ttlSeconds * 1000);
    session.token = crypto.randomUUID();
    return session;
  }

  async revoke(id: string): Promise<void> {
    const session = this.sessions.get(id);
    if (!session) {
      throw new Error(`Session not found: ${id}`);
    }
    session.revoked = true;
  }

  async listUserSessions(userId: string): Promise<Session[]> {
    return Array.from(this.sessions.values()).filter((s) => s.userId === userId);
  }

  async revokeAll(userId: string): Promise<number> {
    let count = 0;
    for (const session of this.sessions.values()) {
      if (session.userId === userId && !session.revoked) {
        session.revoked = true;
        count++;
      }
    }
    return count;
  }
}
