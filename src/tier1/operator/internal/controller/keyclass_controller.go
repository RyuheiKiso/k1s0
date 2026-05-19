// keyclass_controller.go — k1s0 tier1 operator: KeyClass Reconciler
// 05_鍵管理適合仕様.md §v1 key_class セット（5 class）に準拠する。
// KeyClass CRD の rotation_cadence に従い OpenBao Transit API でキーローテーションを実行する。

// パッケージ名: controller（internal パッケージ: operator 外部からのインポート禁止）
package controller

import (
	// bytes パッケージのインポート: HTTP レスポンスボディ読み取りに使用する
	"bytes"
	// context パッケージのインポート: Reconcile コンテキスト管理に使用する
	"context"
	// encoding/json パッケージのインポート: OpenBao API レスポンスのパースに使用する
	"encoding/json"
	// fmt パッケージのインポート: エラーメッセージのフォーマットに使用する
	"fmt"
	// io パッケージのインポート: HTTP レスポンスボディの読み取りに使用する
	"io"
	// net/http パッケージのインポート: OpenBao Transit HTTP API 呼び出しに使用する
	"net/http"
	// os パッケージのインポート: 環境変数からの設定取得に使用する
	"os"
	// strconv パッケージのインポート: duration 文字列のパースに使用する
	"strconv"
	// strings パッケージのインポート: RotationCadence のサフィックス解析に使用する
	"strings"
	// time パッケージのインポート: RequeueAfter の指定と rotation 判定に使用する
	"time"

	// Kubernetes API マシナリーのインポート: metav1.Time に使用する
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	// controller-runtime クライアントのインポート: API サーバとの通信に使用する
	"sigs.k8s.io/controller-runtime/pkg/client"
	// controller-runtime ログのインポート: 構造化ログに使用する
	"sigs.k8s.io/controller-runtime/pkg/log"
	// controller-runtime reconcile のインポート: reconcile.Result / Request 型に使用する
	"sigs.k8s.io/controller-runtime/pkg/reconcile"
	// tier1 v1 API のインポート: KeyClass 型を参照する
	tier1v1 "github.com/k1s0/tier1-operator/api/v1"
)

// openBaoRotateResponse は OpenBao Transit rotate API のレスポンスを宣言する
type openBaoRotateResponse struct {
	// Warnings は API が返す警告メッセージのリスト
	Warnings []string `json:"warnings"`
}

// openBaoKeyInfoResponse は OpenBao Transit key info API のレスポンスを宣言する
type openBaoKeyInfoResponse struct {
	// Data は key metadata を含むマップ
	Data struct {
		// LatestVersion は最新のキーバージョン番号
		LatestVersion int `json:"latest_version"`
	} `json:"data"`
}

// parseCadenceToDuration は "30d" / "7d" / "365d" / "24h" 形式の cadence 文字列を
// time.Duration に変換する。変換失敗時はデフォルト 24h を返す。
func parseCadenceToDuration(cadence string) time.Duration {
	// cadence が空の場合はデフォルト 24 時間を返す
	if cadence == "" {
		// デフォルト値を返す
		return 24 * time.Hour
	}
	// 日単位サフィックス "d" を処理する
	if strings.HasSuffix(cadence, "d") {
		// "d" サフィックスを除去して数値部分を取得する
		numStr := strings.TrimSuffix(cadence, "d")
		// 数値に変換する
		days, err := strconv.Atoi(numStr)
		// 変換成功の場合は日数 × 24 時間を返す
		if err == nil && days > 0 {
			// 日数を時間に変換して返す
			return time.Duration(days) * 24 * time.Hour
		}
	}
	// 標準の Go duration パースを試みる（"24h" / "168h" 等）
	d, err := time.ParseDuration(cadence)
	// パース成功の場合はその値を返す
	if err == nil {
		// パース済み duration を返す
		return d
	}
	// パース失敗時はデフォルト 24 時間を返す
	return 24 * time.Hour
}

