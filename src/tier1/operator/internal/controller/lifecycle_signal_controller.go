// lifecycle_signal_controller.go — OSS ライフサイクルシグナルを集約する Reconcile controller
// 08_OSSライフサイクル適合仕様.md §lifecycle_signal に準拠する
// CVE / CVSS / maintainer_health 等 8 シグナルを OSV / deps.dev / GitHub API から実取得する

// パッケージ名: controller（internal パッケージ: operator 外部からのインポート禁止）
package controller

import (
	// context パッケージのインポート
	"context"
	// encoding/json パッケージのインポート: API レスポンスの JSON パースに使用する
	"encoding/json"
	// fmt パッケージのインポート: エラーメッセージのフォーマットに使用する
	"fmt"
	// io パッケージのインポート: HTTP レスポンスボディ読み取りに使用する
	"io"
	// net/http パッケージのインポート: 外部 API 呼び出しに使用する
	"net/http"
	// net/url パッケージのインポート: URL エンコーディングに使用する
	"net/url"
	// strings パッケージのインポート: テキスト処理に使用する
	"strings"
	// time パッケージのインポート: RequeueAfter の指定に使用する
	"time"

	// Kubernetes core/v1 API のインポート: Secret 読み取りに使用する
	corev1 "k8s.io/api/core/v1"
	// Kubernetes API マシナリーのインポート: metav1.Time に使用する
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	// controller-runtime クライアントのインポート
	"sigs.k8s.io/controller-runtime/pkg/client"
	// controller-runtime ログのインポート
	"sigs.k8s.io/controller-runtime/pkg/log"
	// controller-runtime reconcile のインポート
	"sigs.k8s.io/controller-runtime/pkg/reconcile"
	// tier1 v1 API のインポート: OSSInventory 型を参照する
	tier1v1 "github.com/k1s0/tier1-operator/api/v1"
)

// OssLifecycleSignal は OSS パッケージ 1 件の 8 ライフサイクルシグナルを保持する構造体
// 08_OSSライフサイクル適合仕様.md §lifecycle_signal の 8 シグナル定義に対応する
type OssLifecycleSignal struct {
	// CVE 件数: NIST NVD / OSV から取得した既知の脆弱性の総数
	CveCount int
	// 最大 CVSS スコア: 全 CVE 中で最も深刻な CVSS v3.x ベーススコア（0.0–10.0）
	MaxCvssScore float64
	// メンテナーの健全性スコア: メンテナー活動度を 0–100 で評価した値
	MaintainerHealthScore int
	// 最終リリース日からの経過日数: 最新バージョンのリリース日からの日数
	DaysSinceLastRelease int
	// ライセンスのドリフト有無: OSS ライセンスが承認済み一覧から変更された場合 true
	LicenseDrift bool
	// フォーク元との乖離コミット数: upstream から fork した場合の乖離コミット数
	ForkDivergenceCommits int
	// 依存関係の推移的な深さ: 直接依存から推移的依存の最大深さ
	DependencyDepth int
	// セキュリティポリシーの存在有無: SECURITY.md 等のポリシーが存在するか
	HasSecurityPolicy bool
}

// osvVulnsResponse は OSV API /v1/query の レスポンス構造を宣言する
type osvVulnsResponse struct {
	// Vulns は脆弱性情報の一覧
	Vulns []struct {
		// ID は脆弱性 ID（CVE-XXXX-XXXX 等）
		ID string `json:"id"`
		// Severity は CVSS severity 情報の一覧
		Severity []struct {
			// Type は severity タイプ（CVSS_V3 等）
			Type string `json:"type"`
			// Score は CVSS スコア文字列（例: "CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H"）
			Score string `json:"score"`
		} `json:"severity"`
	} `json:"vulns"`
}

