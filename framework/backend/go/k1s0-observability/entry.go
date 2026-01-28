package k1s0observability

import (
	"encoding/json"
	"time"
)

// LogLevel represents a log level.
type LogLevel string

const (
	// LevelDebug represents debug level.
	LevelDebug LogLevel = "DEBUG"
	// LevelInfo represents info level.
	LevelInfo LogLevel = "INFO"
	// LevelWarn represents warn level.
	LevelWarn LogLevel = "WARN"
	// LevelError represents error level.
	LevelError LogLevel = "ERROR"
)

// LogEntry represents a structured log entry with required fields.
type LogEntry struct {
	// Required fields
	Timestamp   string   `json:"timestamp"`
	Level       LogLevel `json:"level"`
	Message     string   `json:"message"`
	ServiceName string   `json:"service.name"`
	ServiceEnv  string   `json:"service.env"`
	TraceID     string   `json:"trace.id,omitempty"`
	RequestID   string   `json:"request.id,omitempty"`

	// Optional fields
	SpanID      string            `json:"span.id,omitempty"`
	UserID      string            `json:"user.id,omitempty"`
	TenantID    string            `json:"tenant.id,omitempty"`
	Version     string            `json:"service.version,omitempty"`
	InstanceID  string            `json:"service.instance_id,omitempty"`
	ErrorCode   string            `json:"error.code,omitempty"`
	ErrorStack  string            `json:"error.stack,omitempty"`
	Extra       map[string]string `json:"extra,omitempty"`
	CustomFields map[string]any   `json:"-"`
}

// NewLogEntry creates a new LogEntry with the given level and message.
func NewLogEntry(level LogLevel, message string) *LogEntry {
	return &LogEntry{
		Timestamp: time.Now().UTC().Format(time.RFC3339Nano),
		Level:     level,
		Message:   message,
		Extra:     make(map[string]string),
		CustomFields: make(map[string]any),
	}
}

// Debug creates a debug level log entry.
func Debug(message string) *LogEntry {
	return NewLogEntry(LevelDebug, message)
}

// Info creates an info level log entry.
func Info(message string) *LogEntry {
	return NewLogEntry(LevelInfo, message)
}

// Warn creates a warn level log entry.
func Warn(message string) *LogEntry {
	return NewLogEntry(LevelWarn, message)
}

// Error creates an error level log entry.
func Error(message string) *LogEntry {
	return NewLogEntry(LevelError, message)
}

// WithContext adds RequestContext fields to the entry.
func (e *LogEntry) WithContext(ctx *RequestContext) *LogEntry {
	if ctx == nil {
		return e
	}
	e.TraceID = ctx.TraceID
	e.RequestID = ctx.RequestID
	e.SpanID = ctx.SpanID
	e.UserID = ctx.UserID
	e.TenantID = ctx.TenantID
	for k, v := range ctx.Extra {
		e.Extra[k] = v
	}
	return e
}

// WithService adds service information to the entry.
func (e *LogEntry) WithService(config *Config) *LogEntry {
	if config == nil {
		return e
	}
	e.ServiceName = config.ServiceName
	e.ServiceEnv = config.Env
	e.Version = config.Version
	e.InstanceID = config.InstanceID
	return e
}

// WithError adds error information to the entry.
func (e *LogEntry) WithError(err error, errorCode string) *LogEntry {
	if err != nil {
		e.ErrorCode = errorCode
		// In production, you might want to capture stack traces differently
	}
	return e
}

// WithField adds a custom field to the entry.
func (e *LogEntry) WithField(key string, value any) *LogEntry {
	if e.CustomFields == nil {
		e.CustomFields = make(map[string]any)
	}
	e.CustomFields[key] = value
	return e
}

// ToJSON converts the entry to JSON bytes.
func (e *LogEntry) ToJSON() ([]byte, error) {
	// If there are custom fields, we need to merge them into the output
	if len(e.CustomFields) == 0 {
		return json.Marshal(e)
	}

	// Marshal the base entry first
	data, err := json.Marshal(e)
	if err != nil {
		return nil, err
	}

	// Unmarshal into a map to merge custom fields
	var result map[string]any
	if err := json.Unmarshal(data, &result); err != nil {
		return nil, err
	}

	// Add custom fields
	for k, v := range e.CustomFields {
		result[k] = v
	}

	return json.Marshal(result)
}

// String returns the JSON string representation.
func (e *LogEntry) String() string {
	data, err := e.ToJSON()
	if err != nil {
		return ""
	}
	return string(data)
}
