// k1s0 tier2 operator の Go モジュール定義
module github.com/k1s0/tier2-operator

// Go バージョン: 1.23 を使用する
go 1.23.0

// ツールチェーンバージョンを固定する
toolchain go1.23.4

// 直接依存関係
require (
	// k8s.io/apimachinery: Kubernetes API マシナリーパッケージ
	k8s.io/apimachinery v0.32.0
	// k8s.io/client-go: Kubernetes クライアントライブラリ
	k8s.io/client-go v0.32.0 // indirect
	// controller-runtime: Kubernetes コントローラ構築ライブラリ v0.20.0
	sigs.k8s.io/controller-runtime v0.20.0
)

require k8s.io/api v0.32.0

require (
	// anyhow 相当: beorn7/perks は Prometheus ヒストグラム用ライブラリ
	github.com/beorn7/perks v1.0.1 // indirect
	// xxhash v2: Prometheus ラベルハッシュに使用する
	github.com/cespare/xxhash/v2 v2.3.0 // indirect
	// go-spew: デバッグ用構造体表示ライブラリ
	github.com/davecgh/go-spew v1.1.2-0.20180830191138-d8f796af33cc // indirect
	// go-restful v3: Kubernetes API サーバ内部で使用する REST フレームワーク
	github.com/emicklei/go-restful/v3 v3.11.0 // indirect
	// json-patch v5: JSON Patch 適用ライブラリ
	github.com/evanphx/json-patch/v5 v5.9.0 // indirect
	// fsnotify: ファイルシステム変更通知ライブラリ
	github.com/fsnotify/fsnotify v1.7.0 // indirect
	// cbor v2: CBOR エンコード / デコードライブラリ
	github.com/fxamacker/cbor/v2 v2.7.0 // indirect
	// go-logr: 構造化ログインターフェース
	github.com/go-logr/logr v1.4.2 // indirect
	// go-logr zapr: zap バックエンドのロガー
	github.com/go-logr/zapr v1.3.0 // indirect
	// go-openapi jsonpointer: OpenAPI JSON Pointer 実装
	github.com/go-openapi/jsonpointer v0.21.0 // indirect
	// go-openapi jsonreference: OpenAPI JSON Reference 実装
	github.com/go-openapi/jsonreference v0.20.2 // indirect
	// go-openapi swag: OpenAPI swagger ユーティリティ
	github.com/go-openapi/swag v0.23.0 // indirect
	// gogo/protobuf: Kubernetes 内部 Protobuf エンコーダ
	github.com/gogo/protobuf v1.3.2 // indirect
	// golang/protobuf: Protobuf v1 互換ラッパー
	github.com/golang/protobuf v1.5.4 // indirect
	// google/btree: B-tree データ構造ライブラリ
	github.com/google/btree v1.1.3 // indirect
	// google/gnostic-models: OpenAPI モデルライブラリ
	github.com/google/gnostic-models v0.6.8 // indirect
	// google/go-cmp: テスト用差分比較ライブラリ
	github.com/google/go-cmp v0.6.0 // indirect
	// google/gofuzz: ファジング用ライブラリ
	github.com/google/gofuzz v1.2.0 // indirect
	// google/uuid: UUID 生成ライブラリ
	github.com/google/uuid v1.6.0 // indirect
	// josharian/intern: 文字列インターン化ライブラリ
	github.com/josharian/intern v1.0.0 // indirect
	// json-iterator/go: 高速 JSON ライブラリ
	github.com/json-iterator/go v1.1.12 // indirect
	// mailru/easyjson: 高速 JSON 直列化ライブラリ
	github.com/mailru/easyjson v0.7.7 // indirect
	// modern-go/concurrent: スレッドセーフなマップ実装
	github.com/modern-go/concurrent v0.0.0-20180306012644-bacd9c7ef1dd // indirect
	// modern-go/reflect2: 高速リフレクションライブラリ
	github.com/modern-go/reflect2 v1.0.2 // indirect
	// munnerz/goautoneg: HTTP コンテントネゴシエーションライブラリ
	github.com/munnerz/goautoneg v0.0.0-20191010083416-a7dc8b61c822 // indirect
	// pkg/errors: エラーラッピングライブラリ
	github.com/pkg/errors v0.9.1 // indirect
	// prometheus/client_golang: Prometheus メトリクスクライアント
	github.com/prometheus/client_golang v1.19.1 // indirect
	// prometheus/client_model: Prometheus データモデル
	github.com/prometheus/client_model v0.6.1 // indirect
	// prometheus/common: Prometheus 共通ライブラリ
	github.com/prometheus/common v0.55.0 // indirect
	// prometheus/procfs: Linux /proc ファイルシステムパーサー
	github.com/prometheus/procfs v0.15.1 // indirect
	// spf13/pflag: POSIX 準拠フラグライブラリ
	github.com/spf13/pflag v1.0.5 // indirect
	// x448/float16: 16 ビット浮動小数点ライブラリ
	github.com/x448/float16 v0.8.4 // indirect
	// go.uber.org/multierr: 複数エラーの結合ライブラリ
	go.uber.org/multierr v1.11.0 // indirect
	// go.uber.org/zap: 高性能構造化ロガー
	go.uber.org/zap v1.27.0 // indirect
	// golang.org/x/net: Go 拡張ネットワークライブラリ
	golang.org/x/net v0.30.0 // indirect
	// golang.org/x/oauth2: OAuth2 クライアントライブラリ
	golang.org/x/oauth2 v0.23.0 // indirect
	// golang.org/x/sync: 同期プリミティブ拡張ライブラリ
	golang.org/x/sync v0.8.0 // indirect
	// golang.org/x/sys: OS システムコールラッパー
	golang.org/x/sys v0.26.0 // indirect
	// golang.org/x/term: ターミナル操作ライブラリ
	golang.org/x/term v0.25.0 // indirect
	// golang.org/x/text: テキスト処理ライブラリ
	golang.org/x/text v0.19.0 // indirect
	// golang.org/x/time: 時間ユーティリティライブラリ
	golang.org/x/time v0.7.0 // indirect
	// gomodules.xyz/jsonpatch v2: JSON Patch 生成ライブラリ
	gomodules.xyz/jsonpatch/v2 v2.4.0 // indirect
	// google.golang.org/protobuf: Protobuf v2 ライブラリ
	google.golang.org/protobuf v1.36.0 // indirect
	// gopkg.in/evanphx/json-patch.v4: JSON Patch v4 ライブラリ
	gopkg.in/evanphx/json-patch.v4 v4.12.0 // indirect
	// gopkg.in/inf.v0: 任意精度小数ライブラリ
	gopkg.in/inf.v0 v0.9.1 // indirect
	// gopkg.in/yaml.v3: YAML パーサーライブラリ
	gopkg.in/yaml.v3 v3.0.1 // indirect
	// k8s.io/apiextensions-apiserver: CRD API 拡張サーバライブラリ
	k8s.io/apiextensions-apiserver v0.32.0 // indirect
	// k8s.io/klog v2: Kubernetes ロギングライブラリ
	k8s.io/klog/v2 v2.130.1 // indirect
	// k8s.io/kube-openapi: Kubernetes OpenAPI 生成ライブラリ
	k8s.io/kube-openapi v0.0.0-20241105132330-32ad38e42d3f // indirect
	// k8s.io/utils: Kubernetes ユーティリティライブラリ
	k8s.io/utils v0.0.0-20241104100929-3ea5e8cea738 // indirect
	// sigs.k8s.io/json: Kubernetes JSON ライブラリ
	sigs.k8s.io/json v0.0.0-20241010143419-9aa6b5e7a4b3 // indirect
	// sigs.k8s.io/structured-merge-diff v4: Kubernetes 構造化マージ差分ライブラリ
	sigs.k8s.io/structured-merge-diff/v4 v4.4.2 // indirect
	// sigs.k8s.io/yaml: Kubernetes YAML ライブラリ
	sigs.k8s.io/yaml v1.4.0 // indirect
)
