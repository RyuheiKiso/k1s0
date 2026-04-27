// Package k1s0 は tier1 公開 12 API への高水準 Go SDK ファサードを提供する。
//
// 利用例（README.md と整合）:
//
//	package main
//
//	import (
//		"context"
//		"log"
//		"github.com/k1s0/sdk-go/k1s0"
//	)
//
//	func main() {
//		ctx := context.Background()
//		c, err := k1s0.New(ctx, k1s0.Config{
//			Target:   "tier1-state.tier1-facade.svc.cluster.local:50001",
//			TenantID: "tenant-A",
//			Subject:  "service-account-foo",
//			UseTLS:   true,
//		})
//		if err != nil {
//			log.Fatal(err)
//		}
//		defer c.Close()
//
//		// State API
//		data, etag, found, err := c.State().Get(ctx, "valkey-default", "user/123")
//		_ = data; _ = etag; _ = found; _ = err
//		newEtag, err := c.State().Save(ctx, "valkey-default", "user/123", []byte("payload"), k1s0.WithTTL(3600))
//		_ = newEtag
//
//		// PubSub API
//		offset, err := c.PubSub().Publish(ctx, "events", []byte(`{"k":"v"}`), "application/json")
//		_ = offset
//
//		// Secrets API
//		values, version, err := c.Secrets().Get(ctx, "db-credentials")
//		_ = values; _ = version
//
//		// 動詞統一 facade 未実装の service は Raw() 経由（残り 9 service）
//		raw := c.Raw()
//		_ = raw.Workflow // workflowv1.WorkflowServiceClient
//	}
//
// 設計正典:
//   - docs/05_実装/10_ビルド設計/20_Go_module分離戦略/01_Go_module分離戦略.md
//   - docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/
//
// 残り 9 service（Invoke / Workflow / Log / Telemetry / Decision / Audit / Pii /
// Feature / Binding）の動詞統一 facade はロードマップ #8 後続セッションで追加。
package k1s0
