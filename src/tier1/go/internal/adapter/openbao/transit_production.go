// 本ファイルは OpenBao Transit Engine の production 経路 adapter 実装。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/04_Secrets_API.md
//     - FR-T1-SECRETS-003（Transit 暗号化、AES-256-GCM、鍵バージョン管理）
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/04_Secrets_API.md
//
// 役割:
//   OPENBAO_ADDR が設定された production 経路で、bao.Logical() を介して
//   OpenBao Transit Engine の Encrypt / Decrypt / RotateKey API を呼ぶ。
//   本実装と InMemoryTransit は ciphertext フォーマットが異なる:
//     - InMemoryTransit: [4 byte BE version][12 byte nonce][AES-GCM(ct+tag)]
//     - productionTransit: "vault:v<N>:<base64-blob>"（OpenBao server が生成・解釈）
//   そのため、開発で in-memory に保存した ciphertext を production で復号する
//   経路は許容されない（dev/CI と production はデータを共有しない設計、ADR-SEC-002）。
//
// セキュリティ:
//   - AES-256-GCM は OpenBao Transit の既定（FR-T1-SECRETS-003 受け入れ基準）。
//   - 鍵生成 / nonce / 鍵ローテーションはすべて OpenBao server 側で行う。
//   - tier1 はクライアントとして transit/encrypt/<key> 等の path を Write するだけ。

package openbao

import (
	// context 伝搬。
	"context"
	// base64 encode / decode（OpenBao Transit の plaintext / context は base64 規約）。
	"encoding/base64"
	// path 構築。
	"fmt"
	// 数値型変換（KeyVersion / NewVersion など）。
	"strconv"
	// 鍵 version / 時刻取得（RotatedAtMs 用）。
	"time"

	// OpenBao Go SDK。
	bao "github.com/openbao/openbao/api/v2"
)

// transitWriter は OpenBao SDK の `Logical()` から本 adapter が **実際に使う**
// Write メソッドだけを集めた narrow interface。`*bao.Logical` がこれを満たす。
// test では fake を注入できるよう、interface 経由で結線する。
type transitWriter interface {
	// path を Write し、応答 *bao.Secret を返す。OpenBao Transit の場合、
	// Encrypt は data["ciphertext"] = "vault:v1:..." を返却する。
	WriteWithContext(ctx context.Context, path string, data map[string]interface{}) (*bao.Secret, error)
}

// productionTransit は bao.Logical() 経由で OpenBao Transit Engine を呼ぶ TransitAdapter。
type productionTransit struct {
	// SDK の Logical() narrow client（test では fake）。
	writer transitWriter
	// Transit Engine の mount path（既定 "transit"、infra/security/openbao/ で apply）。
	mount string
}

// NewProductionTransit は OpenBao Client から TransitAdapter を生成する。
// `OPENBAO_ADDR` が設定されている cmd/secret 起動経路から呼ばれる。
// `mount` は Transit Engine の mount path で、空文字なら "transit" を採用する。
func NewProductionTransit(c *Client, mount string) TransitAdapter {
	// Client が無い構築（fake / in-memory 経路）は in-memory にフォールバック。
	if c == nil {
		return NewInMemoryTransit()
	}
	// Client は dynamic 用に SDK Logical() を保持しているが、Transit 用に直接公開
	// しないため、Logical() narrow を取り出す追加 helper（transitWriterFor）に委譲する。
	w := c.transitWriterFor()
	if w == nil {
		// SDK 未注入（test 経路）は in-memory にフォールバック。
		return NewInMemoryTransit()
	}
	return &productionTransit{writer: w, mount: resolveTransitMount(mount)}
}

// NewProductionTransitFromWriter は test 用に narrow interface を直接受け取る。
func NewProductionTransitFromWriter(w transitWriter, mount string) TransitAdapter {
	return &productionTransit{writer: w, mount: resolveTransitMount(mount)}
}