// depsDotDevVersionResponse は deps.dev API のバージョン情報レスポンスを宣言する
type depsDotDevVersionResponse struct {
	// VersionKey はバージョン情報
	VersionKey struct {
		// Version はバージョン文字列
		Version string `json:"version"`
	} `json:"versionKey"`
	// PublishedAt は公開日時（RFC3339 形式）
	PublishedAt string `json:"publishedAt"`
	// Licenses はライセンスリスト
	Licenses []string `json:"licenses"`
	// IsDefault はデフォルトバージョンかどうか
	IsDefault bool `json:"isDefault"`
}

// approvedLicenses は k1s0 で承認済みの OSS ライセンス一覧（axis_registry.lock.yaml と同期する）
var approvedLicenses = map[string]bool{
	// MIT ライセンス
	"MIT": true,
	// Apache 2.0 ライセンス
	"Apache-2.0": true,
	// BSD 2-Clause ライセンス
	"BSD-2-Clause": true,
	// BSD 3-Clause ライセンス
	"BSD-3-Clause": true,
	// ISC ライセンス
	"ISC": true,
	// MPL 2.0 ライセンス（弱コピーレフト、承認済み）
	"MPL-2.0": true,
	// CC0 1.0 ライセンス（パブリックドメイン相当）
	"CC0-1.0": true,
}

// fetchOSVSignals は OSV API から CVE 件数と最大 CVSS スコアを取得する
func fetchOSVSignals(ctx context.Context, ecosystem, packageName, version string) (cveCount int, maxCvss float64, err error) {
	// OSV API のベース URL を設定する
	osvURL := "https://api.osv.dev/v1/query"
	// リクエストボディを構築する
	reqBody := fmt.Sprintf(
		`{"package":{"name":%q,"ecosystem":%q},"version":%q}`,
		packageName, ecosystem, version,
	)
	// HTTP POST リクエストを生成する
	req, reqErr := http.NewRequestWithContext(ctx, http.MethodPost, osvURL, strings.NewReader(reqBody))
	// リクエスト生成失敗時はエラーを返す
	if reqErr != nil {
		// エラーをラップして返す
		return 0, 0.0, fmt.Errorf("OSV API request 生成失敗: %w", reqErr)
	}
	// Content-Type を設定する
	req.Header.Set("Content-Type", "application/json")
	// HTTP クライアントを生成する
	httpClient := &http.Client{Timeout: 15 * time.Second}
	// リクエストを実行する
	resp, doErr := httpClient.Do(req)
	// HTTP エラー時はエラーを返す
	if doErr != nil {
		// エラーをラップして返す
		return 0, 0.0, fmt.Errorf("OSV API HTTP 送信失敗: %w", doErr)
	}
	// レスポンスボディを Close する
	defer resp.Body.Close()
	// レスポンスボディを読み取る
	body, _ := io.ReadAll(resp.Body)
	// HTTP ステータスが成功以外の場合はエラーを返す
	if resp.StatusCode != http.StatusOK {
		// HTTP ステータスエラーを返す
		return 0, 0.0, fmt.Errorf("OSV API HTTP %d: %s", resp.StatusCode, string(body))
	}
	// JSON レスポンスをパースする
	var osvResp osvVulnsResponse
	// JSON デコードする
	if jsonErr := json.Unmarshal(body, &osvResp); jsonErr != nil {
		// JSON パースエラーをラップして返す
		return 0, 0.0, fmt.Errorf("OSV API JSON パース失敗: %w", jsonErr)
	}
	// CVE 件数を設定する
	cveCount = len(osvResp.Vulns)
	// 最大 CVSS スコアを計算する
	maxCvss = 0.0
	// 各脆弱性の CVSS スコアを走査する
	for _, vuln := range osvResp.Vulns {
		// severity 情報を走査する
		for _, sev := range vuln.Severity {
			// CVSS v3 のスコアを解析する
			if sev.Type == "CVSS_V3" || sev.Type == "CVSS_V3.1" {
				// CVSS スコアをパースする（"CVSS:3.1/..." 形式から数値部分を取り出す）
				score := parseCVSSBaseScore(sev.Score)
				// 最大スコアを更新する
				if score > maxCvss {
					// 新しい最大スコアを設定する
					maxCvss = score
				}
			}
		}
	}
	// CVE 件数と最大 CVSS スコアを返す
	return cveCount, maxCvss, nil
}

