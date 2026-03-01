package webhookclient

import "time"

// ExportSleepFunc は現在のsleepFuncを返す（テスト用）。
func ExportSleepFunc() func(time.Duration) {
	return sleepFunc
}

// SetSleepFunc はsleepFuncを置き換える（テスト用）。
func SetSleepFunc(f func(time.Duration)) {
	sleepFunc = f
}
