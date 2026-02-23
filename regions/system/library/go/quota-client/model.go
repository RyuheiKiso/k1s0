package quotaclient

import "time"

// QuotaPeriod はクォータの期間。
type QuotaPeriod int

const (
	PeriodHourly  QuotaPeriod = iota
	PeriodDaily
	PeriodMonthly
	PeriodCustom
)

// QuotaStatus はクォータチェック結果。
type QuotaStatus struct {
	Allowed   bool      `json:"allowed"`
	Remaining uint64    `json:"remaining"`
	Limit     uint64    `json:"limit"`
	ResetAt   time.Time `json:"reset_at"`
}

// QuotaUsage はクォータ使用量。
type QuotaUsage struct {
	QuotaID string      `json:"quota_id"`
	Used    uint64      `json:"used"`
	Limit   uint64      `json:"limit"`
	Period  QuotaPeriod `json:"period"`
	ResetAt time.Time   `json:"reset_at"`
}

// QuotaPolicy はクォータポリシー。
type QuotaPolicy struct {
	QuotaID       string      `json:"quota_id"`
	Limit         uint64      `json:"limit"`
	Period        QuotaPeriod `json:"period"`
	ResetStrategy string      `json:"reset_strategy"`
}
