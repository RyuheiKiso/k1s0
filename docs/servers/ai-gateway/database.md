# system-ai-gateway-server データベース設計

## スキーマ

DB: `k1s0_system`（既存）、スキーマ名: `ai_gateway`

```sql
CREATE SCHEMA IF NOT EXISTS ai_gateway;
```

---

## テーブル一覧

| テーブル名 | 説明 |
| --- | --- |
| ai_models | 利用可能なLLMモデル定義 |
| routing_rules | モデルルーティングルール |
| usage_records | テナント別トークン使用量記録 |

---

## ER 図

```
ai_models 1──* routing_rules
ai_models 1──* usage_records
```

---

## テーブル定義

### ai_models

```sql
CREATE TABLE ai_gateway.ai_models (
    id                  VARCHAR(64) PRIMARY KEY,
    name                VARCHAR(256) NOT NULL,
    provider            VARCHAR(64) NOT NULL,        -- "openai", "anthropic", "local"
    context_window      INTEGER NOT NULL DEFAULT 4096,
    cost_per_1k_input   DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    cost_per_1k_output  DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    enabled             BOOLEAN NOT NULL DEFAULT true,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### routing_rules

```sql
CREATE TABLE ai_gateway.routing_rules (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    model_id    VARCHAR(64) NOT NULL REFERENCES ai_gateway.ai_models(id),
    priority    INTEGER NOT NULL DEFAULT 0,
    strategy    VARCHAR(32) NOT NULL DEFAULT 'cost',  -- "cost" | "latency"
    enabled     BOOLEAN NOT NULL DEFAULT true,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### usage_records

```sql
CREATE TABLE ai_gateway.usage_records (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id           VARCHAR(64) NOT NULL,
    model_id            VARCHAR(64) NOT NULL REFERENCES ai_gateway.ai_models(id),
    prompt_tokens       INTEGER NOT NULL DEFAULT 0,
    completion_tokens   INTEGER NOT NULL DEFAULT 0,
    cost_usd            DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_usage_records_tenant_id ON ai_gateway.usage_records(tenant_id);
CREATE INDEX idx_usage_records_created_at ON ai_gateway.usage_records(created_at);
```

---

## 初期データ

```sql
-- OpenAIモデル初期設定
INSERT INTO ai_gateway.ai_models (id, name, provider, context_window, cost_per_1k_input, cost_per_1k_output)
VALUES
    ('gpt-4o-mini', 'GPT-4o Mini', 'openai', 128000, 0.00015, 0.0006),
    ('gpt-4o',      'GPT-4o',      'openai', 128000, 0.005,   0.015),
    ('text-embedding-3-small', 'Text Embedding 3 Small', 'openai', 8191, 0.00002, 0.0);
```
