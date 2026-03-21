// AES-GCM 暗号化セッションストア（S-04 対応）
// SESSION_ENCRYPTION_KEY 環境変数から 32 バイト（AES-256）の鍵を使用して
// SessionData を暗号化してから Redis に保存する。
// 鍵が未設定の場合は通常の RedisStore にフォールバックする（main.go 参照）。
package session

import (
	"context"
	"crypto/aes"
	"crypto/cipher"
	"crypto/rand"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"io"
	"time"

	"github.com/google/uuid"
	"github.com/redis/go-redis/v9"
)

// EncryptedStore は AES-GCM で SessionData を暗号化して Redis に保存するセッションストア。
// Store インターフェースを完全に実装し、RedisStore の代替として使用できる。
type EncryptedStore struct {
	client redis.Cmdable
	prefix string
	// key は AES-256-GCM の暗号化鍵（32 バイト固定）。
	key []byte
}

// NewEncryptedStore は AES-GCM 暗号化を使用するセッションストアを作成する。
// key は正確に 32 バイト（AES-256）でなければならない。
// SESSION_ENCRYPTION_KEY 環境変数が設定されている場合は main.go から呼び出す。
func NewEncryptedStore(client redis.Cmdable, prefix string, key []byte) (*EncryptedStore, error) {
	if len(key) != 32 {
		return nil, fmt.Errorf("暗号化キーは 32 バイト（AES-256）である必要があります: 受け取ったバイト数 %d", len(key))
	}
	if prefix == "" {
		prefix = "bff:session:"
	}
	// 外部スライスへの依存を避けるためにキーをコピーする
	keyCopy := make([]byte, 32)
	copy(keyCopy, key)
	return &EncryptedStore{client: client, prefix: prefix, key: keyCopy}, nil
}

// redisKey はセッション ID に prefix を付与した Redis キーを返す。
func (s *EncryptedStore) redisKey(id string) string {
	return s.prefix + id
}

// encrypt は plaintext を AES-GCM で暗号化し、base64 URL エンコード文字列を返す。
// 形式: base64url(nonce || ciphertext || auth_tag)
// nonce はリクエストごとに乱数生成する（GCM NonceSize = 12 バイト）。
func (s *EncryptedStore) encrypt(plaintext []byte) (string, error) {
	block, err := aes.NewCipher(s.key)
	if err != nil {
		return "", fmt.Errorf("AES cipher 初期化に失敗: %w", err)
	}
	gcm, err := cipher.NewGCM(block)
	if err != nil {
		return "", fmt.Errorf("GCM 初期化に失敗: %w", err)
	}

	// nonce を暗号論的乱数で生成する
	nonce := make([]byte, gcm.NonceSize())
	if _, err := io.ReadFull(rand.Reader, nonce); err != nil {
		return "", fmt.Errorf("nonce の生成に失敗: %w", err)
	}

	// nonce をプレフィックスとして ciphertext に付加する（復号時に先頭から取り出す）
	ciphertext := gcm.Seal(nonce, nonce, plaintext, nil)
	return base64.RawURLEncoding.EncodeToString(ciphertext), nil
}

// decrypt は base64 URL エンコードされた暗号文を復号して plaintext を返す。
// nonce が先頭 gcm.NonceSize() バイトに格納されていることを前提とする。
func (s *EncryptedStore) decrypt(encoded string) ([]byte, error) {
	data, err := base64.RawURLEncoding.DecodeString(encoded)
	if err != nil {
		return nil, fmt.Errorf("base64 デコードに失敗: %w", err)
	}

	block, err := aes.NewCipher(s.key)
	if err != nil {
		return nil, fmt.Errorf("AES cipher 初期化に失敗: %w", err)
	}
	gcm, err := cipher.NewGCM(block)
	if err != nil {
		return nil, fmt.Errorf("GCM 初期化に失敗: %w", err)
	}

	nonceSize := gcm.NonceSize()
	if len(data) < nonceSize {
		return nil, fmt.Errorf("暗号文が短すぎます（nonce を取り出せません）")
	}

	// 先頭の nonce と残りの ciphertext+auth_tag を分離して復号する
	nonce, ciphertext := data[:nonceSize], data[nonceSize:]
	plaintext, err := gcm.Open(nil, nonce, ciphertext, nil)
	if err != nil {
		return nil, fmt.Errorf("GCM 復号に失敗（鍵不一致または改ざん）: %w", err)
	}
	return plaintext, nil
}

// Create は SessionData を暗号化して Redis に保存し、新規セッション ID を返す。
func (s *EncryptedStore) Create(ctx context.Context, data *SessionData, ttl time.Duration) (string, error) {
	id := uuid.New().String()
	data.CreatedAt = time.Now().Unix()

	b, err := json.Marshal(data)
	if err != nil {
		return "", fmt.Errorf("セッションのシリアライズに失敗: %w", err)
	}

	encrypted, err := s.encrypt(b)
	if err != nil {
		return "", fmt.Errorf("セッションの暗号化に失敗: %w", err)
	}

	if err := s.client.Set(ctx, s.redisKey(id), encrypted, ttl).Err(); err != nil {
		return "", fmt.Errorf("セッションの保存に失敗: %w", err)
	}
	return id, nil
}

// Get は Redis からセッションを取得し、復号して返す。存在しない場合は nil を返す。
func (s *EncryptedStore) Get(ctx context.Context, id string) (*SessionData, error) {
	val, err := s.client.Get(ctx, s.redisKey(id)).Result()
	if err == redis.Nil {
		return nil, nil
	}
	if err != nil {
		return nil, fmt.Errorf("セッションの取得に失敗: %w", err)
	}

	plaintext, err := s.decrypt(val)
	if err != nil {
		return nil, fmt.Errorf("セッションの復号に失敗: %w", err)
	}

	var data SessionData
	if err := json.Unmarshal(plaintext, &data); err != nil {
		return nil, fmt.Errorf("セッションのデシリアライズに失敗: %w", err)
	}
	return &data, nil
}

// Update は既存セッションデータを暗号化して上書きする。
func (s *EncryptedStore) Update(ctx context.Context, id string, data *SessionData, ttl time.Duration) error {
	b, err := json.Marshal(data)
	if err != nil {
		return fmt.Errorf("セッションのシリアライズに失敗: %w", err)
	}

	encrypted, err := s.encrypt(b)
	if err != nil {
		return fmt.Errorf("セッションの暗号化に失敗: %w", err)
	}

	if err := s.client.Set(ctx, s.redisKey(id), encrypted, ttl).Err(); err != nil {
		return fmt.Errorf("セッションの更新に失敗: %w", err)
	}
	return nil
}

// Delete はセッションを Redis から削除する。
func (s *EncryptedStore) Delete(ctx context.Context, id string) error {
	if err := s.client.Del(ctx, s.redisKey(id)).Err(); err != nil {
		return fmt.Errorf("セッションの削除に失敗: %w", err)
	}
	return nil
}

// Touch はセッションの TTL を延長する（スライディング有効期限）。
func (s *EncryptedStore) Touch(ctx context.Context, id string, ttl time.Duration) error {
	if err := s.client.Expire(ctx, s.redisKey(id), ttl).Err(); err != nil {
		return fmt.Errorf("セッション TTL の延長に失敗: %w", err)
	}
	return nil
}
