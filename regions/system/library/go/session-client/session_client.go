package sessionclient

import (
	"context"
	"crypto/rand"
	"encoding/hex"
	"fmt"
	"sync"
	"time"
)

// Session はセッション情報。
type Session struct {
	ID        string            `json:"id"`
	UserID    string            `json:"user_id"`
	Token     string            `json:"token"`
	ExpiresAt time.Time         `json:"expires_at"`
	CreatedAt time.Time         `json:"created_at"`
	Revoked   bool              `json:"revoked"`
	Metadata  map[string]string `json:"metadata,omitempty"`
}

// CreateSessionRequest はセッション作成リクエスト。
type CreateSessionRequest struct {
	UserID     string            `json:"user_id"`
	TTLSeconds int64             `json:"ttl_seconds"`
	Metadata   map[string]string `json:"metadata,omitempty"`
}

// RefreshSessionRequest はセッション更新リクエスト。
type RefreshSessionRequest struct {
	ID         string `json:"id"`
	TTLSeconds int64  `json:"ttl_seconds"`
}

// SessionClient はセッションクライアントのインターフェース。
type SessionClient interface {
	Create(ctx context.Context, req CreateSessionRequest) (*Session, error)
	Get(ctx context.Context, id string) (*Session, error)
	Refresh(ctx context.Context, req RefreshSessionRequest) (*Session, error)
	Revoke(ctx context.Context, id string) error
	ListUserSessions(ctx context.Context, userID string) ([]*Session, error)
	RevokeAll(ctx context.Context, userID string) (int, error)
}

// InMemorySessionClient はメモリ内のセッションクライアント。
type InMemorySessionClient struct {
	sessions map[string]*Session
	mu       sync.RWMutex
}

// NewInMemorySessionClient は新しい InMemorySessionClient を生成する。
func NewInMemorySessionClient() *InMemorySessionClient {
	return &InMemorySessionClient{
		sessions: make(map[string]*Session),
	}
}

func generateID() string {
	b := make([]byte, 16)
	_, _ = rand.Read(b)
	return hex.EncodeToString(b)
}

// Create はセッションを作成する。
func (c *InMemorySessionClient) Create(_ context.Context, req CreateSessionRequest) (*Session, error) {
	c.mu.Lock()
	defer c.mu.Unlock()

	now := time.Now()
	session := &Session{
		ID:        generateID(),
		UserID:    req.UserID,
		Token:     generateID(),
		ExpiresAt: now.Add(time.Duration(req.TTLSeconds) * time.Second),
		CreatedAt: now,
		Revoked:   false,
		Metadata:  req.Metadata,
	}
	c.sessions[session.ID] = session
	return session, nil
}

// Get はセッションを取得する。
func (c *InMemorySessionClient) Get(_ context.Context, id string) (*Session, error) {
	c.mu.RLock()
	defer c.mu.RUnlock()
	session, ok := c.sessions[id]
	if !ok {
		return nil, fmt.Errorf("session not found: %s", id)
	}
	return session, nil
}

// Refresh はセッションの有効期限を更新する。
func (c *InMemorySessionClient) Refresh(_ context.Context, req RefreshSessionRequest) (*Session, error) {
	c.mu.Lock()
	defer c.mu.Unlock()
	session, ok := c.sessions[req.ID]
	if !ok {
		return nil, fmt.Errorf("session not found: %s", req.ID)
	}
	session.ExpiresAt = time.Now().Add(time.Duration(req.TTLSeconds) * time.Second)
	session.Token = generateID()
	return session, nil
}

// Revoke はセッションを無効化する。
func (c *InMemorySessionClient) Revoke(_ context.Context, id string) error {
	c.mu.Lock()
	defer c.mu.Unlock()
	session, ok := c.sessions[id]
	if !ok {
		return fmt.Errorf("session not found: %s", id)
	}
	session.Revoked = true
	return nil
}

// ListUserSessions はユーザーのセッション一覧を返す。
func (c *InMemorySessionClient) ListUserSessions(_ context.Context, userID string) ([]*Session, error) {
	c.mu.RLock()
	defer c.mu.RUnlock()
	var result []*Session
	for _, s := range c.sessions {
		if s.UserID == userID {
			result = append(result, s)
		}
	}
	return result, nil
}

// RevokeAll はユーザーの全セッションを無効化し、無効化した数を返す。
func (c *InMemorySessionClient) RevokeAll(_ context.Context, userID string) (int, error) {
	c.mu.Lock()
	defer c.mu.Unlock()
	count := 0
	for _, s := range c.sessions {
		if s.UserID == userID && !s.Revoked {
			s.Revoked = true
			count++
		}
	}
	return count, nil
}
