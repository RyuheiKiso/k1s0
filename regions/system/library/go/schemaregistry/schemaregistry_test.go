package schemaregistry_test

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	sr "github.com/k1s0-platform/system-library-go-schemaregistry"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func newTestServer(t *testing.T, handler http.HandlerFunc) (*httptest.Server, *sr.SchemaRegistryConfig) {
	t.Helper()
	server := httptest.NewServer(handler)
	t.Cleanup(server.Close)
	return server, &sr.SchemaRegistryConfig{URL: server.URL}
}

// RegisterSchemaがスキーマを正常に登録し、サーバーから返されたIDを返すことを確認する。
func TestRegisterSchema(t *testing.T) {
	server, cfg := newTestServer(t, func(w http.ResponseWriter, r *http.Request) {
		assert.Equal(t, http.MethodPost, r.Method)
		assert.Contains(t, r.URL.Path, "/subjects/user.created.v1-value/versions")
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(map[string]int{"id": 42})
	})
	_ = server

	client, err := sr.NewClient(cfg)
	require.NoError(t, err)

	id, err := client.RegisterSchema(context.Background(), "user.created.v1-value", `{"type":"record"}`, "AVRO")
	require.NoError(t, err)
	assert.Equal(t, 42, id)
}

// サーバーが404を返した場合にRegisterSchemaがNotFoundエラーを返すことを確認する。
func TestRegisterSchema_NotFound(t *testing.T) {
	server, cfg := newTestServer(t, func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusNotFound)
	})
	_ = server

	client, err := sr.NewClient(cfg)
	require.NoError(t, err)

	_, err = client.RegisterSchema(context.Background(), "nonexistent", `{}`, "AVRO")
	assert.True(t, sr.IsNotFound(err))
}

// GetSchemaByIDがIDに対応するスキーマをスキーマレジストリから取得することを確認する。
func TestGetSchemaByID(t *testing.T) {
	server, cfg := newTestServer(t, func(w http.ResponseWriter, r *http.Request) {
		assert.Equal(t, "/schemas/ids/42", r.URL.Path)
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(sr.RegisteredSchema{
			Schema:     `{"type":"record"}`,
			SchemaType: "AVRO",
		})
	})
	_ = server

	client, err := sr.NewClient(cfg)
	require.NoError(t, err)

	schema, err := client.GetSchemaByID(context.Background(), 42)
	require.NoError(t, err)
	assert.Equal(t, 42, schema.ID)
	assert.Equal(t, `{"type":"record"}`, schema.Schema)
}

// 存在しないIDでGetSchemaByIDを呼び出した際にNotFoundエラーが返ることを確認する。
func TestGetSchemaByID_NotFound(t *testing.T) {
	server, cfg := newTestServer(t, func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusNotFound)
	})
	_ = server

	client, err := sr.NewClient(cfg)
	require.NoError(t, err)

	_, err = client.GetSchemaByID(context.Background(), 999)
	assert.True(t, sr.IsNotFound(err))
}

// GetLatestSchemaがサブジェクトの最新バージョンのスキーマを取得することを確認する。
func TestGetLatestSchema(t *testing.T) {
	server, cfg := newTestServer(t, func(w http.ResponseWriter, r *http.Request) {
		assert.Equal(t, "/subjects/user.created.v1-value/versions/latest", r.URL.Path)
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(sr.RegisteredSchema{
			ID:      42,
			Subject: "user.created.v1-value",
			Version: 3,
			Schema:  `{"type":"record"}`,
		})
	})
	_ = server

	client, err := sr.NewClient(cfg)
	require.NoError(t, err)

	schema, err := client.GetLatestSchema(context.Background(), "user.created.v1-value")
	require.NoError(t, err)
	assert.Equal(t, 3, schema.Version)
}

// GetSchemaVersionが指定したバージョン番号のスキーマを取得することを確認する。
func TestGetSchemaVersion(t *testing.T) {
	server, cfg := newTestServer(t, func(w http.ResponseWriter, r *http.Request) {
		assert.Equal(t, "/subjects/user.created.v1-value/versions/2", r.URL.Path)
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(sr.RegisteredSchema{Version: 2})
	})
	_ = server

	client, err := sr.NewClient(cfg)
	require.NoError(t, err)

	schema, err := client.GetSchemaVersion(context.Background(), "user.created.v1-value", 2)
	require.NoError(t, err)
	assert.Equal(t, 2, schema.Version)
}

// ListSubjectsがスキーマレジストリに登録された全サブジェクトのリストを返すことを確認する。
func TestListSubjects(t *testing.T) {
	server, cfg := newTestServer(t, func(w http.ResponseWriter, r *http.Request) {
		assert.Equal(t, "/subjects", r.URL.Path)
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode([]string{"user.created.v1-value", "order.placed.v1-value"})
	})
	_ = server

	client, err := sr.NewClient(cfg)
	require.NoError(t, err)

	subjects, err := client.ListSubjects(context.Background())
	require.NoError(t, err)
	assert.Len(t, subjects, 2)
	assert.Contains(t, subjects, "user.created.v1-value")
}