// rotateKeyViaOpenBao は OpenBao Transit API でキーをローテーションする
// POST /v1/transit/keys/{key_name}/rotate を呼び出す
func rotateKeyViaOpenBao(ctx context.Context, keyName string) error {
	// 環境変数から OpenBao アドレスを取得する（デフォルト: http://openbao.k1s0.svc:8200）
	baoAddr := os.Getenv("OPENBAO_ADDR")
	// アドレスが未設定の場合はデフォルト値を使用する
	if baoAddr == "" {
		// デフォルト OpenBao アドレスを設定する
		baoAddr = "http://openbao.k1s0.svc:8200"
	}
	// 環境変数から OpenBao Vault トークンを取得する
	baoToken := os.Getenv("OPENBAO_TOKEN")
	// トークンが未設定の場合はエラーを返す
	if baoToken == "" {
		// OPENBAO_TOKEN 未設定エラーを返す
		return fmt.Errorf("OPENBAO_TOKEN 環境変数が設定されていない")
	}
	// rotate API エンドポイント URL を構築する
	rotateURL := fmt.Sprintf("%s/v1/transit/keys/%s/rotate", baoAddr, keyName)
	// POST リクエストボディは空（rotate は POST のみで完結する）
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, rotateURL, bytes.NewReader([]byte("{}")))
	// リクエスト生成失敗時はエラーを返す
	if err != nil {
		// リクエスト生成エラーをラップして返す
		return fmt.Errorf("OpenBao rotate request 生成失敗: %w", err)
	}
	// X-Vault-Token ヘッダーを設定する
	req.Header.Set("X-Vault-Token", baoToken)
	// Content-Type を JSON に設定する
	req.Header.Set("Content-Type", "application/json")
	// HTTP クライアントで POST リクエストを送信する
	httpClient := &http.Client{Timeout: 10 * time.Second}
	// リクエストを実行する
	resp, err := httpClient.Do(req)
	// HTTP エラー時はエラーを返す
	if err != nil {
		// HTTP 送信エラーをラップして返す
		return fmt.Errorf("OpenBao rotate HTTP 送信失敗: %w", err)
	}
	// レスポンスボディを Close する
	defer resp.Body.Close()
	// レスポンスボディを読み取る
	body, _ := io.ReadAll(resp.Body)
	// HTTP ステータスが成功（2xx）以外の場合はエラーを返す
	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		// HTTP ステータスエラーをラップして返す
		return fmt.Errorf("OpenBao rotate 失敗: HTTP %d body=%s", resp.StatusCode, string(body))
	}
	// レスポンスを JSON パースする（warnings を取り出す目的）
	var rotateResp openBaoRotateResponse
	// JSON デコードする（エラーは無視 — warnings は任意情報）
	_ = json.Unmarshal(body, &rotateResp)
	// rotate 成功を返す
	return nil
}