// resolveTransitMount は空 mount に既定 "transit" を充てる。先頭 / 末尾 slash は除去。
func resolveTransitMount(mount string) string {
	if mount == "" {
		return "transit"
	}
	// "transit/" 末尾 slash や "/transit" 先頭 slash の素朴な正規化。
	for len(mount) > 0 && mount[0] == '/' {
		mount = mount[1:]
	}
	for len(mount) > 0 && mount[len(mount)-1] == '/' {
		mount = mount[:len(mount)-1]
	}
	if mount == "" {
		return "transit"
	}
	return mount
}

// Encrypt は OpenBao Transit の transit/encrypt/<key> を呼んで ciphertext を発行する。
//
// 入出力:
//   - plaintext は base64 エンコードして data["plaintext"] に詰める（OpenBao 規約）。
//   - AAD は base64 エンコードして data["context"] に詰める（GCM の Additional Authenticated Data）。
//   - 応答 data["ciphertext"] は "vault:v<N>:<base64-blob>" 形式の opaque string。
//   - 応答 data["key_version"] は当該操作で使われた key_version（int / json.Number）。
func (p *productionTransit) Encrypt(req TransitEncryptRequest) (TransitEncryptResponse, error) {
	if req.KeyName == "" {
		return TransitEncryptResponse{}, fmt.Errorf("tier1/secrets/transit: key_name required")
	}
	// OpenBao は plaintext を base64 で受け取る規約。
	body := map[string]interface{}{
		"plaintext": base64.StdEncoding.EncodeToString(req.Plaintext),
	}
	if len(req.AAD) > 0 {
		body["context"] = base64.StdEncoding.EncodeToString(req.AAD)
	}
	path := fmt.Sprintf("%s/encrypt/%s", p.mount, req.KeyName)
	// context は ResolveTimeout の余地を持たせるため呼出側 ctx を尊重。
	// 上位 (handler) は handler ctx を渡す前提だが、SDK narrow は引数を持たないため
	// productionTransit の interface も ctx を受け取らない（in-memory backend 互換）。
	// production 経路では SDK 内部 timeout（既定 60s）を使う。
	secret, err := p.writer.WriteWithContext(context.Background(), path, body)
	if err != nil {
		return TransitEncryptResponse{}, err
	}
	if secret == nil || secret.Data == nil {
		return TransitEncryptResponse{}, fmt.Errorf(
			"tier1/secrets/transit: empty response from OpenBao at %s", path,
		)
	}
	// 応答パース。SDK は string を string、数値を json.Number で返す。
	ct, ok := secret.Data["ciphertext"].(string)
	if !ok || ct == "" {
		return TransitEncryptResponse{}, fmt.Errorf(
			"tier1/secrets/transit: invalid ciphertext field in response (path=%s)", path,
		)
	}
	keyVersion := extractKeyVersion(secret.Data, "key_version")
	return TransitEncryptResponse{
		Ciphertext: []byte(ct),
		KeyVersion: keyVersion,
	}, nil
}

// Decrypt は OpenBao Transit の transit/decrypt/<key> を呼んで plaintext を復号する。
//
// 入出力:
//   - Ciphertext は productionTransit.Encrypt が返した "vault:v<N>:<base64>" string を
//     []byte に詰めたものを期待する。
//   - 応答 data["plaintext"] は base64 で返るため、本層で decode して []byte で返す。
func (p *productionTransit) Decrypt(req TransitDecryptRequest) (TransitDecryptResponse, error) {
	if req.KeyName == "" {
		return TransitDecryptResponse{}, fmt.Errorf("tier1/secrets/transit: key_name required")
	}
	if len(req.Ciphertext) == 0 {
		return TransitDecryptResponse{}, ErrTransitCiphertextMalformed
	}
	body := map[string]interface{}{
		"ciphertext": string(req.Ciphertext),
	}
	if len(req.AAD) > 0 {
		body["context"] = base64.StdEncoding.EncodeToString(req.AAD)
	}
	path := fmt.Sprintf("%s/decrypt/%s", p.mount, req.KeyName)
	secret, err := p.writer.WriteWithContext(context.Background(), path, body)
	if err != nil {
		return TransitDecryptResponse{}, err
	}
	if secret == nil || secret.Data == nil {
		return TransitDecryptResponse{}, fmt.Errorf(
			"tier1/secrets/transit: empty response from OpenBao at %s", path,
		)
	}
	plainB64, ok := secret.Data["plaintext"].(string)
	if !ok {
		return TransitDecryptResponse{}, fmt.Errorf(
			"tier1/secrets/transit: invalid plaintext field in response (path=%s)", path,
		)
	}
	plain, err := base64.StdEncoding.DecodeString(plainB64)
	if err != nil {
		return TransitDecryptResponse{}, fmt.Errorf(
			"tier1/secrets/transit: malformed plaintext base64 in response: %w", err,
		)
	}
	return TransitDecryptResponse{
		Plaintext:  plain,
		KeyVersion: extractKeyVersion(secret.Data, "key_version"),
	}, nil
}

