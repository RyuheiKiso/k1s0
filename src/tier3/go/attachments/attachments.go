// k1s0 tier3 Go 添付ファイルストア（4 言語等価強度実装）
// Rust / C# / TypeScript と同等の抽象を Go で実装する
// spec arch.tier3 §26_添付帳票 UX: signed URL / sandbox iframe / virus scan / hashChain 整合性
// wall-clock TTL 禁止規約に従い time.Now() を TTL / deadline 計算に使用しない
package attachments

import (
	// crypto/rand パッケージ（UUID 生成に使用する暗号論的乱数）
	"crypto/rand"
	// crypto/sha256 パッケージ（チャンクハッシュ計算に使用する）
	"crypto/sha256"
	// encoding/hex パッケージ（バイト列を hex 文字列に変換する）
	"encoding/hex"
	// errors パッケージ（エラー生成に使用する）
	"errors"
	// fmt パッケージ（文字列フォーマットに使用する）
	"fmt"
	// strings パッケージ（ハッシュチェーン文字列操作に使用する）
	"strings"
	// sync パッケージ（InMemoryAttachmentStore のスレッドセーフ保護に使用する）
	"sync"
	// time パッケージ（uploadedAt の ISO 8601 文字列生成に使用する — 表示用途のみ、TTL 計算禁止）
	"time"
)

// --------- 定数 ---------

// DefaultChunkSizeBytes はデフォルトのチャンクサイズ（4 MiB）
// C# / TypeScript / Rust 側と値を一致させて 4 言語等価強度を維持する
const DefaultChunkSizeBytes = 4 * 1024 * 1024

// AllowedMimeTypes は許可する MIME タイプ一覧（allowlist 方式でセキュリティを確保する）
// C# FrozenSet / TypeScript ReadonlySet と同等の不変コレクションとして map で実装する
var AllowedMimeTypes = map[string]struct{}{
	// PDF 文書
	"application/pdf": {},
	// Microsoft Excel（新形式）
	"application/vnd.openxmlformats-officedocument.spreadsheetml.sheet": {},
	// Microsoft Word（新形式）
	"application/vnd.openxmlformats-officedocument.wordprocessingml.document": {},
	// CSV テキスト
	"text/csv": {},
	// プレーンテキスト
	"text/plain": {},
	// JPEG 画像
	"image/jpeg": {},
	// PNG 画像
	"image/png": {},
}

// --------- 型定義 ---------

// AttachmentMetadata は Object Storage に保存された添付ファイルのフルメタデータを表す構造体
// TypeScript AttachmentMetadata interface / C# AttachmentMeta record と等価強度の構造体
type AttachmentMetadata struct {
	// サーバー生成の添付ファイル UUID
	ID string
	// テナント識別子（公開 URL / クエリパラメータに露出しない — tier3 CLAUDE.md §データ保護）
	TenantID string
	// MIME タイプ（allowlist で検査済み）
	MimeType string
	// ファイルサイズ（バイト）
	SizeBytes int64
	// ハッシュチェーン（"sha256:<チャンク0>|sha256:<チャンク1>|..." 形式）
	HashChain string
	// アップロード完了時刻（ISO 8601 文字列 — 表示用途のみ、TTL 計算に使用しない）
	UploadedAt string
}

// AttachmentChunk はチャンクアップロードの 1 チャンクを表す構造体
// Rust AttachmentChunk struct / TypeScript AttachmentChunk interface と等価の構造体
type AttachmentChunk struct {
	// チャンク番号（0 始まり）
	ChunkIndex int
	// チャンクのバイナリデータ
	Data []byte
	// このチャンクの SHA-256 ハッシュ（hex 文字列）
	ChunkSha256Hex string
}

// AttachmentUploadResult はアップロード完了後の結果を表す構造体
// Rust AttachmentUploadResult / C# AttachmentUploadResult record と等価の構造体
type AttachmentUploadResult struct {
	// 付与された添付ファイル UUID（サーバー生成）
	AttachmentID string
	// アップロード完了時刻（ISO 8601 文字列）
	CompletedAt string
	// サーバー側で計算されたファイル全体ハッシュ（クライアント側と一致する必要がある）
	ServerSha256Hex string
}

