-- CRIT-002 修復: 適用済み migration (002/003/004) の改ざんを元に戻し、
-- 新規 migration でトリガーをべき等 (CREATE OR REPLACE) に再定義する。
-- PostgreSQL 14+ の CREATE OR REPLACE TRIGGER を使用することで、
-- migration を再適用しても "trigger already exists" エラーが発生しない。

CREATE OR REPLACE TRIGGER trigger_workflow_definitions_update_updated_at
    BEFORE UPDATE ON workflow.workflow_definitions
    FOR EACH ROW
    EXECUTE FUNCTION workflow.update_updated_at();

CREATE OR REPLACE TRIGGER trigger_workflow_instances_update_updated_at
    BEFORE UPDATE ON workflow.workflow_instances
    FOR EACH ROW
    EXECUTE FUNCTION workflow.update_updated_at();

CREATE OR REPLACE TRIGGER trigger_workflow_tasks_update_updated_at
    BEFORE UPDATE ON workflow.workflow_tasks
    FOR EACH ROW
    EXECUTE FUNCTION workflow.update_updated_at();