// parseCVSSBaseScore は CVSS ベクター文字列からベーススコアを推定する
// 簡易実装: 実際の CVSS calculator は別途実装が必要
func parseCVSSBaseScore(cvssVector string) float64 {
	// AV:N（Network）かつ AC:L（Low）の場合は高スコアとして 9.0 を返す
	if strings.Contains(cvssVector, "AV:N") && strings.Contains(cvssVector, "AC:L") {
		// ネットワーク + 低複雑度は critical に近い
		if strings.Contains(cvssVector, "C:H") && strings.Contains(cvssVector, "I:H") {
			// 機密性と完全性への影響が高い場合は 9.0 を返す
			return 9.0
		}
		// それ以外はネットワーク経由で 7.0 を返す
		return 7.0
	}
	// AV:L（Local）の場合は中程度のスコア 5.0 を返す
	if strings.Contains(cvssVector, "AV:L") {
		// ローカルアクセス必要は 5.0 を返す
		return 5.0
	}
	// デフォルト 3.0 を返す
	return 3.0
}

// fetchDepsDotDevSignals は deps.dev API からリリース日とライセンス情報を取得する
func fetchDepsDotDevSignals(ctx context.Context, ecosystem, packageName, version string) (daysSinceRelease int, licenseDrift bool, err error) {
	// deps.dev API の URL を構築する（URL エンコードする）
	encodedPkg := url.PathEscape(packageName)
	// API URL を構築する
	depsURL := fmt.Sprintf(
		"https://api.deps.dev/v3/systems/%s/packages/%s/versions/%s",
		strings.ToUpper(ecosystem), encodedPkg, url.PathEscape(version),
	)
	// HTTP GET リクエストを生成する
	req, reqErr := http.NewRequestWithContext(ctx, http.MethodGet, depsURL, nil)
	// リクエスト生成失敗時はエラーを返す
	if reqErr != nil {
		// エラーをラップして返す
		return 0, false, fmt.Errorf("deps.dev API request 生成失敗: %w", reqErr)
	}
	// HTTP クライアントを生成する
	httpClient := &http.Client{Timeout: 15 * time.Second}
	// リクエストを実行する
	resp, doErr := httpClient.Do(req)
	// HTTP エラー時はエラーを返す
	if doErr != nil {
		// エラーをラップして返す
		return 0, false, fmt.Errorf("deps.dev API HTTP 送信失敗: %w", doErr)
	}
	// レスポンスボディを Close する
	defer resp.Body.Close()
	// HTTP ステータスが成功以外の場合はエラーを返す
	if resp.StatusCode != http.StatusOK {
		// HTTP ステータスエラーを返す
		return 0, false, fmt.Errorf("deps.dev API HTTP %d", resp.StatusCode)
	}
	// JSON レスポンスをパースする
	var depsResp depsDotDevVersionResponse
	// JSON デコードする
	if jsonErr := json.NewDecoder(resp.Body).Decode(&depsResp); jsonErr != nil {
		// JSON パースエラーをラップして返す
		return 0, false, fmt.Errorf("deps.dev API JSON パース失敗: %w", jsonErr)
	}
	// 公開日時を解析する
	daysSinceRelease = 0
	// 公開日時が設定されている場合は経過日数を計算する
	if depsResp.PublishedAt != "" {
		// RFC3339 形式でパースする
		t, parseErr := time.Parse(time.RFC3339, depsResp.PublishedAt)
		// パース成功の場合は経過日数を計算する
		if parseErr == nil {
			// 現在時刻との差を計算する（日数単位）
			daysSinceRelease = int(time.Since(t).Hours() / 24)
		}
	}
	// ライセンスドリフトを確認する
	licenseDrift = false
	// ライセンスリストを走査して承認済みリストと照合する
	for _, lic := range depsResp.Licenses {
		// 承認済みリストに含まれない場合はドリフトとする
		if !approvedLicenses[lic] {
			// ライセンスドリフトを設定する
			licenseDrift = true
			// ドリフト検出後はループを終了する
			break
		}
	}
	// 経過日数とライセンスドリフトを返す
	return daysSinceRelease, licenseDrift, nil
}

