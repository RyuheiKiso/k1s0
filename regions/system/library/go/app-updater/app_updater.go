package appupdater

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"strconv"
	"strings"
	"sync"
	"time"
)

// UpdateType はアップデートの種類。
type UpdateType int

const (
	// None はアップデート不要。
	None UpdateType = iota
	// Optional はオプションのアップデート。
	Optional
	// Mandatory は必須のアップデート。
	Mandatory
)

// String は UpdateType の文字列表現を返す。
func (u UpdateType) String() string {
	switch u {
	case None:
		return "none"
	case Optional:
		return "optional"
	case Mandatory:
		return "mandatory"
	default:
		return "unknown"
	}
}

// AppVersionInfo はアプリバージョン情報。
type AppVersionInfo struct {
	LatestVersion  string     `json:"latest_version"`
	MinimumVersion string     `json:"minimum_version"`
	Mandatory      bool       `json:"mandatory"`
	ReleaseNotes   string     `json:"release_notes,omitempty"`
	PublishedAt    *time.Time `json:"published_at,omitempty"`
}

// UpdateCheckResult はアップデートチェック結果。
type UpdateCheckResult struct {
	CurrentVersion string       `json:"current_version"`
	LatestVersion  string       `json:"latest_version"`
	MinimumVersion string       `json:"minimum_version"`
	UpdateType     UpdateType   `json:"update_type"`
	ReleaseNotes   string       `json:"release_notes,omitempty"`
}

// NeedsUpdate はアップデートが必要かどうかを返す。
func (r *UpdateCheckResult) NeedsUpdate() bool {
	return r.UpdateType != None
}

// IsMandatory は必須アップデートかどうかを返す。
func (r *UpdateCheckResult) IsMandatory() bool {
	return r.UpdateType == Mandatory
}

// DownloadArtifactInfo はダウンロードアーティファクト情報。
type DownloadArtifactInfo struct {
	URL       string     `json:"url"`
	Checksum  string     `json:"checksum"`
	Size      int64      `json:"size"`
	ExpiresAt *time.Time `json:"expires_at,omitempty"`
}

// --- Errors ---

// AppUpdaterError はアプリアップデーターの基本エラー。
type AppUpdaterError struct {
	Code    string `json:"code"`
	Message string `json:"message"`
}

func (e *AppUpdaterError) Error() string {
	return fmt.Sprintf("AppUpdaterError(%s): %s", e.Code, e.Message)
}

// ConnectionError は接続エラー。
type ConnectionError struct {
	AppUpdaterError
}

// NewConnectionError は新しい ConnectionError を生成する。
func NewConnectionError(message string) *ConnectionError {
	return &ConnectionError{AppUpdaterError{Code: "CONNECTION_ERROR", Message: message}}
}

// InvalidConfigError は無効な設定エラー。
type InvalidConfigError struct {
	AppUpdaterError
}

// NewInvalidConfigError は新しい InvalidConfigError を生成する。
func NewInvalidConfigError(message string) *InvalidConfigError {
	return &InvalidConfigError{AppUpdaterError{Code: "INVALID_CONFIG", Message: message}}
}

// ParseError はパースエラー。
type ParseError struct {
	AppUpdaterError
}

// NewParseError は新しい ParseError を生成する。
func NewParseError(message string) *ParseError {
	return &ParseError{AppUpdaterError{Code: "PARSE_ERROR", Message: message}}
}

// UnauthorizedError は認証エラー。
type UnauthorizedError struct {
	AppUpdaterError
}

// NewUnauthorizedError は新しい UnauthorizedError を生成する。
func NewUnauthorizedError(message string) *UnauthorizedError {
	return &UnauthorizedError{AppUpdaterError{Code: "UNAUTHORIZED", Message: message}}
}

// AppNotFoundError はアプリ未検出エラー。
type AppNotFoundError struct {
	AppUpdaterError
}

// NewAppNotFoundError は新しい AppNotFoundError を生成する。
func NewAppNotFoundError(message string) *AppNotFoundError {
	return &AppNotFoundError{AppUpdaterError{Code: "APP_NOT_FOUND", Message: message}}
}

// VersionNotFoundError はバージョン未検出エラー。
type VersionNotFoundError struct {
	AppUpdaterError
}