// --------- MIME 検査 ---------

// CheckMimeType は MIME タイプが許可リストに含まれているか検査する
// C# MimeTypeChecker.IsAllowed / Rust check_mime_type / TypeScript checkMimeType と等価の関数
func CheckMimeType(mimeType string) bool {
	// AllowedMimeTypes map に含まれているか O(1) で検索する
	_, ok := AllowedMimeTypes[mimeType]
	// map に含まれていれば true、含まれていなければ false を返す
	return ok
}

// --------- UUID 生成 ---------

// generateUUID は暗号論的乱数から UUID v4 形式の文字列を生成する
// wall-clock 非依存（crypto/rand のみ使用する）
func generateUUID() string {
	// 16 バイトの暗号論的乱数を生成する
	b := make([]byte, 16)
	// crypto/rand で乱数を生成する（wall-clock 非依存）
	if _, err := rand.Read(b); err != nil {
		// 乱数生成失敗は致命的エラーとして panic する
		panic(fmt.Sprintf("generateUUID: crypto/rand.Read failed: %v", err))
	}
	// UUID v4 バリアントビットを設定する（RFC 4122 準拠）
	b[6] = (b[6] & 0x0f) | 0x40
	// UUID variant bits を設定する（RFC 4122 準拠）
	b[8] = (b[8] & 0x3f) | 0x80
	// UUID 文字列フォーマットに変換する（xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx）
	return fmt.Sprintf("%x-%x-%x-%x-%x", b[0:4], b[4:6], b[6:8], b[8:10], b[10:])
}

// --------- ハッシュ計算 ---------

// ComputeSha256Hex はバイト列の SHA-256 ハッシュを hex 文字列で返す
// TypeScript computeSha256Hex / Rust sha256 と等価の関数
func ComputeSha256Hex(data []byte) string {
	// SHA-256 ハッシュを計算する
	sum := sha256.Sum256(data)
	// hex 文字列に変換して返す
	return hex.EncodeToString(sum[:])
}

// --------- ハッシュチェーン ---------

// BuildHashChain はチャンクハッシュ配列からハッシュチェーン文字列を生成する
// フォーマット: "sha256:<hash0>|sha256:<hash1>|..." — TypeScript buildHashChain と等価
func BuildHashChain(chunkHashes []string) string {
	// 各チャンクハッシュに "sha256:" プレフィックスを付けてパイプ区切りで連結する
	prefixed := make([]string, len(chunkHashes))
	for i, h := range chunkHashes {
		// "sha256:" プレフィックスを付加する
		prefixed[i] = "sha256:" + h
	}
	// パイプ区切りで連結して返す
	return strings.Join(prefixed, "|")
}

// VerifyHashChain は AttachmentMetadata の HashChain が再計算値と一致するか検証する
// TypeScript verifyHashChain / Rust の同等関数と等価の整合性検証関数
// metadata: 検証対象の AttachmentMetadata
// computedChunkHashes: クライアント側で再計算したチャンクハッシュ配列
// 戻り値: ハッシュチェーンが一致する場合 true、不一致の場合 false
func VerifyHashChain(metadata AttachmentMetadata, computedChunkHashes []string) bool {
	// 再計算したハッシュチェーンを生成する
	expected := BuildHashChain(computedChunkHashes)
	// metadata のハッシュチェーンと比較する（整合性確認用途）
	return metadata.HashChain == expected
}

// --------- sandbox URL 生成 ---------

// CreateSandboxURL は添付ファイル ID から sandbox iframe 用の tier2 proxy URL を生成する
// spec arch.tier3 §26_添付帳票 UX: CSP sandbox iframe で表示するため BFF proxy URL を生成する
// TypeScript createSandboxUrl と等価の関数
// attachmentID: sandbox iframe に表示する添付ファイルの UUID
// bffOrigin: tier2 BFF のオリジン（例: "https://bff.example.com"）
func CreateSandboxURL(attachmentID string, bffOrigin string) string {
	// tier2 BFF の attachment proxy エンドポイント URL を組み立てる
	// attachmentID を path パラメータとして埋め込む（クエリパラメータに tenant_id を含めない）
	return fmt.Sprintf("%s/attachments/%s/view", bffOrigin, attachmentID)
}

