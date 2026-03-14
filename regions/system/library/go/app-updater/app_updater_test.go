package appupdater_test

import (
	"context"
	"testing"
	"time"

	appupdater "github.com/k1s0-platform/system-library-go-app-updater"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// CompareVersions が等しいバージョン文字列に対して 0 を返すことを確認する。
func TestCompareVersions_Equal(t *testing.T) {
	assert.Equal(t, 0, appupdater.CompareVersions("1.0.0", "1.0.0"))
}

// CompareVersions が左辺が大きい場合に 1 を返すことを確認する。
func TestCompareVersions_Greater(t *testing.T) {
	assert.Equal(t, 1, appupdater.CompareVersions("2.0.0", "1.0.0"))
	assert.Equal(t, 1, appupdater.CompareVersions("1.1.0", "1.0.0"))
	assert.Equal(t, 1, appupdater.CompareVersions("1.0.1", "1.0.0"))
}

// CompareVersions が左辺が小さい場合に -1 を返すことを確認する。
func TestCompareVersions_Lesser(t *testing.T) {
	assert.Equal(t, -1, appupdater.CompareVersions("1.0.0", "2.0.0"))
	assert.Equal(t, -1, appupdater.CompareVersions("1.0.0", "1.1.0"))
	assert.Equal(t, -1, appupdater.CompareVersions("1.0.0", "1.0.1"))
}

// CompareVersions がセグメント数が異なるバージョン文字列を正しく比較することを確認する。
func TestCompareVersions_DifferentLengths(t *testing.T) {
	assert.Equal(t, 0, appupdater.CompareVersions("1.0", "1.0.0"))
	assert.Equal(t, -1, appupdater.CompareVersions("1.0", "1.0.1"))
	assert.Equal(t, 1, appupdater.CompareVersions("1.0.1", "1.0"))
}

// CompareVersions がプレリリースサフィックスを数値部のみで比較することを確認する。
func TestCompareVersions_PreReleaseSuffix(t *testing.T) {
	// Pre-release suffixes like "-beta" are stripped to numeric parts.
	// "1.0.0-beta" → [1, 0, 0] (non-numeric chars removed from "0-beta" → "0").
	assert.Equal(t, 0, appupdater.CompareVersions("1.0.0-beta", "1.0.0"))
	// "1.0.0-rc1" → [1, 0, 1] (non-numeric chars removed from "0-rc1" → "01" → 1).
	assert.Equal(t, 0, appupdater.CompareVersions("1.0.0-rc1", "1.0.1"))
}

// DetermineUpdateType が現在バージョンが最低バージョン未満の場合に Mandatory を返すことを確認する。
func TestDetermineUpdateType_Mandatory_BelowMinimum(t *testing.T) {
	info := &appupdater.AppVersionInfo{
		LatestVersion:  "2.0.0",
		MinimumVersion: "1.5.0",
		Mandatory:      false,
	}
	result := appupdater.DetermineUpdateType("1.0.0", info)
	assert.Equal(t, appupdater.Mandatory, result)
}

// DetermineUpdateType が現在バージョンが最新バージョン未満かつ最低バージョン以上の場合に Optional を返すことを確認する。
func TestDetermineUpdateType_Optional_BelowLatest(t *testing.T) {
	info := &appupdater.AppVersionInfo{
		LatestVersion:  "2.0.0",
		MinimumVersion: "1.0.0",
		Mandatory:      false,
	}
	result := appupdater.DetermineUpdateType("1.5.0", info)
	assert.Equal(t, appupdater.Optional, result)
}

// DetermineUpdateType が現在バージョンが最新バージョンと同じ場合に None を返すことを確認する。
func TestDetermineUpdateType_None_AtLatest(t *testing.T) {
	info := &appupdater.AppVersionInfo{
		LatestVersion:  "2.0.0",
		MinimumVersion: "1.0.0",
		Mandatory:      false,
	}
	result := appupdater.DetermineUpdateType("2.0.0", info)
	assert.Equal(t, appupdater.None, result)
}

// DetermineUpdateType が Mandatory フラグが true の場合に常に Mandatory を返すことを確認する。
func TestDetermineUpdateType_Mandatory_Flag(t *testing.T) {
	info := &appupdater.AppVersionInfo{
		LatestVersion:  "2.0.0",
		MinimumVersion: "1.0.0",
		Mandatory:      true,
	}
	result := appupdater.DetermineUpdateType("2.0.0", info)
	assert.Equal(t, appupdater.Mandatory, result)
}

// InMemoryAppUpdater の FetchVersionInfo がバージョン情報を正しく返すことを確認する。
func TestInMemoryAppUpdater_FetchVersionInfo(t *testing.T) {
	now := time.Now()
	info := &appupdater.AppVersionInfo{
		LatestVersion:  "2.0.0",
		MinimumVersion: "1.0.0",
		Mandatory:      false,
		ReleaseNotes:   "New features",
		PublishedAt:    &now,
	}

	updater := appupdater.NewInMemoryAppUpdater(info, "1.5.0")
	ctx := context.Background()

	result, err := updater.FetchVersionInfo(ctx)
	require.NoError(t, err)
	assert.Equal(t, "2.0.0", result.LatestVersion)
	assert.Equal(t, "1.0.0", result.MinimumVersion)
	assert.Equal(t, "New features", result.ReleaseNotes)
	assert.False(t, result.Mandatory)
}

// InMemoryAppUpdater の CheckForUpdate が更新が必要な場合に Optional の UpdateResult を返すことを確認する。
func TestInMemoryAppUpdater_CheckForUpdate(t *testing.T) {
	info := &appupdater.AppVersionInfo{
		LatestVersion:  "2.0.0",
		MinimumVersion: "1.0.0",
		Mandatory:      false,
	}

	updater := appupdater.NewInMemoryAppUpdater(info, "1.5.0")
	ctx := context.Background()

	result, err := updater.CheckForUpdate(ctx)
	require.NoError(t, err)
	assert.Equal(t, "1.5.0", result.CurrentVersion)
	assert.Equal(t, "2.0.0", result.LatestVersion)
	assert.Equal(t, appupdater.Optional, result.UpdateType)
	assert.True(t, result.NeedsUpdate())
	assert.False(t, result.IsMandatory())
}

// InMemoryAppUpdater の CheckForUpdate が最新バージョンの場合に None の UpdateResult を返すことを確認する。
func TestInMemoryAppUpdater_CheckForUpdate_NoUpdate(t *testing.T) {
	info := &appupdater.AppVersionInfo{
		LatestVersion:  "2.0.0",
		MinimumVersion: "1.0.0",
		Mandatory:      false,
	}

	updater := appupdater.NewInMemoryAppUpdater(info, "2.0.0")
	ctx := context.Background()

	result, err := updater.CheckForUpdate(ctx)
	require.NoError(t, err)
	assert.Equal(t, appupdater.None, result.UpdateType)
	assert.False(t, result.NeedsUpdate())
}

// InMemoryAppUpdater の SetVersionInfo でバージョン情報を更新すると次の FetchVersionInfo に反映されることを確認する。
func TestInMemoryAppUpdater_SetVersionInfo(t *testing.T) {
	info := &appupdater.AppVersionInfo{
		LatestVersion:  "1.0.0",
		MinimumVersion: "1.0.0",
	}

	updater := appupdater.NewInMemoryAppUpdater(info, "1.0.0")
	ctx := context.Background()

	newInfo := &appupdater.AppVersionInfo{
		LatestVersion:  "3.0.0",
		MinimumVersion: "2.0.0",
	}
	updater.SetVersionInfo(newInfo)

	result, err := updater.FetchVersionInfo(ctx)
	require.NoError(t, err)
	assert.Equal(t, "3.0.0", result.LatestVersion)
}

// InMemoryAppUpdater の SetCurrentVersion で現在バージョンを変更すると次の CheckForUpdate の結果に反映されることを確認する。
func TestInMemoryAppUpdater_SetCurrentVersion(t *testing.T) {
	info := &appupdater.AppVersionInfo{
		LatestVersion:  "2.0.0",
		MinimumVersion: "1.0.0",
	}

	updater := appupdater.NewInMemoryAppUpdater(info, "1.0.0")
	ctx := context.Background()

	updater.SetCurrentVersion("2.0.0")

	result, err := updater.CheckForUpdate(ctx)
	require.NoError(t, err)
	assert.Equal(t, appupdater.None, result.UpdateType)
}

// UpdateType の String メソッドが各値に対応する文字列を正しく返すことを確認する。
func TestUpdateTypeString(t *testing.T) {
	assert.Equal(t, "none", appupdater.None.String())
	assert.Equal(t, "optional", appupdater.Optional.String())
	assert.Equal(t, "mandatory", appupdater.Mandatory.String())
}

// NewAppRegistryAppUpdater が不正な設定（空の ServerURL または AppID）でエラーを返すことを確認する。
func TestNewAppRegistryAppUpdater_InvalidConfig(t *testing.T) {
	_, err := appupdater.NewAppRegistryAppUpdater(appupdater.AppUpdaterConfig{
		ServerURL: "",
		AppID:     "my-app",
	}, "1.0.0")
	require.Error(t, err)

	_, err = appupdater.NewAppRegistryAppUpdater(appupdater.AppUpdaterConfig{
		ServerURL: "https://example.com",
		AppID:     "",
	}, "1.0.0")
	require.Error(t, err)
}
