// 本ファイルは OpenBao 接続を行わない in-memory KVv2 backend。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/04_Secrets_API.md
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/01_tier1全体コンポーネント俯瞰.md
//     - DS-SW-COMP-006（t1-secret Pod、OpenBao 直結）
//
// 役割:
//   OpenBao server を持たない開発 / CI 環境でも cmd/secret バイナリが起動から
//   gRPC 応答まで実値を返せるよう、`kvClient` interface を満たす in-memory 実装を
//   提供する。production では `OPENBAO_ADDR` 環境変数で実 OpenBao に切替わる。
//
// バージョニング:
//   KVv2 互換のセマンティクスを保つため、各 path につき version 1.. の履歴を
//   保持する。Get（最新）は最新 version を返し、GetVersion(v) で個別 version を返す。
//   Put は version を +1 してエントリを追加する。Rotate は同値 Put でバージョン bump する。
//
// 並行制御:
//   sync.Mutex で全操作を直列化する。production の OpenBao は MVCC で並行可能だが、
//   in-memory backend は単一 Pod 配下の dev / CI 用途に限定するため単純化する。

// Package openbao は tier1 Go ファサードが OpenBao を直接叩くためのアダプタ層。
package openbao

import (
	// 全 RPC で context を伝搬する。
	"context"
	// 並行制御の Mutex を提供する。
	"sync"

	// OpenBao SDK の KVSecret / KVVersionMetadata 型を構築するため import する。
	bao "github.com/openbao/openbao/api/v2"
)

// inMemoryEntry は 1 つの KVv2 path につき 1 version 分の保存データ。
type inMemoryEntry struct {
	// data は version 内の key-value マップ。
	data map[string]interface{}
	// version はバージョン番号（1..）。
	version int
}

// InMemoryKV は OpenBao 接続なしで動作する kvClient 実装。
//
// 用途:
//   - cmd/secret バイナリの dev / CI モード
//   - openbao_test 系の round-trip 試験
//
// 制約:
//   - process 内でのみ永続化（再起動で全データ消失）
//   - tenant 境界は path prefix で表現（`tenant/<tenant_id>/<name>` 規約）
type InMemoryKV struct {
	// mu は全操作を直列化する Mutex。
	mu sync.Mutex
	// entries は path → 全 version の slice（昇順）。
	entries map[string][]inMemoryEntry
}

// NewInMemoryKV は空の InMemoryKV を生成する。
func NewInMemoryKV() *InMemoryKV {
	// 空 map で初期化する。
	return &InMemoryKV{entries: map[string][]inMemoryEntry{}}
}

// Get は最新 version を返す。未存在時は (nil, nil) を返す（adapter で NotFound 翻訳）。
func (s *InMemoryKV) Get(_ context.Context, secretPath string) (*bao.KVSecret, error) {
	// Mutex で entries 読出を保護する。
	s.mu.Lock()
	defer s.mu.Unlock()
	// path 配下の version 列を取得する。
	versions, ok := s.entries[secretPath]
	// 未存在 / 空 slice は nil で返す（OpenBao SDK 慣行）。
	if !ok || len(versions) == 0 {
		// nil を返却する。
		return nil, nil
	}
	// 最新 version は slice 末尾に格納されている。
	last := versions[len(versions)-1]
	// data は呼出側破壊防止のためコピーを返す。
	return s.toKVSecret(last), nil
}

// GetVersion は指定 version を返す。version > 履歴長 / version <= 0 は nil を返す。
func (s *InMemoryKV) GetVersion(_ context.Context, secretPath string, version int) (*bao.KVSecret, error) {
	// Mutex で entries 読出を保護する。
	s.mu.Lock()
	defer s.mu.Unlock()
	// path 配下の version 列を取得する。
	versions, ok := s.entries[secretPath]
	// 未存在 / 範囲外は nil を返却する。
	if !ok || version <= 0 || version > len(versions) {
		// nil を返却する。
		return nil, nil
	}
	// version は 1-based なので index は version-1。
	target := versions[version-1]
	// KVSecret に変換して返す。
	return s.toKVSecret(target), nil
}

