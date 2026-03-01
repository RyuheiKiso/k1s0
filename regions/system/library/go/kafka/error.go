package kafka

import "fmt"

// KafkaError は Kafka 操作に関するエラーを表す。
type KafkaError struct {
	// Op はエラーが発生した操作名。
	Op string
	// Message はエラーメッセージ。
	Message string
	// Err は原因となったエラー。
	Err error
}

// Error はエラーメッセージを返す。
func (e *KafkaError) Error() string {
	if e.Err != nil {
		return fmt.Sprintf("%s: %s: %v", e.Op, e.Message, e.Err)
	}
	return fmt.Sprintf("%s: %s", e.Op, e.Message)
}

// Unwrap は原因となったエラーを返す。
func (e *KafkaError) Unwrap() error {
	return e.Err
}
