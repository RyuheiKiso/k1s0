import { PlusOutlined } from "@ant-design/icons";
import {
  Alert,
  Button,
  Drawer,
  Empty,
  Form,
  FormListFieldData,
  Input,
  List,
  Space,
  Select,
  Table,
  Tag,
  message,
} from "antd";
import { useEffect, useState } from "react";
import { ShellCard } from "../components/ShellCard";
import { useColumns, useCreateRule, useRuleCheck, useRules, useTables } from "../hooks";

export function RuleConsolePage() {
  const { data } = useTables();
  const tables = data?.tables ?? [];
  const [tableName, setTableName] = useState<string>();
  const [drawerOpen, setDrawerOpen] = useState(false);
  const [form] = Form.useForm();
  const rules = useRules(tableName);
  const result = useRuleCheck(tableName);
  const createRule = useCreateRule();
  const columns = useColumns(tableName);

  useEffect(() => {
    if (!tableName && tables.length > 0) {
      setTableName(tables[0].name);
    }
  }, [tableName, tables]);

  const submitRule = async () => {
    try {
      const values = await form.validateFields();
      await createRule.mutateAsync({
        name: values.name,
        description: values.description,
        rule_type: values.rule_type,
        severity: values.severity,
        source_table: tableName!,
        evaluation_timing: values.evaluation_timing,
        error_message_template: values.error_message_template,
        conditions: (values.conditions ?? []).map(
          (
            condition: {
              left_column: string;
              operator: string;
              right_table?: string;
              right_column?: string;
              right_value?: string;
              logical_connector?: string;
            },
            index: number
          ) => ({
            condition_order: index + 1,
            left_column: condition.left_column,
            operator: condition.operator,
            right_table: condition.right_table,
            right_column: condition.right_column,
            right_value: condition.right_value,
            logical_connector: condition.logical_connector ?? "AND",
          })
        ),
      });
      message.success(`Rule ${values.name} created`);
      form.resetFields();
      setDrawerOpen(false);
    } catch (error) {
      if (error instanceof Error) {
        message.error(error.message);
      }
    }
  };

  return (
    <>
      <div className="page-stack">
        <ShellCard title="On-demand Consistency">
          <div className="selector-row">
            {tables.map((table) => (
              <button
                key={table.id}
                className={table.name === tableName ? "pill pill-active" : "pill"}
                onClick={() => setTableName(table.name)}
              >
                {table.display_name}
              </button>
            ))}
          </div>
        </ShellCard>
        <ShellCard
          title="Rule Catalog"
          extra={
            <Button type="primary" icon={<PlusOutlined />} disabled={!tableName} onClick={() => setDrawerOpen(true)}>
              New rule
            </Button>
          }
        >
          <Table
            rowKey="id"
            loading={rules.isLoading}
            pagination={false}
            dataSource={rules.data ?? []}
            locale={{ emptyText: <Empty description="No rules" /> }}
            columns={[
              { title: "Name", dataIndex: "name" },
              { title: "Type", dataIndex: "rule_type" },
              { title: "Timing", dataIndex: "evaluation_timing" },
              {
                title: "Severity",
                dataIndex: "severity",
                render: (value) => <Tag color={value === "warning" ? "orange" : "red"}>{value}</Tag>,
              },
            ]}
          />
        </ShellCard>
        <ShellCard title="Check Results">
          {result.data?.results?.length ? (
            <List
              dataSource={result.data.results}
              renderItem={(item) => (
                <List.Item>
                  <List.Item.Meta
                    title={
                      <span>
                        {item.rule_name}{" "}
                        <Tag color={item.passed ? "green" : item.severity === "warning" ? "orange" : "red"}>
                          {item.severity}
                        </Tag>
                      </span>
                    }
                    description={item.message ?? "passed"}
                  />
                </List.Item>
              )}
            />
          ) : tableName ? (
            <Alert type="success" showIcon message="No consistency violations returned." />
          ) : (
            <Empty description="Pick a table" />
          )}
        </ShellCard>
      </div>

      <Drawer
        title={`Create rule${tableName ? ` for ${tableName}` : ""}`}
        width={420}
        open={drawerOpen}
        onClose={() => setDrawerOpen(false)}
        extra={
          <Button type="primary" loading={createRule.isPending} onClick={() => void submitRule()}>
            Save
          </Button>
        }
      >
        <Form
          form={form}
          layout="vertical"
          initialValues={{
            rule_type: "range",
            severity: "error",
            evaluation_timing: "before_save",
            conditions: [
              {
                operator: "gte",
                logical_connector: "AND",
              },
            ],
          }}
        >
          <Form.Item label="Rule name" name="name" rules={[{ required: true }]}>
            <Input placeholder="price_floor" />
          </Form.Item>
          <Form.Item label="Description" name="description">
            <Input.TextArea rows={3} />
          </Form.Item>
          <Form.Item label="Rule type" name="rule_type" rules={[{ required: true }]}>
            <Select
              options={["range", "conditional", "uniqueness", "cross_table"].map((value) => ({
                label: value,
                value,
              }))}
            />
          </Form.Item>
          <Form.Item label="Severity" name="severity" rules={[{ required: true }]}>
            <Select options={["error", "warning", "info"].map((value) => ({ label: value, value }))} />
          </Form.Item>
          <Form.Item label="Timing" name="evaluation_timing" rules={[{ required: true }]}>
            <Select
              options={["before_save", "after_save", "on_demand", "scheduled"].map((value) => ({
                label: value,
                value,
              }))}
            />
          </Form.Item>
          <Form.List name="conditions">
            {(fields, { add, remove }) => (
              <div className="condition-stack">
                {fields.map((field, index) => (
                  <ConditionEditor
                    key={field.key}
                    field={field}
                    index={index}
                    canRemove={fields.length > 1}
                    onRemove={() => remove(field.name)}
                    tableOptions={tables.map((table) => ({ label: table.display_name, value: table.name }))}
                    columnOptions={(columns.data ?? []).map((column) => ({
                      label: column.display_name,
                      value: column.column_name,
                    }))}
                  />
                ))}
                <Button
                  type="dashed"
                  icon={<PlusOutlined />}
                  onClick={() => add({ operator: "eq", logical_connector: "AND" })}
                >
                  Add condition
                </Button>
              </div>
            )}
          </Form.List>
          <Form.Item label="Error message" name="error_message_template" rules={[{ required: true }]}>
            <Input placeholder="Amount must be >= 0" />
          </Form.Item>
        </Form>
      </Drawer>
    </>
  );
}