// Put は version +1 のエントリを追加し、新規 KVSecret を返す。
// opts は本 in-memory backend では未使用（CAS / Check-And-Set は将来拡張）。
func (s *InMemoryKV) Put(_ context.Context, secretPath string, data map[string]interface{}, _ ...bao.KVOption) (*bao.KVSecret, error) {
	// Mutex で entries 読書を保護する。
	s.mu.Lock()
	defer s.mu.Unlock()
	// 現在 version 数 + 1 が新 version 番号。
	newVersion := len(s.entries[secretPath]) + 1
	// data は呼出側破壊防止のため shallow copy する。
	cp := make(map[string]interface{}, len(data))
	// for-range で key/value を新 map に詰める。
	for k, v := range data {
		// 1 件ずつ複製する。
		cp[k] = v
	}
	// 新 version を生成する。
	entry := inMemoryEntry{data: cp, version: newVersion}
	// path 配下の version 列に append する。
	s.entries[secretPath] = append(s.entries[secretPath], entry)
	// 呼出側に返却する KVSecret を生成する。
	return s.toKVSecret(entry), nil
}

// List は path 配下の secret 名一覧を返す。OpenBao SDK の KVv2 には List がないため、
// production では Logical().List(metadataPath) で代替するが、in-memory では entries map
// を走査して prefix 一致するものを返す。BulkGet（FR-T1-SECRETS-002）の補助用。
func (s *InMemoryKV) List(_ context.Context, prefix string) ([]string, error) {
	// Mutex で entries 読出を保護する。
	s.mu.Lock()
	defer s.mu.Unlock()
	// prefix から相対 path を抽出する一時 slice。
	out := make([]string, 0, len(s.entries))
	// 全エントリを走査する。
	for path := range s.entries {
		// prefix 一致しないものは除外する。
		if !hasPrefix(path, prefix) {
			// 除外して次の iteration に進む。
			continue
		}
		// prefix 配下のフル path を返却する（呼出側で trimPrefix する運用）。
		out = append(out, path)
	}
	// path 順序は map 走査依存だが、呼出側がソート前提とするなら別途 sort.Strings する。
	return out, nil
}

// hasPrefix は文字列 prefix 判定の薄いラッパ（strings.HasPrefix を import せず inline）。
func hasPrefix(s, prefix string) bool {
	// 長さ不足は即 false。
	if len(s) < len(prefix) {
		// false を返す。
		return false
	}
	// 先頭 N 文字が一致するか比較する。
	return s[:len(prefix)] == prefix
}

// toKVSecret は inMemoryEntry を OpenBao SDK の KVSecret に変換する。
// 呼出側破壊防止のため data は shallow copy した新 map を渡す。
func (s *InMemoryKV) toKVSecret(e inMemoryEntry) *bao.KVSecret {
	// data の shallow copy を作る。
	cp := make(map[string]interface{}, len(e.data))
	// for-range で詰め直す。
	for k, v := range e.data {
		// 1 件ずつ複製する。
		cp[k] = v
	}
	// KVSecret に詰めて返す。
	return &bao.KVSecret{
		// data 本体。
		Data: cp,
		// バージョンメタ。
		VersionMetadata: &bao.KVVersionMetadata{Version: e.version},
	}
}

// NewClientWithInMemoryKV は in-memory backend を持つ Client を生成する。
// cmd/secret/main.go から OPENBAO_ADDR 未設定時の fallback として呼ばれる。
func NewClientWithInMemoryKV() *Client {
	// 空 in-memory KV を生成する。
	mem := NewInMemoryKV()
	// 既存 InMemoryKV を流用する constructor に委譲する。
	return NewClientFromInMemoryKV(mem)
}

// NewClientFromInMemoryKV は呼出側が用意した InMemoryKV を Client に埋め込んで返す。
// 試験で seed 用途に kv への直接アクセスを残しつつ、production と同じ Client / adapter
// 経路を辿らせるために使う（kv と lister の両方が同一 InMemoryKV を指す）。
func NewClientFromInMemoryKV(mem *InMemoryKV) *Client {
	// Client に in-memory backend を埋め込む（address は識別ラベル）。
	return &Client{
		// アドレスは "in-memory" 固定（観測用ラベル）。
		address: "in-memory",
		// kvClient interface に in-memory 実装を割当てる。
		kv: mem,
		// Lister interface に同一の in-memory 実装を割当てる。
		lister: mem,
	}
}

// Lister は path 配下の secret 名一覧を取得する narrow interface。
// production: bao.Client.Logical().List() を経由した薄い shim、in-memory: 直接実装。
type Lister interface {
	// List は prefix 配下のフル path 一覧を返す。
	List(ctx context.Context, prefix string) ([]string, error)
}