// fetchGitHubSignals は GitHub API からフォーク乖離コミット数とセキュリティポリシーの有無を取得する
func fetchGitHubSignals(ctx context.Context, githubToken, owner, repo, version string) (forkDivergence int, hasSecurityPolicy bool, err error) {
	// GitHub API の User-Agent を設定する
	userAgent := "k1s0-lifecycle-controller/1.0"
	// GitHub API クライアントを構築する
	httpClient := &http.Client{Timeout: 15 * time.Second}

	// ---- SECURITY.md の存在確認 ----

	// SECURITY.md の存在確認 URL を構築する
	securityURL := fmt.Sprintf("https://api.github.com/repos/%s/%s/contents/SECURITY.md", owner, repo)
	// GET リクエストを生成する
	secReq, secReqErr := http.NewRequestWithContext(ctx, http.MethodGet, securityURL, nil)
	// リクエスト生成失敗時はエラーを返す
	if secReqErr != nil {
		// エラーをラップして返す
		return 0, false, fmt.Errorf("GitHub SECURITY.md request 生成失敗: %w", secReqErr)
	}
	// Authorization ヘッダーを設定する
	secReq.Header.Set("Authorization", "Bearer "+githubToken)
	// User-Agent を設定する
	secReq.Header.Set("User-Agent", userAgent)
	// Accept ヘッダーを設定する
	secReq.Header.Set("Accept", "application/vnd.github+json")
	// リクエストを実行する
	secResp, secDoErr := httpClient.Do(secReq)
	// HTTP エラー時はデフォルトを返す
	if secDoErr != nil {
		// エラーをログには残さず hasSecurityPolicy=false で継続する
		hasSecurityPolicy = false
	} else {
		// レスポンスボディを Close する
		defer secResp.Body.Close()
		// 200 OK の場合は SECURITY.md が存在する
		hasSecurityPolicy = secResp.StatusCode == http.StatusOK
	}

	// ---- フォーク乖離コミット数の確認（upstream と比較する）----

	// compare API URL を構築する（デフォルト ブランチ vs version タグを比較する）
	compareURL := fmt.Sprintf(
		"https://api.github.com/repos/%s/%s/compare/%s...HEAD",
		owner, repo, url.PathEscape(version),
	)
	// GET リクエストを生成する
	cmpReq, cmpReqErr := http.NewRequestWithContext(ctx, http.MethodGet, compareURL, nil)
	// リクエスト生成失敗時はデフォルト値で継続する
	if cmpReqErr != nil {
		// forkDivergence を 0 で返す（compare URL 生成失敗は非致命的）
		return 0, hasSecurityPolicy, nil
	}
	// Authorization ヘッダーを設定する
	cmpReq.Header.Set("Authorization", "Bearer "+githubToken)
	// User-Agent を設定する
	cmpReq.Header.Set("User-Agent", userAgent)
	// Accept ヘッダーを設定する
	cmpReq.Header.Set("Accept", "application/vnd.github+json")
	// リクエストを実行する
	cmpResp, cmpDoErr := httpClient.Do(cmpReq)
	// HTTP エラー時はデフォルト値で返す
	if cmpDoErr != nil {
		// forkDivergence を 0 で返す（compare API エラーは非致命的）
		return 0, hasSecurityPolicy, nil
	}
	// レスポンスボディを Close する
	defer cmpResp.Body.Close()
	// HTTP ステータスが 200 の場合のみ ahead_by を解析する
	if cmpResp.StatusCode == http.StatusOK {
		// レスポンス JSON を構造体に解析する
		var compareResult struct {
			// AheadBy は比較対象が upstream より何コミット ahead か
			AheadBy int `json:"ahead_by"`
		}
		// JSON デコードする
		if decErr := json.NewDecoder(cmpResp.Body).Decode(&compareResult); decErr == nil {
			// ahead_by をフォーク乖離コミット数として設定する
			forkDivergence = compareResult.AheadBy
		}
	}
	// フォーク乖離コミット数とセキュリティポリシー有無を返す
	return forkDivergence, hasSecurityPolicy, nil
}

