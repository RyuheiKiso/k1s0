// tests/e2e/owner/helpers/cve_client.go
//
// security/ で使う CVE block check helper。Trivy / Grype が image scan で
// Critical を検出した時に Harbor が push を deny したかを確認する。
//
// 設計正典:
//   docs/02_構想設計/04_CICDと配信/00_CICDパイプライン.md（Harbor 門番、Trivy CVE Critical 拒否）
//   docs/05_実装/30_CI_CD設計/35_e2e_test_design/10_owner_suite/02_ディレクトリ構造.md
//
// リリース時点は skeleton.
package helpers

import (
	"context"
	"fmt"
)

// CVEClient は Harbor / Trivy の CVE 検出結果を確認する薄い helper
type CVEClient struct {
	// HarborBaseURL は Harbor API の base URL
	HarborBaseURL string
	// AuthToken は Harbor robot account token
	AuthToken string
}

// NewCVEClient は Harbor base URL + token から client を生成
func NewCVEClient(baseURL, token string) *CVEClient {
	return &CVEClient{HarborBaseURL: baseURL, AuthToken: token}
}

// HasBlockingCVE は image が Critical CVE で blocked されているかを確認する。
// 採用初期で Harbor /api/v2.0/projects/<project>/repositories/<repo>/artifacts/<digest>/additions/vulnerabilities を実装。
func (c *CVEClient) HasBlockingCVE(_ context.Context, image string) (bool, error) {
	return false, fmt.Errorf("HasBlockingCVE 未実装 (image=%s、Harbor API 統合は採用初期)", image)
}
