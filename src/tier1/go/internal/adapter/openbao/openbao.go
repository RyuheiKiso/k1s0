// 本ファイルは tier1 Go の OpenBao アダプタ層の起点。
//
// 設計正典:
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/01_tier1全体コンポーネント俯瞰.md
//     - DS-SW-COMP-006（t1-secret: Active 1 / standby 2、HPA 禁止、OpenBao 直結）
//   docs/02_構想設計/adr/ADR-SEC-002-openbao.md
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/04_Secrets_API.md
//
// 役割（plan 04-06 結線済）:
//   t1-secret Pod が OpenBao（HashiCorp Vault fork）と直接連携するためのアダプタ。
//   Dapr Secrets building block を経由せず、OpenBao Go SDK の KVv2 API を narrow
//   interface 越しに呼び出す。
//
// テスタビリティ設計:
//   `kvClient` narrow interface で SDK の KVv2 を抽象化し、test では fake を注入できる。
//   production の Client は OpenBao SDK の `*bao.Client` を保持し、KVv2(mount) で
//   namespace 別の実 client を取得する。

// Package openbao は tier1 Go ファサードが OpenBao を直接叩くためのアダプタ層。
package openbao

import (
	"context"
	"errors"

	// OpenBao Go SDK。
	bao "github.com/openbao/openbao/api/v2"
)

// ErrNotWired は OpenBao backend と未結線である旨を示すセンチネルエラー。
// 主に SecretsAdapter の placeholder 実装が返す（test 環境で client 不在時など）。
var ErrNotWired = errors.New("tier1: OpenBao not wired")

// ErrSecretNotFound はシークレット未存在を示すセンチネルエラー。
// OpenBao SDK は 404 を error で返すため、handler 側で gRPC NotFound に翻訳する。
var ErrSecretNotFound = errors.New("tier1: secret not found")

// kvClient は本パッケージが OpenBao SDK の KVv2 から **実際に使うメソッド** だけを
// 集めた narrow interface。`*bao.KVv2` がこれを満たすため production では SDK を
// そのまま注入し、test では fake を注入する。
type kvClient interface {
	// 単一 secret 取得（最新バージョン）。
	Get(ctx context.Context, secretPath string) (*bao.KVSecret, error)
	// 単一 secret 取得（指定バージョン）。
	GetVersion(ctx context.Context, secretPath string, version int) (*bao.KVSecret, error)
	// 単一 secret の新規バージョン書込。
	Put(ctx context.Context, secretPath string, data map[string]interface{}, opts ...bao.KVOption) (*bao.KVSecret, error)
}

// Client は tier1 Go ファサードから見た OpenBao のアダプタ。
// 本構造体は KVv2 narrow interface を保持し、複数の adapter から共有して使う。
type Client struct {
	// OpenBao server アドレス（観測性 / デバッグ用途で SidecarAddress 同様に exposing）。
	address string
	// KVv2 narrow client（production: SDK の KVv2、test: fake）。
	kv kvClient
	// path 配下の secret 名列挙用 narrow client（BulkGet 用、production: Logical().List() 経由 shim）。
	lister Lister
	// dynamic credential 取得用 narrow client（production: SDK の Logical()、test: fake）。
	// 動的 Secret 発行（FR-T1-SECRETS-002）の OpenBao Database Engine 経路で使う。
	dynamicReader dynamicReader
	// SDK Client インスタンス（Close 用、fake 注入時は nil）。
	closer interface{ Close() }
}

// Config は Client 初期化時に渡される設定。
type Config struct {
	// OpenBao server URL（例: "https://openbao.k1s0-security.svc.cluster.local:8200"）。
	Address string
	// 認証トークン（JWT / approle / kubernetes auth で別途取得済の値）。
	Token string
	// KV mount path（例: "secret"）。
	KVMount string
}

// New は Config から Client を生成し、OpenBao SDK の HTTP client を初期化する。
// 接続検証は SDK 内部で遅延されるため、Get / Put 呼び出し時に network エラーが発生する。
func New(_ context.Context, cfg Config) (*Client, error) {
	// SDK の Default config をベースに、address だけ上書きする。
	baoConfig := bao.DefaultConfig()
	if cfg.Address != "" {
		baoConfig.Address = cfg.Address
	}
	// SDK Client 生成。HTTPS / mTLS / proxy 等は DefaultConfig の挙動に委ねる。
	sdkClient, err := bao.NewClient(baoConfig)
	if err != nil {
		return nil, err
	}
	if cfg.Token != "" {
		sdkClient.SetToken(cfg.Token)
	}
	// KVv2 mount 配下の client を narrow interface 越しに保持する。
	mount := cfg.KVMount
	if mount == "" {
		// k1s0 既定 mount 名（secret/）。infra/security/openbao/ で apply する想定。
		mount = "secret"
	}
	return &Client{
		address: baoConfig.Address,
		kv:      sdkClient.KVv2(mount),
		// production の Lister は Logical().List() を mount 配下の metadata path で呼ぶ shim。
		lister: newProductionLister(sdkClient, mount),
		// 動的 Secret（Database Engine）の Read 経路。SDK の Logical() を narrow に切り出す。
		dynamicReader: sdkClient.Logical(),
		// SDK Client は Close() を持たない（HTTP client は GC 任せ）ため closer は nil。
	}, nil
}

// NewWithKVClient は test 用コンストラクタ。任意の kvClient 実装を受け取る。
func NewWithKVClient(addr string, kv kvClient) *Client {
	return &Client{address: addr, kv: kv, closer: nil}
}

// Address は OpenBao server address を返す。
func (c *Client) Address() string {
	return c.address
}

// Close は Client が保持する OpenBao SDK Client の解放を行う。
// 本 SDK は HTTP client なので Close 不要だが、interface 統一のため定義する。
func (c *Client) Close() error {
	if c.closer == nil {
		return nil
	}
	c.closer.Close()
	return nil
}

// kvClientFor は内部 narrow client を返す。adapter 実装からのみ使う。
func (c *Client) kvClientFor() kvClient {
	return c.kv
}

// listerFor は内部 Lister narrow client を返す。BulkGet 等 List が必要な adapter から呼ぶ。
// 注入されていない場合は nil を返す（adapter 側で空一覧扱いにする）。
func (c *Client) listerFor() Lister {
	return c.lister
}

// dynamicReaderFor は内部 dynamic credential 用 narrow client を返す。
// 動的 Secret 発行 adapter（productionDynamic）から呼び出される。
// fake / in-memory 注入時は nil（caller 側で ErrNotWired にフォールバック）。
func (c *Client) dynamicReaderFor() dynamicReader {
	return c.dynamicReader
}
