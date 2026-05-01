// 本ファイルは k1s0 Go SDK の Pii 動詞統一 facade。
package k1s0

import (
	"context"

	piiv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/pii/v1"
)

// PiiClient は PiiService の動詞統一 facade。
type PiiClient struct{ client *Client }

// Pii は親 Client から PiiClient を返す。
func (c *Client) Pii() *PiiClient { return c.pii }

// Classify は PII 種別の検出。findings と contains_pii を返す。
func (p *PiiClient) Classify(ctx context.Context, text string) (findings []*piiv1.PiiFinding, containsPii bool, err error) {
	resp, e := p.client.raw.Pii.Classify(ctx, &piiv1.ClassifyRequest{
		Text:    text,
		Context: p.client.tenantContext(ctx),
	})
	if e != nil {
		return nil, false, e
	}
	return resp.GetFindings(), resp.GetContainsPii(), nil
}

// Mask はマスキング。氏名 → [NAME] 等への置換後テキストと findings を返す。
func (p *PiiClient) Mask(ctx context.Context, text string) (maskedText string, findings []*piiv1.PiiFinding, err error) {
	resp, e := p.client.raw.Pii.Mask(ctx, &piiv1.MaskRequest{
		Text:    text,
		Context: p.client.tenantContext(ctx),
	})
	if e != nil {
		return "", nil, e
	}
	return resp.GetMaskedText(), resp.GetFindings(), nil
}

// Pseudonymize は FR-T1-PII-002（決定論的仮名化）の facade。
// 同一 salt + 同一 fieldType + 同一 value で同一の URL-safe base64 仮名値を返す。
// salt が空 / value が空 / fieldType が空 のいずれかは server 側で InvalidArgument。
func (p *PiiClient) Pseudonymize(ctx context.Context, fieldType, value, salt string) (pseudonym string, err error) {
	resp, e := p.client.raw.Pii.Pseudonymize(ctx, &piiv1.PseudonymizeRequest{
		FieldType: fieldType,
		Value:     value,
		Salt:      salt,
		Context:   p.client.tenantContext(ctx),
	})
	if e != nil {
		return "", e
	}
	return resp.GetPseudonym(), nil
}
