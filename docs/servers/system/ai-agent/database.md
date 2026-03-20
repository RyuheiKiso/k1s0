# system-ai-agent-server データベース設計

## スキーマ

DB: `k1s0_system`（既存）、スキーマ名: `ai_agent`

```sql
CREATE SCHEMA IF NOT EXISTS ai_agent;
```

---

## テーブル一覧

| テーブル名 | 説明 |
| --- | --- |
| agent_definitions | エージェント定義（モデル・システムプロンプト・ツール） |
| executions | エージェント実行記録 |
| execution_steps | 実行ステップ詳細 |

---

## ER 図

```
agent_definitions 1──* executions 1──* execution_steps
```

---

## テーブル定義

### agent_definitions

```sql
CREATE TABLE ai_agent.agent_definitions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            VARCHAR(256) NOT NULL,
    description     TEXT NOT NULL DEFAULT '',
    model_id        VARCHAR(64) NOT NULL,      -- ai-gateway側のモデルID
    system_prompt   TEXT NOT NULL DEFAULT '',
    tools           JSONB NOT NULL DEFAULT '[]',   -- ツール名のリスト
    max_steps       INTEGER NOT NULL DEFAULT 10,
    enabled         BOOLEAN NOT NULL DEFAULT true,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### executions

```sql
CREATE TABLE ai_agent.executions (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id    UUID NOT NULL REFERENCES ai_agent.agent_definitions(id),
    session_id  VARCHAR(256) NOT NULL DEFAULT '',
    tenant_id   VARCHAR(64) NOT NULL,
    input       TEXT NOT NULL,
    output      TEXT,
    status      VARCHAR(32) NOT NULL DEFAULT 'pending',  -- pending|running|completed|failed|cancelled
    context     JSONB NOT NULL DEFAULT '{}',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_executions_agent_id ON ai_agent.executions(agent_id);
CREATE INDEX idx_executions_tenant_id ON ai_agent.executions(tenant_id);
CREATE INDEX idx_executions_status ON ai_agent.executions(status);
```

### execution_steps

```sql
CREATE TABLE ai_agent.execution_steps (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    execution_id    UUID NOT NULL REFERENCES ai_agent.executions(id),
    step_index      INTEGER NOT NULL,
    step_type       VARCHAR(32) NOT NULL,   -- "thought" | "action" | "observation"
    input           TEXT NOT NULL DEFAULT '',
    output          TEXT NOT NULL DEFAULT '',
    tool_name       VARCHAR(128),
    status          VARCHAR(32) NOT NULL DEFAULT 'completed',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (execution_id, step_index)
);

CREATE INDEX idx_execution_steps_execution_id ON ai_agent.execution_steps(execution_id);
```
