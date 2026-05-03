// tests/e2e/owner/helpers/k8s_client.go
//
// owner suite 共通 k8s client helper。kubectl 操作を Go から呼ぶ薄いラッパ。
// 8 部位（platform / observability / security / ha-dr / upgrade /
// sdk-roundtrip / tier3-web / perf）すべてが本 helper を使う想定。
//
// 設計正典:
//   ADR-TEST-008（owner suite ディレクトリ）
//   docs/05_実装/30_CI_CD設計/35_e2e_test_design/10_owner_suite/02_ディレクトリ構造.md
//
// IMP-CI-E2E-004 の helpers/ 配下、part 横断 helper として提供する。
package helpers

import (
	// context は kubectl wait / Pod readiness 検証で使う
	"context"
	// errors は wrap で error chain を残す
	"errors"
	// fmt は assertion メッセージのフォーマットで使う
	"fmt"
	// os は KUBECONFIG env の読み取りで使う
	"os"
	// time は wait timeout の指定で使う
	"time"

	// metav1 は ListOptions / GetOptions 等の k8s API メタ型
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	// kubernetes は ClientSet を提供する公式 client
	"k8s.io/client-go/kubernetes"
	// clientcmd は kubeconfig file からの ClientConfig 構築
	"k8s.io/client-go/tools/clientcmd"
)

// K8sClient は owner suite test 全体で使う kubernetes ClientSet wrapper。
// kubeconfig path は KUBECONFIG env から取得し、t.Helper() を呼んで
// 失敗時の log を test 自体のフレーム位置に表示する。
type K8sClient struct {
	// ClientSet は kubernetes 公式 client（直接 expose することで test code が
	// pods / services / nodes / namespaces を自由に呼べる）
	ClientSet *kubernetes.Clientset
	// KubeconfigPath は debug 用に保持（owner suite では tests/.owner-e2e/<日付>/kubeconfig）
	KubeconfigPath string
}

// NewK8sClient は KUBECONFIG env から ClientSet を構築する。
// owner suite では tools/e2e/owner/up.sh が kubeconfig を生成 + KUBECONFIG export する前提。
func NewK8sClient() (*K8sClient, error) {
	// KUBECONFIG env を取得（未設定時は default の ~/.kube/config を使う）
	kubeconfigPath := os.Getenv("KUBECONFIG")
	if kubeconfigPath == "" {
		// home dir + .kube/config のフォールバック
		homeDir, err := os.UserHomeDir()
		if err != nil {
			return nil, fmt.Errorf("KUBECONFIG env も $HOME も取得できない: %w", err)
		}
		kubeconfigPath = homeDir + "/.kube/config"
	}
	// kubeconfig file から rest.Config を構築
	restConfig, err := clientcmd.BuildConfigFromFlags("", kubeconfigPath)
	if err != nil {
		return nil, fmt.Errorf("kubeconfig %s から rest.Config 構築失敗: %w", kubeconfigPath, err)
	}
	// ClientSet を生成（Pod / Service / Node 等の core API client 集合）
	clientSet, err := kubernetes.NewForConfig(restConfig)
	if err != nil {
		return nil, fmt.Errorf("ClientSet 生成失敗: %w", err)
	}
	return &K8sClient{ClientSet: clientSet, KubeconfigPath: kubeconfigPath}, nil
}

// WaitForPodsReady は指定 namespace 内の全 Pod が Ready になるまで待機する。
// timeout 超過で error を返す。skip された Pod（Job 完了後 / Evicted）は
// kubectl wait と同じく除外する想定だが本 helper は単純 polling で実装する。
func (c *K8sClient) WaitForPodsReady(ctx context.Context, namespace string, timeout time.Duration) error {
	// poll 開始時刻を記録、deadline で打ち切り
	deadline := time.Now().Add(timeout)
	for time.Now().Before(deadline) {
		// Pod 一覧を取得（Pending / Running / Failed すべて含む）
		podList, err := c.ClientSet.CoreV1().Pods(namespace).List(ctx, metav1.ListOptions{})
		if err != nil {
			return fmt.Errorf("namespace=%s Pod 一覧取得失敗: %w", namespace, err)
		}
		// Pod が 0 件なら待機継続（namespace install 直後 / Pod がまだ作られていない可能性）
		if len(podList.Items) == 0 {
			time.Sleep(2 * time.Second)
			continue
		}
		// 全 Pod の Ready condition を集計
		allReady := true
		for _, pod := range podList.Items {
			ready := false
			for _, cond := range pod.Status.Conditions {
				if cond.Type == "Ready" && cond.Status == "True" {
					ready = true
					break
				}
			}
			if !ready {
				allReady = false
				break
			}
		}
		if allReady {
			return nil
		}
		time.Sleep(2 * time.Second)
	}
	return errors.New("Pod Ready 待機タイムアウト namespace=" + namespace)
}

// GetNodeCount は cluster の node 数を返す。owner suite は 5（3 CP + 2 W）想定。
func (c *K8sClient) GetNodeCount(ctx context.Context) (int, error) {
	// node 一覧を取得
	nodeList, err := c.ClientSet.CoreV1().Nodes().List(ctx, metav1.ListOptions{})
	if err != nil {
		return 0, fmt.Errorf("node 一覧取得失敗: %w", err)
	}
	return len(nodeList.Items), nil
}