// NewVersionNotFoundError は新しい VersionNotFoundError を生成する。
func NewVersionNotFoundError(message string) *VersionNotFoundError {
	return &VersionNotFoundError{AppUpdaterError{Code: "VERSION_NOT_FOUND", Message: message}}
}

// ChecksumError はチェックサムエラー。
type ChecksumError struct {
	AppUpdaterError
}

// NewChecksumError は新しい ChecksumError を生成する。
func NewChecksumError(message string) *ChecksumError {
	return &ChecksumError{AppUpdaterError{Code: "CHECKSUM_ERROR", Message: message}}
}

// --- Config ---

// AppUpdaterConfig はアプリアップデーターの設定。
type AppUpdaterConfig struct {
	ServerURL     string        `json:"server_url"`
	AppID         string        `json:"app_id"`
	Platform      string        `json:"platform,omitempty"`
	Arch          string        `json:"arch,omitempty"`
	CheckInterval time.Duration `json:"check_interval,omitempty"`
	Timeout       time.Duration `json:"timeout,omitempty"`
}

// --- Interface ---

// AppUpdater はアプリアップデーターのインターフェース。
type AppUpdater interface {
	FetchVersionInfo(ctx context.Context) (*AppVersionInfo, error)
	CheckForUpdate(ctx context.Context) (*UpdateCheckResult, error)
}

// --- AppRegistryAppUpdater ---

// AppRegistryAppUpdater はレジストリAPIを使うアプリアップデーター。
type AppRegistryAppUpdater struct {
	config         AppUpdaterConfig
	currentVersion string
	httpClient     *http.Client
}

// NewAppRegistryAppUpdater は新しい AppRegistryAppUpdater を生成する。
func NewAppRegistryAppUpdater(config AppUpdaterConfig, currentVersion string) (*AppRegistryAppUpdater, error) {
	if strings.TrimSpace(config.ServerURL) == "" {
		return nil, NewInvalidConfigError("serverURL must not be empty")
	}
	if strings.TrimSpace(config.AppID) == "" {
		return nil, NewInvalidConfigError("appID must not be empty")
	}

	timeout := config.Timeout
	if timeout == 0 {
		timeout = 10 * time.Second
	}

	return &AppRegistryAppUpdater{
		config:         config,
		currentVersion: currentVersion,
		httpClient:     &http.Client{Timeout: timeout},
	}, nil
}

// FetchVersionInfo はバージョン情報を取得する。
func (a *AppRegistryAppUpdater) FetchVersionInfo(ctx context.Context) (*AppVersionInfo, error) {
	url := fmt.Sprintf("%s/api/v1/apps/%s/versions/latest", strings.TrimRight(a.config.ServerURL, "/"), a.config.AppID)

	if a.config.Platform != "" {
		url += "?platform=" + a.config.Platform
		if a.config.Arch != "" {
			url += "&arch=" + a.config.Arch
		}
	} else if a.config.Arch != "" {
		url += "?arch=" + a.config.Arch
	}

	req, err := http.NewRequestWithContext(ctx, http.MethodGet, url, nil)
	if err != nil {
		return nil, NewConnectionError(fmt.Sprintf("failed to create request: %v", err))
	}

	resp, err := a.httpClient.Do(req)
	if err != nil {
		return nil, NewConnectionError(fmt.Sprintf("failed to fetch version info: %v", err))
	}
	defer resp.Body.Close()

	switch resp.StatusCode {
	case http.StatusUnauthorized, http.StatusForbidden:
		return nil, NewUnauthorizedError("unauthorized access to registry API")
	case http.StatusNotFound:
		return nil, NewAppNotFoundError(fmt.Sprintf("app not found: %s", a.config.AppID))
	}

	if resp.StatusCode != http.StatusOK {
		return nil, NewConnectionError(fmt.Sprintf("unexpected status code: %d", resp.StatusCode))
	}

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, NewConnectionError(fmt.Sprintf("failed to read response body: %v", err))
	}

	var info AppVersionInfo
	if err := json.Unmarshal(body, &info); err != nil {
		return nil, NewParseError(fmt.Sprintf("failed to parse version info: %v", err))
	}

	return &info, nil
}