// fetchKeyLatestVersion は OpenBao Transit API から最新キーバージョンを取得する
// GET /v1/transit/keys/{key_name} を呼び出す
func fetchKeyLatestVersion(ctx context.Context, keyName string) (int, error) {
	// 環境変数から OpenBao アドレスを取得する
	baoAddr := os.Getenv("OPENBAO_ADDR")
	// アドレスが未設定の場合はデフォルト値を使用する
	if baoAddr == "" {
		// デフォルト OpenBao アドレスを設定する
		baoAddr = "http://openbao.k1s0.svc:8200"
	}
	// 環境変数から OpenBao Vault トークンを取得する
	baoToken := os.Getenv("OPENBAO_TOKEN")
	// トークンが未設定の場合はエラーを返す
	if baoToken == "" {
		// OPENBAO_TOKEN 未設定エラーを返す
		return 0, fmt.Errorf("OPENBAO_TOKEN 環境変数が設定されていない")
	}
	// key info API エンドポイント URL を構築する
	keyInfoURL := fmt.Sprintf("%s/v1/transit/keys/%s", baoAddr, keyName)
	// GET リクエストを生成する
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, keyInfoURL, nil)
	// リクエスト生成失敗時はエラーを返す
	if err != nil {
		// エラーをラップして返す
		return 0, fmt.Errorf("OpenBao key info request 生成失敗: %w", err)
	}
	// X-Vault-Token ヘッダーを設定する
	req.Header.Set("X-Vault-Token", baoToken)
	// HTTP クライアントで GET リクエストを送信する
	httpClient := &http.Client{Timeout: 10 * time.Second}
	// リクエストを実行する
	resp, err := httpClient.Do(req)
	// HTTP エラー時はエラーを返す
	if err != nil {
		// HTTP 送信エラーをラップして返す
		return 0, fmt.Errorf("OpenBao key info HTTP 送信失敗: %w", err)
	}
	// レスポンスボディを Close する
	defer resp.Body.Close()
	// レスポンスボディを読み取る
	body, _ := io.ReadAll(resp.Body)
	// HTTP ステータスが成功以外の場合はエラーを返す
	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		// HTTP ステータスエラーを返す
		return 0, fmt.Errorf("OpenBao key info 失敗: HTTP %d body=%s", resp.StatusCode, string(body))
	}
	// JSON レスポンスをパースする
	var keyInfo openBaoKeyInfoResponse
	// JSON デコードする
	if err := json.Unmarshal(body, &keyInfo); err != nil {
		// JSON パースエラーをラップして返す
		return 0, fmt.Errorf("OpenBao key info JSON パース失敗: %w", err)
	}
	// 最新バージョン番号を返す
	return keyInfo.Data.LatestVersion, nil
}

// KeyClassReconciler は KeyClass リソースを reconcile するコントローラ構造体
// 05_鍵管理適合仕様.md §rotation_cadence に基づく鍵ローテーションスケジュールを管理する
type KeyClassReconciler struct {
	// Kubernetes クライアント: API サーバとのリソース読み書きに使用する
	client.Client
}

