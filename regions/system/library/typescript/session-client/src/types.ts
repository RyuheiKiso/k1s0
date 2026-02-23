export interface Session {
  id: string;
  userId: string;
  token: string;
  expiresAt: Date;
  createdAt: Date;
  revoked: boolean;
  metadata: Record<string, string>;
}

export interface CreateSessionRequest {
  userId: string;
  ttlSeconds: number;
  metadata?: Record<string, string>;
}

export interface RefreshSessionRequest {
  id: string;
  ttlSeconds: number;
}