// CheckForUpdate はアップデートチェックを行う。
func (a *AppRegistryAppUpdater) CheckForUpdate(ctx context.Context) (*UpdateCheckResult, error) {
	versionInfo, err := a.FetchVersionInfo(ctx)
	if err != nil {
		return nil, err
	}

	updateType := DetermineUpdateType(a.currentVersion, versionInfo)

	return &UpdateCheckResult{
		CurrentVersion: a.currentVersion,
		LatestVersion:  versionInfo.LatestVersion,
		MinimumVersion: versionInfo.MinimumVersion,
		UpdateType:     updateType,
		ReleaseNotes:   versionInfo.ReleaseNotes,
	}, nil
}

// --- InMemoryAppUpdater ---

// InMemoryAppUpdater はテスト用のインメモリアプリアップデーター。
type InMemoryAppUpdater struct {
	versionInfo    *AppVersionInfo
	currentVersion string
	mu             sync.RWMutex
}

// NewInMemoryAppUpdater は新しい InMemoryAppUpdater を生成する。
func NewInMemoryAppUpdater(versionInfo *AppVersionInfo, currentVersion string) *InMemoryAppUpdater {
	return &InMemoryAppUpdater{
		versionInfo:    versionInfo,
		currentVersion: currentVersion,
	}
}

// FetchVersionInfo はバージョン情報を返す。
func (m *InMemoryAppUpdater) FetchVersionInfo(_ context.Context) (*AppVersionInfo, error) {
	m.mu.RLock()
	defer m.mu.RUnlock()
	return m.versionInfo, nil
}

// CheckForUpdate はアップデートチェック結果を返す。
func (m *InMemoryAppUpdater) CheckForUpdate(_ context.Context) (*UpdateCheckResult, error) {
	m.mu.RLock()
	defer m.mu.RUnlock()

	updateType := DetermineUpdateType(m.currentVersion, m.versionInfo)

	return &UpdateCheckResult{
		CurrentVersion: m.currentVersion,
		LatestVersion:  m.versionInfo.LatestVersion,
		MinimumVersion: m.versionInfo.MinimumVersion,
		UpdateType:     updateType,
		ReleaseNotes:   m.versionInfo.ReleaseNotes,
	}, nil
}

// SetVersionInfo はバージョン情報を設定する。
func (m *InMemoryAppUpdater) SetVersionInfo(info *AppVersionInfo) {
	m.mu.Lock()
	defer m.mu.Unlock()
	m.versionInfo = info
}

// SetCurrentVersion は現在のバージョンを設定する。
func (m *InMemoryAppUpdater) SetCurrentVersion(version string) {
	m.mu.Lock()
	defer m.mu.Unlock()
	m.currentVersion = version
}

// --- Helper functions ---

// CompareVersions は2つのバージョン文字列を比較する。
// left < right なら負の値、left == right なら 0、left > right なら正の値を返す。
func CompareVersions(left, right string) int {
	leftParts := normalizeVersion(left)
	rightParts := normalizeVersion(right)

	length := max(len(leftParts), len(rightParts))

	for i := range length {
		leftValue := 0
		if i < len(leftParts) {
			leftValue = leftParts[i]
		}
		rightValue := 0
		if i < len(rightParts) {
			rightValue = rightParts[i]
		}
		if leftValue != rightValue {
			if leftValue < rightValue {
				return -1
			}
			return 1
		}
	}

	return 0
}

// DetermineUpdateType はアップデートの種類を判定する。
func DetermineUpdateType(currentVersion string, versionInfo *AppVersionInfo) UpdateType {
	if CompareVersions(currentVersion, versionInfo.MinimumVersion) < 0 || versionInfo.Mandatory {
		return Mandatory
	}

	if CompareVersions(currentVersion, versionInfo.LatestVersion) < 0 {
		return Optional
	}

	return None
}

func normalizeVersion(version string) []int {
	segments := strings.Split(version, ".")
	result := make([]int, 0, len(segments))
	for _, segment := range segments {
		// Strip non-numeric characters.
		cleaned := strings.Map(func(r rune) rune {
			if r >= '0' && r <= '9' {
				return r
			}
			return -1
		}, segment)
		n, err := strconv.Atoi(cleaned)
		if err != nil {
			n = 0
		}
		result = append(result, n)
	}
	return result
}