function ConditionEditor({
  field,
  index,
  canRemove,
  onRemove,
  tableOptions,
  columnOptions,
}: {
  field: FormListFieldData;
  index: number;
  canRemove: boolean;
  onRemove: () => void;
  tableOptions: Array<{ label: string; value: string }>;
  columnOptions: Array<{ label: string; value: string }>;
}) {
  return (
    <div className="condition-card">
      <Space style={{ width: "100%", justifyContent: "space-between" }}>
        <strong>Condition {index + 1}</strong>
        {canRemove ? (
          <Button size="small" danger onClick={onRemove}>
            Remove
          </Button>
        ) : null}
      </Space>
      {index > 0 ? (
        <Form.Item
          label="Connector"
          name={[field.name, "logical_connector"]}
          rules={[{ required: true }]}
        >
          <Select options={["AND", "OR"].map((value) => ({ label: value, value }))} />
        </Form.Item>
      ) : null}
      <Form.Item
        label="Left column"
        name={[field.name, "left_column"]}
        rules={[{ required: true }]}
      >
        <Select options={columnOptions} showSearch />
      </Form.Item>
      <Form.Item
        label="Operator"
        name={[field.name, "operator"]}
        rules={[{ required: true }]}
      >
        <Select
          options={["eq", "neq", "gt", "gte", "lt", "lte", "regex", "in", "not_in", "between", "exists", "not_exists"].map(
            (value) => ({ label: value, value })
          )}
        />
      </Form.Item>
      <Form.Item label="Right table" name={[field.name, "right_table"]}>
        <Select allowClear options={tableOptions} showSearch />
      </Form.Item>
      <Form.Item label="Right column" name={[field.name, "right_column"]}>
        <Select allowClear options={columnOptions} showSearch />
      </Form.Item>
      <Form.Item label="Right value" name={[field.name, "right_value"]}>
        <Input placeholder='0, ["A","B"], {current.code}' />
      </Form.Item>
    </div>
  );
}