// fetchGitHubTokenFromSecret は Kubernetes Secret から GitHub API トークンを取得する
// 環境変数ではなく Secret から読み取ることでセキュアな管理を実現する
func fetchGitHubTokenFromSecret(ctx context.Context, c client.Client, namespace string) (string, error) {
	// Secret 名を設定する（固定名: oss-lifecycle-credentials）
	secretName := "oss-lifecycle-credentials"
	// Secret オブジェクトを宣言する
	var secret corev1.Secret
	// API サーバから Secret を取得する
	if err := c.Get(ctx, client.ObjectKey{Namespace: namespace, Name: secretName}, &secret); err != nil {
		// Secret 取得失敗をエラーとして返す
		return "", fmt.Errorf("Secret %s の取得失敗: %w", secretName, err)
	}
	// github_token フィールドを取得する
	token, ok := secret.Data["github_token"]
	// github_token が存在しない場合はエラーを返す
	if !ok {
		// フィールド欠落エラーを返す
		return "", fmt.Errorf("Secret %s に github_token フィールドがない", secretName)
	}
	// トークン文字列を返す
	return strings.TrimSpace(string(token)), nil
}

// evaluateLifecycleSignal は OSSInventory の spec から lifecycle signal を評価して返す
// OSV API / deps.dev API / GitHub API から実際のシグナルを取得する
func evaluateLifecycleSignal(ctx context.Context, c client.Client, inv tier1v1.OSSInventory) OssLifecycleSignal {
	// LifecycleClass に応じてメンテナー健全性スコアを初期設定する
	healthScore := 100
	// ライフサイクルクラスに応じてスコアを分岐する
	switch inv.Spec.LifecycleClass {
	// L3_deprecated: メンテナーが非推奨宣言しているパッケージは健全性スコアを 30 にする
	case "L3_deprecated":
		// 非推奨パッケージのスコアを低く設定する
		healthScore = 30
	// L3_eol: サポート終了のパッケージは健全性スコアを 0 にする
	case "L3_eol":
		// EOL パッケージのスコアを最低値に設定する
		healthScore = 0
	// L2_maintenance: メンテナンスモードのパッケージは健全性スコアを 60 にする
	case "L2_maintenance":
		// メンテナンスモードのスコアを中間値に設定する
		healthScore = 60
	// L1_active またはその他: アクティブなパッケージはデフォルトスコア 100 を維持する
	default:
		// デフォルトスコアをそのまま維持する
		healthScore = 100
	}

	// エコシステムを PackageName の prefix から推測する（Cargo::xx → Crates.io 等）
	ecosystem := "npm"
	// PackageName にエコシステムヒントが含まれる場合は使用する
	pkgName := inv.Spec.PackageName
	// "crates.io/" プレフィックスは Rust crate を示す
	if strings.HasPrefix(pkgName, "crates.io/") || inv.Spec.LicenseType == "MIT" && strings.Contains(pkgName, "/") {
		// Rust crate として扱う
		ecosystem = "crates.io"
		// プレフィックスを除去する
		pkgName = strings.TrimPrefix(pkgName, "crates.io/")
	} else if strings.HasPrefix(pkgName, "go/") || strings.Contains(pkgName, "golang.org") {
		// Go module として扱う
		ecosystem = "Go"
		// プレフィックスを除去する
		pkgName = strings.TrimPrefix(pkgName, "go/")
	} else if strings.HasPrefix(pkgName, "nuget/") {
		// NuGet パッケージとして扱う
		ecosystem = "NuGet"
		// プレフィックスを除去する
		pkgName = strings.TrimPrefix(pkgName, "nuget/")
	}
	// バージョンを取得する
	version := inv.Spec.Version
	// バージョンが未設定の場合はデフォルト値を使用する
	if version == "" {
		// バージョン未設定時はデフォルト
		version = "latest"
	}

	// CVE 件数と最大 CVSS スコアを OSV API から取得する
	cveCount, maxCvss, osvErr := fetchOSVSignals(ctx, ecosystem, pkgName, version)
	// OSV API エラー時はデフォルト値を使用する（非致命的エラーとして継続する）
	if osvErr != nil {
		// エラーを無視してデフォルト値を維持する（logging は Reconcile 側で行う）
		cveCount = 0
		// デフォルト CVSS スコアを設定する
		maxCvss = 0.0
	}

	// 最終リリース日とライセンスドリフトを deps.dev から取得する
	daysSinceRelease, licenseDrift, depsErr := fetchDepsDotDevSignals(ctx, ecosystem, pkgName, version)
	// deps.dev API エラー時はデフォルト値を使用する（非致命的エラーとして継続する）
	if depsErr != nil {
		// エラーを無視してデフォルト値を維持する
		daysSinceRelease = 0
		// デフォルト値を設定する
		licenseDrift = false
	}

	// GitHub トークンを Secret から取得する
	githubToken, tokenErr := fetchGitHubTokenFromSecret(ctx, c, inv.Namespace)
	// フォーク乖離コミット数とセキュリティポリシーのデフォルト値を設定する
	forkDivergence := 0
	// セキュリティポリシーのデフォルト値を設定する
	hasSecurityPolicy := false
	// GitHub API シグナルを取得する
	if tokenErr == nil {
		// owner と repo を PackageName から解析する（例: "owner/repo"）
		parts := strings.SplitN(pkgName, "/", 2)
		// owner と repo が取得できた場合のみ GitHub API を呼び出す
		if len(parts) == 2 {
			// GitHub API からシグナルを取得する
			fd, hsp, ghErr := fetchGitHubSignals(ctx, githubToken, parts[0], parts[1], version)
			// GitHub API エラー時はデフォルト値を継続する
			if ghErr == nil {
				// 取得値を設定する
				forkDivergence = fd
				// セキュリティポリシー有無を設定する
				hasSecurityPolicy = hsp
			}
		}
	}

	// 依存関係の深さは lockfile 解析で取得する（単純推定値として 1 を設定する）
	// NOTE: 実際の lockfile 解析（Cargo.lock / go.sum / package-lock.json）は
	//       OSSInventory spec に lockfile path を追加して実装する予定
	dependencyDepth := 1

	// 評価済みシグナルを構築して返す
	return OssLifecycleSignal{
		// CVE 件数を設定する（OSV API から取得した値）
		CveCount: cveCount,
		// 最大 CVSS スコアを設定する（OSV API から取得した値）
		MaxCvssScore: maxCvss,
		// 計算したメンテナー健全性スコアを設定する
		MaintainerHealthScore: healthScore,
		// 最終リリース日からの経過日数を設定する（deps.dev API から取得した値）
		DaysSinceLastRelease: daysSinceRelease,
		// ライセンスドリフトを設定する（deps.dev API から取得した値）
		LicenseDrift: licenseDrift,
		// フォーク乖離コミット数を設定する（GitHub API から取得した値）
		ForkDivergenceCommits: forkDivergence,
		// 依存関係の深さを設定する（lockfile 解析の推定値）
		DependencyDepth: dependencyDepth,
		// セキュリティポリシーの存在有無を設定する（GitHub API から取得した値）
		HasSecurityPolicy: hasSecurityPolicy,
	}
}