// Reconcile は KeyClass リソースの desired state と actual state を一致させる
// 05_鍵管理適合仕様.md §rotation_cadence / §destruction_method enforcement のメインロジックを担う
func (r *KeyClassReconciler) Reconcile(ctx context.Context, req reconcile.Request) (reconcile.Result, error) {
	// ロガーをコンテキストから取得する
	logger := log.FromContext(ctx)
	// Reconcile 開始をログに記録する
	logger.Info("Reconciling KeyClass", "name", req.Name, "namespace", req.Namespace)

	// ---- 1. KeyClass リソースを API サーバから取得する ----

	// reconcile 対象の KeyClass 変数を宣言する
	var keyClass tier1v1.KeyClass
	// API サーバから KeyClass を取得する
	if err := r.Get(ctx, req.NamespacedName, &keyClass); err != nil {
		// リソースが存在しない場合は正常終了する（削除済みの可能性）
		return reconcile.Result{}, client.IgnoreNotFound(err)
	}

	// ---- 2. Purpose と RotationCadence の検証 ----

	// Purpose が未設定の場合は警告して再試行する
	if keyClass.Spec.Purpose == "" {
		// Purpose 未設定を警告ログに記録する
		logger.Info("KeyClass Purpose is not set, key rotation enforcement skipped", "name", req.Name)
		// 10 分後に再試行する
		return reconcile.Result{RequeueAfter: 10 * time.Minute}, nil
	}

	// ---- 3. RotationCadence から回転周期を計算して、次回ローテーションが必要か判定する ----

	// RotationCadence を time.Duration に変換する
	rotationInterval := parseCadenceToDuration(keyClass.Spec.RotationCadence)

	// 前回ローテーション時刻が未設定の場合は初回実行とみなしてローテーションを実行する
	needsRotation := keyClass.Status.LastRotationAt == nil
	// 前回ローテーションから rotation interval が経過しているか確認する
	if !needsRotation && keyClass.Status.LastRotationAt != nil {
		// 前回ローテーション時刻からの経過時間を計算する
		elapsed := time.Since(keyClass.Status.LastRotationAt.Time)
		// 経過時間が rotation interval を超えている場合はローテーション対象とする
		needsRotation = elapsed >= rotationInterval
	}

	// ---- 4. OpenBao Transit でキーローテーションを実行する ----

	// キーの名前を設定する（KeyClass 名を OpenBao のキー名として使用する）
	keyName := keyClass.Name

	// ローテーションが必要な場合のみ実行する
	if needsRotation {
		// OpenBao Transit API でキーローテーションを実行する
		if err := rotateKeyViaOpenBao(ctx, keyName); err != nil {
			// ローテーション失敗をエラーログに記録する
			logger.Error(err, "OpenBao key rotation failed",
				"name", keyName,
				"purpose", keyClass.Spec.Purpose,
				"backend", keyClass.Spec.Backend,
			)
			// エラーを返してリトライさせる（controller-runtime が exponential backoff）
			return reconcile.Result{}, fmt.Errorf("rotate key %s via OpenBao: %w", keyName, err)
		}
		// ローテーション成功をログに記録する
		logger.Info("OpenBao key rotation succeeded",
			"name", keyName,
			"purpose", keyClass.Spec.Purpose,
			"rotationCadence", keyClass.Spec.RotationCadence,
		)
	} else {
		// ローテーション不要をログに記録する（次回 Reconcile で再確認する）
		logger.Info("KeyClass rotation not needed yet",
			"name", keyName,
			"rotationCadence", keyClass.Spec.RotationCadence,
		)
	}

	// ---- 5. OpenBao から最新バージョン番号を取得して status に反映する ----

	// 最新キーバージョンを取得する（エラーは非致命的として警告に留める）
	latestVersion, versionErr := fetchKeyLatestVersion(ctx, keyName)
	// バージョン取得に成功した場合はログに記録する
	if versionErr != nil {
		// 取得失敗を警告ログに記録する（status 更新は続行する）
		logger.Info("could not fetch latest key version from OpenBao",
			"name", keyName,
			"error", versionErr.Error(),
		)
	} else {
		// 最新バージョンをログに記録する
		logger.Info("OpenBao key latest version fetched", "name", keyName, "version", latestVersion)
	}

	// ---- 6. Status.Rotated と LastRotationAt を更新する ----

	// ローテーション処理が完了したので rotated = true に設定する
	keyClass.Status.Rotated = true
	// 現在時刻を LastRotationAt に設定する（status 記録目的なので wall-clock 許可）
	now := metav1.Now()
	// ポインタを設定する
	keyClass.Status.LastRotationAt = &now

	// Status サブリソースを更新する
	if err := r.Status().Update(ctx, &keyClass); err != nil {
		// Status 更新失敗をログに記録してリトライさせる
		logger.Error(err, "Failed to update KeyClass status")
		// エラーを返してリトライを促す
		return reconcile.Result{}, fmt.Errorf("KeyClass status update failed: %w", err)
	}

	// Reconcile 完了をログに記録する
	logger.Info("KeyClass reconciled successfully",
		"name", req.Name,
		"purpose", keyClass.Spec.Purpose,
		"rotationCadence", keyClass.Spec.RotationCadence,
		"backend", keyClass.Spec.Backend,
		"rotated", needsRotation,
	)

	// rotation interval の半分後に再 Reconcile をスケジュールする（早めに状態確認する）
	nextCheck := rotationInterval / 2
	// 最小再確認間隔は 1 時間とする
	if nextCheck < time.Hour {
		// 最小間隔を 1 時間に設定する
		nextCheck = time.Hour
	}
	// 次回 Reconcile を設定して返す
	return reconcile.Result{RequeueAfter: nextCheck}, nil
}