// CheckCompatibilityがスキーマの互換性チェックでtrueを返すケースを確認する。
func TestCheckCompatibility_Compatible(t *testing.T) {
	server, cfg := newTestServer(t, func(w http.ResponseWriter, r *http.Request) {
		assert.Equal(t, "/compatibility/subjects/user.created.v1-value/versions/latest", r.URL.Path)
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(map[string]bool{"is_compatible": true})
	})
	_ = server

	client, err := sr.NewClient(cfg)
	require.NoError(t, err)

	compatible, err := client.CheckCompatibility(context.Background(), "user.created.v1-value", `{"type":"record"}`)
	require.NoError(t, err)
	assert.True(t, compatible)
}

// CheckCompatibilityがスキーマの互換性チェックでfalseを返すケースを確認する。
func TestCheckCompatibility_Incompatible(t *testing.T) {
	server, cfg := newTestServer(t, func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(map[string]bool{"is_compatible": false})
	})
	_ = server

	client, err := sr.NewClient(cfg)
	require.NoError(t, err)

	compatible, err := client.CheckCompatibility(context.Background(), "subject", `{}`)
	require.NoError(t, err)
	assert.False(t, compatible)
}

// HealthCheckがサーバーが正常な場合にエラーなしで成功することを確認する。
func TestHealthCheck_Healthy(t *testing.T) {
	server, cfg := newTestServer(t, func(w http.ResponseWriter, r *http.Request) {
		assert.Equal(t, "/", r.URL.Path)
		w.WriteHeader(http.StatusOK)
	})
	_ = server

	client, err := sr.NewClient(cfg)
	require.NoError(t, err)

	err = client.HealthCheck(context.Background())
	assert.NoError(t, err)
}

// HealthCheckがサーバーが503を返した場合にエラーを返すことを確認する。
func TestHealthCheck_Unhealthy(t *testing.T) {
	server, cfg := newTestServer(t, func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusServiceUnavailable)
	})
	_ = server

	client, err := sr.NewClient(cfg)
	require.NoError(t, err)

	err = client.HealthCheck(context.Background())
	assert.Error(t, err)
}

// SubjectNameがトピック名とサフィックスを結合して正しいサブジェクト名を生成することを確認する。
func TestSchemaRegistryConfig_SubjectName(t *testing.T) {
	cfg := &sr.SchemaRegistryConfig{URL: "http://localhost:8081"}
	assert.Equal(t, "k1s0.system.user.created.v1-value", cfg.SubjectName("k1s0.system.user.created.v1", "value"))
	assert.Equal(t, "k1s0.system.user.created.v1-key", cfg.SubjectName("k1s0.system.user.created.v1", "key"))
}

// URLが空のInvalidConfigでNewClientを呼び出した際にエラーが返ることを確認する。
func TestNewClient_InvalidConfig(t *testing.T) {
	_, err := sr.NewClient(&sr.SchemaRegistryConfig{URL: ""})
	assert.Error(t, err)
}

// NotFoundErrorがエラーメッセージにnot foundを含み、IsNotFoundがtrueを返すことを確認する。
func TestNotFoundError(t *testing.T) {
	err := &sr.NotFoundError{Resource: "schema id=42"}
	assert.Contains(t, err.Error(), "not found")
	assert.True(t, sr.IsNotFound(err))
}

// SchemaRegistryErrorがステータスコードとメッセージを含み、IsNotFoundがfalseを返すことを確認する。
func TestSchemaRegistryError(t *testing.T) {
	err := &sr.SchemaRegistryError{StatusCode: 500, Message: "internal error"}
	assert.Contains(t, err.Error(), "500")
	assert.Contains(t, err.Error(), "internal error")
	assert.False(t, sr.IsNotFound(err))
}

// 存在しないサブジェクトでGetLatestSchemaを呼び出した際にNotFoundエラーが返ることを確認する。
func TestGetLatestSchema_NotFound(t *testing.T) {
	server, cfg := newTestServer(t, func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusNotFound)
	})
	_ = server

	client, err := sr.NewClient(cfg)
	require.NoError(t, err)

	_, err = client.GetLatestSchema(context.Background(), "nonexistent-subject")
	assert.True(t, sr.IsNotFound(err))
}

// 存在しないサブジェクトとバージョンでGetSchemaVersionを呼び出した際にNotFoundエラーが返ることを確認する。
func TestGetSchemaVersion_NotFound(t *testing.T) {
	server, cfg := newTestServer(t, func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusNotFound)
	})
	_ = server

	client, err := sr.NewClient(cfg)
	require.NoError(t, err)

	_, err = client.GetSchemaVersion(context.Background(), "nonexistent-subject", 99)
	assert.True(t, sr.IsNotFound(err))
}

// サブジェクトが存在しない場合にListSubjectsが空のリストを返すことを確認する。
func TestListSubjects_Empty(t *testing.T) {
	server, cfg := newTestServer(t, func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode([]string{})
	})
	_ = server

	client, err := sr.NewClient(cfg)
	require.NoError(t, err)

	subjects, err := client.ListSubjects(context.Background())
	require.NoError(t, err)
	assert.Empty(t, subjects)
}
