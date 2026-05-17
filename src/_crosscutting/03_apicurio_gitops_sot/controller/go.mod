// k1s0 Apicurio GitOps コントローラの Go モジュール定義
module github.com/k1s0/apicurio-controller

// Go バージョン: 1.23 を使用する
go 1.23

// 依存関係: controller-runtime v0.20.0 を使用する
require (
	// controller-runtime: Kubernetes コントローラ構築ライブラリ
	sigs.k8s.io/controller-runtime v0.20.0
	// k8s.io/apimachinery: Kubernetes API マシナリーパッケージ
	k8s.io/apimachinery v0.32.0
	// k8s.io/client-go: Kubernetes クライアントライブラリ
	k8s.io/client-go v0.32.0
	// go-resty: HTTP クライアントライブラリ (Apicurio API 呼び出しに使用)
	github.com/go-resty/resty/v2 v2.16.2
)
