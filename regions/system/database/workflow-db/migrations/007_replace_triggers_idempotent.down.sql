-- CRIT-002 修復のロールバック: べき等トリガーを元の通常トリガーに戻す
-- （トリガーの定義自体は同一のため、実質的には DROP + CREATE のみ）

DROP TRIGGER IF EXISTS trigger_workflow_definitions_update_updated_at ON workflow.workflow_definitions;
CREATE TRIGGER trigger_workflow_definitions_update_updated_at
    BEFORE UPDATE ON workflow.workflow_definitions
    FOR EACH ROW
    EXECUTE FUNCTION workflow.update_updated_at();

DROP TRIGGER IF EXISTS trigger_workflow_instances_update_updated_at ON workflow.workflow_instances;
CREATE TRIGGER trigger_workflow_instances_update_updated_at
    BEFORE UPDATE ON workflow.workflow_instances
    FOR EACH ROW
    EXECUTE FUNCTION workflow.update_updated_at();

DROP TRIGGER IF EXISTS trigger_workflow_tasks_update_updated_at ON workflow.workflow_tasks;
CREATE TRIGGER trigger_workflow_tasks_update_updated_at
    BEFORE UPDATE ON workflow.workflow_tasks
    FOR EACH ROW
    EXECUTE FUNCTION workflow.update_updated_at();
