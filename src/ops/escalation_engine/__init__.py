"""src/ops/escalation_engine

self_escalation_engine: Alertmanager webhook → runbook 引当 → 3 階層 escalation DAG → Mattermost page。

仕様: 17_運用ループ適合仕様.md §self_escalation_engine
起動: python -m src.ops.escalation_engine.engine [--port 9093] [--dry-run]
"""