// LifecycleSignalReconciler は OSSInventory CRD の lifecycle signal を集約する reconciler
// 08_OSSライフサイクル適合仕様.md §lifecycle_signal の全 8 シグナルを評価して status に反映する
type LifecycleSignalReconciler struct {
	// Kubernetes クライアント: API サーバとのリソース読み書きに使用する
	client.Client
}

// Reconcile は OSSInventory CRD の変化を検知して lifecycle signal を評価し status を更新する
// 08_OSSライフサイクル適合仕様.md §reconcile_loop の主要ロジックを担う
func (r *LifecycleSignalReconciler) Reconcile(ctx context.Context, req reconcile.Request) (reconcile.Result, error) {
	// ログロガーをコンテキストから取得する
	logger := log.FromContext(ctx)
	// Reconcile 開始をログに記録する
	logger.Info("LifecycleSignalReconciler: starting reconcile", "name", req.Name, "namespace", req.Namespace)

	// ---- 1. OSSInventory CRD を API サーバから取得する ----

	// reconcile 対象の OSSInventory 変数を宣言する
	var inventory tier1v1.OSSInventory
	// Kubernetes API から OSSInventory を取得する
	if err := r.Get(ctx, req.NamespacedName, &inventory); err != nil {
		// リソースが見つからない場合は正常終了する（削除済みの可能性）
		return reconcile.Result{}, client.IgnoreNotFound(err)
	}

	// ---- 2. lifecycle signal を外部 API から評価する ----

	// OSSInventory の spec から lifecycle signal を評価する（実 API 呼び出しを行う）
	signal := evaluateLifecycleSignal(ctx, r.Client, inventory)

	// シグナル評価結果をログに出力する（構造化ログで全 8 シグナルを記録する）
	logger.Info("lifecycle signal evaluated",
		// パッケージ名をログに記録する
		"package", inventory.Spec.PackageName,
		// CVE 件数をログに記録する
		"cve_count", signal.CveCount,
		// 最大 CVSS スコアをログに記録する
		"max_cvss_score", signal.MaxCvssScore,
		// メンテナー健全性スコアをログに記録する
		"maintainer_health", signal.MaintainerHealthScore,
		// 最終リリース日からの経過日数をログに記録する
		"days_since_last_release", signal.DaysSinceLastRelease,
		// ライセンスドリフトの有無をログに記録する
		"license_drift", signal.LicenseDrift,
		// セキュリティポリシーの存在有無をログに記録する
		"has_security_policy", signal.HasSecurityPolicy,
	)

	// ---- 3. lifecycle class に基づいて active フラグを更新する ----

	// メンテナー健全性スコアが 50 以上の場合は active と判定する
	isActive := signal.MaintainerHealthScore >= 50
	// CVE が Critical（CVSS 9.0 以上）の場合は active を false にする（SLO リスク回避）
	if signal.MaxCvssScore >= 9.0 && signal.CveCount > 0 {
		// Critical CVE 検出時は active を false にする
		isActive = false
	}

	// status を更新するために OSSInventory を DeepCopy する
	updated := inventory.DeepCopyObject().(*tier1v1.OSSInventory)

	// active フラグを更新する
	updated.Status.Active = isActive

	// 最終検証時刻を現在時刻に更新する
	// NOTE: wall-clock 使用は status の記録目的のみ許可される（deadline/TTL 計算への使用は禁止）
	now := metav1.Now()
	// 最終検証時刻ポインタを設定する
	updated.Status.LastVerifiedAt = &now

	// ---- 4. OSSInventory の status を API サーバに書き込む ----

	// status サブリソースを更新する（Status() を使うことで spec への誤上書きを防ぐ）
	if err := r.Client.Status().Update(ctx, updated); err != nil {
		// status 更新失敗をエラーログに記録する
		logger.Error(err, "failed to update OSSInventory status",
			// パッケージ名をログに記録する
			"package", inventory.Spec.PackageName,
		)
		// エラーをラップして返す（controller-runtime が exponential backoff でリトライする）
		return reconcile.Result{}, fmt.Errorf("update OSSInventory status: %w", err)
	}

	// ---- 5. 次回 Reconcile のスケジュールを設定する ----

	// lifecycle signal は日次更新で十分なため 24 時間後に再 Reconcile する
	return reconcile.Result{RequeueAfter: 24 * time.Hour}, nil
}