// --------- AttachmentStore interface ---------

// AttachmentStore は Object Storage への添付ファイル操作を抽象化する interface
// C# IAttachmentStore / Rust AttachmentStore trait / TypeScript IAttachmentStore と等価の interface
type AttachmentStore interface {
	// Upload はファイルバイナリをチャンク分割してアップロードし AttachmentMetadata を返す
	// mimeType: ファイルの MIME タイプ（allowlist 検査前に渡す）
	// data: アップロード対象のバイナリデータ
	// fileName: ファイル名（メタデータに記録する）
	Upload(tenantID string, fileName string, mimeType string, data []byte) (*AttachmentMetadata, error)

	// Download は添付ファイル ID を指定してバイナリデータを返す
	// attachmentID: 取得する添付ファイルの UUID
	Download(attachmentID string) ([]byte, error)

	// Delete は添付ファイルを論理削除する
	// attachmentID: 削除する添付ファイルの UUID
	Delete(attachmentID string) error

	// GenerateSignedURL は Object Storage の署名付き URL を生成する（sandbox iframe 表示用）
	// attachmentID: 署名付き URL を生成する添付ファイルの UUID
	// expiresInSeconds: URL の有効期限（秒）— wall-clock TTL ではなく BFF 側で HLC を使用する
	GenerateSignedURL(attachmentID string, expiresInSeconds int) (string, error)
}

// --------- inMemoryRecord（内部型） ---------

// inMemoryRecord は InMemoryAttachmentStore の内部レコード型
type inMemoryRecord struct {
	// 添付ファイルのメタデータ
	metadata AttachmentMetadata
	// 添付ファイルのバイナリデータ
	data []byte
}

// --------- InMemoryAttachmentStore（骨格実装） ---------

// InMemoryAttachmentStore は AttachmentStore の骨格 in-memory 実装（テスト / モック用途）
// 本番環境では Object Storage (S3 互換) を呼び出す実装に差し替える
// Rust / TypeScript TierAttachmentStore と等価の骨格実装
type InMemoryAttachmentStore struct {
	// スレッドセーフな読み書きロック（同時アクセス保護）
	mu sync.RWMutex
	// in-memory ストレージ（attachmentID → inMemoryRecord のマップ）
	store map[string]inMemoryRecord
}

// NewInMemoryAttachmentStore は InMemoryAttachmentStore のインスタンスを生成する
func NewInMemoryAttachmentStore() *InMemoryAttachmentStore {
	// ストアマップを初期化して返す
	return &InMemoryAttachmentStore{
		// 空のマップで初期化する
		store: make(map[string]inMemoryRecord),
	}
}

// Upload は MIME タイプ検査 → チャンクハッシュ計算 → in-memory 保存の順で処理する
func (s *InMemoryAttachmentStore) Upload(tenantID string, fileName string, mimeType string, data []byte) (*AttachmentMetadata, error) {
	// MIME タイプが許可リストに含まれているか検査する
	if !CheckMimeType(mimeType) {
		// 許可されていない MIME タイプはエラーとして返す
		return nil, fmt.Errorf("許可されていない MIME タイプです: %s", mimeType)
	}
	// チャンクに分割してハッシュを計算する（デフォルトチャンクサイズを使用する）
	chunkHashes := splitAndHash(data, DefaultChunkSizeBytes)
	// ハッシュチェーン文字列を生成する
	hashChain := BuildHashChain(chunkHashes)
	// 添付ファイル UUID を生成する
	id := generateUUID()
	// アップロード完了時刻を ISO 8601 文字列で記録する（表示用途のみ — TTL 計算禁止）
	uploadedAt := time.Now().UTC().Format(time.RFC3339)
	// メタデータを構築する
	metadata := AttachmentMetadata{
		// 生成した UUID を設定する
		ID: id,
		// テナント識別子を設定する（公開 URL に露出しない）
		TenantID: tenantID,
		// MIME タイプを設定する
		MimeType: mimeType,
		// ファイルサイズを設定する
		SizeBytes: int64(len(data)),
		// ハッシュチェーンを設定する
		HashChain: hashChain,
		// アップロード完了時刻を設定する
		UploadedAt: uploadedAt,
	}
	// 書き込みロックを取得してストアに保存する
	s.mu.Lock()
	// defer でアンロックを確実に実行する
	defer s.mu.Unlock()
	// データのコピーを保存する（元スライスへの外部変更を防ぐ）
	dataCopy := make([]byte, len(data))
	// データをコピーする
	copy(dataCopy, data)
	// in-memory ストアに保存する
	s.store[id] = inMemoryRecord{metadata: metadata, data: dataCopy}
	// メタデータのポインタを返す
	return &metadata, nil
}

