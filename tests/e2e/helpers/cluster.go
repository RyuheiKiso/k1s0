// 本ファイルは E2E シナリオから利用する kind cluster 操作ヘルパ。
// 設計正典: docs/05_実装/00_ディレクトリ設計/70_共通資産/02_tests配置.md
// 関連 ADR: ADR-TEST-001 (Test Pyramid + testcontainers) / ADR-TEST-002 (E2E 自動化)
//
// 前提: tools/local-stack/up.sh --role tier1-go-dev (or 同等以上の role) で kind cluster
// + 本番再現スタックが起動済。E2E test は kubeconfig 経由で既存 cluster に接続する。
package helpers

import (
	"context"
	"fmt"
	"os"
	"path/filepath"
	"testing"
	"time"

	corev1 "k8s.io/api/core/v1"
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/client-go/kubernetes"
	"k8s.io/client-go/tools/clientcmd"
)

// Cluster は E2E シナリオが操作する kind cluster の handle。
type Cluster struct {
	Kubeconfig string
	Client     *kubernetes.Clientset
	// テスト中に作成した namespace（Teardown で削除する）
	createdNamespaces []string
}

// SetupCluster は既存の kind cluster（tools/local-stack/up.sh で起動済）に接続する。
// kubeconfig は KUBECONFIG env var、未指定なら $HOME/.kube/config を使う。
// kind cluster が未起動の場合、test を Skip して開発者に local-stack 起動を促す。
func SetupCluster(t *testing.T) *Cluster {
	t.Helper()

	kubeconfig := os.Getenv("KUBECONFIG")
	if kubeconfig == "" {
		home, err := os.UserHomeDir()
		if err != nil {
			t.Fatalf("UserHomeDir: %v", err)
		}
		kubeconfig = filepath.Join(home, ".kube", "config")
	}

	if _, err := os.Stat(kubeconfig); err != nil {
		t.Skipf("kubeconfig が見つからない (%s): tools/local-stack/up.sh --role tier1-go-dev を先に実行してください", kubeconfig)
	}

	config, err := clientcmd.BuildConfigFromFlags("", kubeconfig)
	if err != nil {
		t.Fatalf("BuildConfigFromFlags: %v", err)
	}

	client, err := kubernetes.NewForConfig(config)
	if err != nil {
		t.Fatalf("kubernetes.NewForConfig: %v", err)
	}

	// 接続確認 — kind cluster が起動していなければ Skip
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()
	if _, err := client.CoreV1().Namespaces().List(ctx, metav1.ListOptions{Limit: 1}); err != nil {
		t.Skipf("kind cluster に接続できない (%v): tools/local-stack/up.sh --role tier1-go-dev を先に実行してください", err)
	}

	return &Cluster{
		Kubeconfig: kubeconfig,
		Client:     client,
	}
}

// CreateNamespace は test 用 namespace を作成する。Teardown で自動削除される。
func (c *Cluster) CreateNamespace(t *testing.T, name string) {
	t.Helper()
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()
	ns := &corev1.Namespace{ObjectMeta: metav1.ObjectMeta{Name: name}}
	if _, err := c.Client.CoreV1().Namespaces().Create(ctx, ns, metav1.CreateOptions{}); err != nil {
		t.Fatalf("create namespace %q: %v", name, err)
	}
	c.createdNamespaces = append(c.createdNamespaces, name)
}

// WaitForRunningPodInNamespace は指定 namespace で Phase=Running の Pod が
// 少なくとも 1 つ出現するまで待つ。timeout で諦めて error を返す。
// 「namespace 自体が無い」ケースは namespace not found エラーを返す。
func (c *Cluster) WaitForRunningPodInNamespace(ctx context.Context, namespace string, timeout time.Duration) (int, error) {
	deadline := time.Now().Add(timeout)
	for {
		pods, err := c.Client.CoreV1().Pods(namespace).List(ctx, metav1.ListOptions{})
		if err != nil {
			return 0, fmt.Errorf("list pods in %q: %w", namespace, err)
		}
		running := 0
		for _, p := range pods.Items {
			if p.Status.Phase == corev1.PodRunning {
				running++
			}
		}
		if running > 0 {
			return running, nil
		}
		if time.Now().After(deadline) {
			return 0, fmt.Errorf("timeout waiting for Running Pod in namespace %q (found %d Pods, 0 Running)", namespace, len(pods.Items))
		}
		select {
		case <-ctx.Done():
			return 0, ctx.Err()
		case <-time.After(2 * time.Second):
		}
	}
}

// Teardown は SetupCluster で作成した namespace を削除する。
// kind cluster 自体は破棄しない（local-stack が管理）。
func (c *Cluster) Teardown(t *testing.T) {
	t.Helper()
	if c == nil || c.Client == nil {
		return
	}
	ctx, cancel := context.WithTimeout(context.Background(), 60*time.Second)
	defer cancel()
	for _, ns := range c.createdNamespaces {
		if err := c.Client.CoreV1().Namespaces().Delete(ctx, ns, metav1.DeleteOptions{}); err != nil {
			t.Logf("teardown: delete namespace %q failed: %v", ns, err)
		}
	}
}