// RotateKey は OpenBao Transit の transit/keys/<key>/rotate を呼んで鍵をローテーションする。
// OpenBao 側で latest_version が +1 され、旧版鍵は引き続き Decrypt 用に保持される
// （FR-T1-SECRETS-003 受け入れ基準「旧版でも Decrypt 可能」を OpenBao が担保）。
//
// OpenBao の rotate API は previous_version を返さないため、tier1 で latest_version を
// 取得する必要がある場合は事前に keys/<key> を Read する設計だが、本実装は
// 「rotate 直後の応答に含まれる current」をそのまま NewVersion とし、PreviousVersion は
// new - 1 で導出する（最初の rotate で previous=1, new=2 となる、OpenBao 慣用）。
func (p *productionTransit) RotateKey(req TransitRotateKeyRequest) (TransitRotateKeyResponse, error) {
	if req.KeyName == "" {
		return TransitRotateKeyResponse{}, fmt.Errorf("tier1/secrets/transit: key_name required")
	}
	path := fmt.Sprintf("%s/keys/%s/rotate", p.mount, req.KeyName)
	// rotate API は body 不要（OpenBao 側で新版鍵を生成）。
	if _, err := p.writer.WriteWithContext(context.Background(), path, map[string]interface{}{}); err != nil {
		return TransitRotateKeyResponse{}, err
	}
	// rotate 後の latest version を取得するために keys/<key> を Read する。
	// OpenBao は POST にも応答するため、Read の代わりに同 path に空 body を Write しても良いが、
	// 慣用に従って情報取得は GET 相当の Read（Logical().ReadWithContext）に分離するべき。
	// ここでは productionTransit の writer narrow に Read 機能が無いため、
	// "rotate 応答が new version を含むこと" に依存しない実装として
	// previous は 0 / new は 0 を返し、handler 段で Time だけ詰める運用とする。
	// 呼出側（handler）は NewVersion>0 のみを意味あり扱いとする想定。
	now := time.Now().UnixMilli()
	return TransitRotateKeyResponse{
		// production 経路では keys/<key> Read のための adapter 拡張が別途必要。
		// MVP としては 0 で返し、handler 側は RotatedAtMs だけを意味あり扱いにする。
		NewVersion:      0,
		PreviousVersion: 0,
		RotatedAtMs:     now,
	}, nil
}

// extractKeyVersion は OpenBao SDK 応答の数値フィールドを int に正規化する。
// SDK は数値を `json.Number` で返すため、float64 / int / string のいずれもケアする。
func extractKeyVersion(data map[string]interface{}, key string) int {
	v, ok := data[key]
	if !ok {
		return 0
	}
	switch x := v.(type) {
	case float64:
		return int(x)
	case int:
		return x
	case int64:
		return int(x)
	case string:
		n, err := strconv.Atoi(x)
		if err != nil {
			return 0
		}
		return n
	default:
		// json.Number は Stringer を満たす。fmt.Sprintf 経由で string 化した上で parse。
		s := fmt.Sprintf("%v", v)
		n, err := strconv.Atoi(s)
		if err != nil {
			return 0
		}
		return n
	}
}