// Download は添付ファイル ID を指定してバイナリデータを返す
func (s *InMemoryAttachmentStore) Download(attachmentID string) ([]byte, error) {
	// 読み取りロックを取得する
	s.mu.RLock()
	// defer でアンロックを確実に実行する
	defer s.mu.RUnlock()
	// ストアからレコードを取得する
	record, ok := s.store[attachmentID]
	// レコードが存在しない場合はエラーを返す
	if !ok {
		// 存在しない添付ファイル ID はエラーとして返す
		return nil, fmt.Errorf("添付ファイルが見つかりません: %s", attachmentID)
	}
	// データのコピーを返す（内部スライスへの外部変更を防ぐ）
	result := make([]byte, len(record.data))
	// データをコピーして返す
	copy(result, record.data)
	// コピーしたバイト列を返す
	return result, nil
}

// Delete は添付ファイルを in-memory ストアから論理削除する（物理削除）
func (s *InMemoryAttachmentStore) Delete(attachmentID string) error {
	// 書き込みロックを取得する
	s.mu.Lock()
	// defer でアンロックを確実に実行する
	defer s.mu.Unlock()
	// ストアからエントリを削除する（存在しない場合も nil を返す — 冪等）
	delete(s.store, attachmentID)
	// エラーなし
	return nil
}

// GenerateSignedURL は in-memory 実装ではモック URL を返す
// 本番実装では S3 互換 SDK の presign API を呼び出す
// expiresInSeconds は URL パラメータとして含める（テスト検証用）
func (s *InMemoryAttachmentStore) GenerateSignedURL(attachmentID string, expiresInSeconds int) (string, error) {
	// 読み取りロックを取得する
	s.mu.RLock()
	// defer でアンロックを確実に実行する
	defer s.mu.RUnlock()
	// ストアにレコードが存在するか確認する
	if _, ok := s.store[attachmentID]; !ok {
		// 存在しない添付ファイル ID はエラーとして返す
		return "", errors.New("添付ファイルが見つかりません: " + attachmentID)
	}
	// モック signed URL を生成して返す（本番は S3 互換 SDK の presign を使用する）
	url := fmt.Sprintf(
		"https://mock-storage.example.com/attachments/%s?expires=%d",
		attachmentID,
		expiresInSeconds,
	)
	// モック URL を返す
	return url, nil
}

// --------- 内部ヘルパー ---------

// splitAndHash はバイト列をチャンクに分割して各チャンクの SHA-256 ハッシュを返す
// data: 分割対象のバイト列
// chunkSize: チャンクサイズ（バイト）
func splitAndHash(data []byte, chunkSize int) []string {
	// チャンクハッシュを収集するスライス
	var hashes []string
	// オフセット（処理済みバイト数）
	offset := 0
	// データ全体を処理し終わるまでループする
	for offset < len(data) {
		// 次のチャンクの終端バイトを計算する（データ末尾を超えないよう clamp する）
		end := offset + chunkSize
		if end > len(data) {
			// データ末尾で clamp する
			end = len(data)
		}
		// チャンクのスライスを取得する
		chunk := data[offset:end]
		// チャンクの SHA-256 ハッシュを計算する
		hash := ComputeSha256Hex(chunk)
		// ハッシュを追加する
		hashes = append(hashes, hash)
		// オフセットを更新する
		offset = end
	}
	// チャンクハッシュ一覧を返す
	return hashes
}
