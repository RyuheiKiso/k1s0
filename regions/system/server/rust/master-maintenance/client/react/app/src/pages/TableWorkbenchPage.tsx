import {
  Alert,
  Button,
  Col,
  DatePicker,
  Descriptions,
  Divider,
  Drawer,
  Empty,
  Form,
  Input,
  InputNumber,
  Popconfirm,
  Row,
  Select,
  Space,
  TableColumnsType,
  Switch,
  Table,
  Tag,
  Timeline,
  Typography,
  message,
} from "antd";
import { PlusOutlined } from "@ant-design/icons";
import { useEffect, useMemo, useState } from "react";
import { ShellCard } from "../components/ShellCard";
import dayjs from "dayjs";
import type { ColumnDefinition } from "../types";
import {
  useColumns,
  useCreateColumns,
  useCreateDisplayConfig,
  useCreateRecord,
  useCreateTable,
  useDeleteDisplayConfig,
  useDeleteRecord,
  useDisplayConfigs,
  useRecords,
  useTableAuditLogs,
  useTables,
  useUpdateDisplayConfig,
  useUpdateRecord,
} from "../hooks";

export function TableWorkbenchPage() {
  const { data, isLoading } = useTables();
  const createTable = useCreateTable();
  const tables = data?.tables ?? [];
  const [selectedTable, setSelectedTable] = useState<string>();
  const [tableDrawerOpen, setTableDrawerOpen] = useState(false);
  const [columnDrawerOpen, setColumnDrawerOpen] = useState(false);
  const [recordDrawerOpen, setRecordDrawerOpen] = useState(false);
  const [configDrawerOpen, setConfigDrawerOpen] = useState(false);
  const [editingRecord, setEditingRecord] = useState<Record<string, unknown> | null>(null);
  const [editingConfigId, setEditingConfigId] = useState<string | null>(null);
  const [tableForm] = Form.useForm();
  const [columnForm] = Form.useForm();
  const [recordForm] = Form.useForm();
  const [configForm] = Form.useForm();

  useEffect(() => {
    if (!selectedTable && tables.length > 0) {
      setSelectedTable(tables[0].name);
    }
  }, [selectedTable, tables]);

  const activeTable = useMemo(
    () => tables.find((table) => table.name === selectedTable) ?? tables[0],
    [selectedTable, tables]
  );
  const columnsQuery = useColumns(activeTable?.name);
  const createColumns = useCreateColumns(activeTable?.name);
  const recordsQuery = useRecords(activeTable?.name);
  const createRecord = useCreateRecord(activeTable?.name);
  const updateRecord = useUpdateRecord(activeTable?.name);
  const deleteRecord = useDeleteRecord(activeTable?.name);
  const auditLogs = useTableAuditLogs(activeTable?.name);
  const displayConfigs = useDisplayConfigs(activeTable?.name);
  const createDisplayConfig = useCreateDisplayConfig(activeTable?.name);
  const updateDisplayConfig = useUpdateDisplayConfig(activeTable?.name);
  const deleteDisplayConfig = useDeleteDisplayConfig(activeTable?.name);
  const editableColumns = (columnsQuery.data ?? []).filter((column) => !column.is_readonly);
  const primaryKeyColumn = (columnsQuery.data ?? []).find((column) => column.is_primary_key);

  const submitTable = async () => {
    try {
      const values = await tableForm.validateFields();
      await createTable.mutateAsync(values);
      message.success(`Table ${values.display_name} created`);
      tableForm.resetFields();
      setTableDrawerOpen(false);
    } catch (error) {
      if (error instanceof Error) {
        message.error(error.message);
      }
    }
  };

  const submitColumn = async () => {
    try {
      const values = await columnForm.validateFields();
      await createColumns.mutateAsync([
        {
          ...values,
          display_order: values.display_order ?? (columnsQuery.data?.length ?? 0),
          is_sortable: values.is_sortable ?? true,
          is_nullable: values.is_nullable ?? true,
          is_visible_in_list: values.is_visible_in_list ?? true,
          is_visible_in_form: values.is_visible_in_form ?? true,
        },
      ]);
      message.success(`Column ${values.display_name} created`);
      columnForm.resetFields();
      setColumnDrawerOpen(false);
    } catch (error) {
      if (error instanceof Error) {
        message.error(error.message);
      }
    }
  };

  const openCreateRecord = () => {
    recordForm.resetFields();
    setEditingRecord(null);
    setRecordDrawerOpen(true);
  };

  const openEditRecord = (record: Record<string, unknown>) => {
    setEditingRecord(record);
    recordForm.setFieldsValue(toFormValues(record, editableColumns));
    setRecordDrawerOpen(true);
  };

  const submitRecord = async () => {
    try {
      const values = await recordForm.validateFields();
      const payload = fromFormValues(values, editableColumns);
      if (editingRecord && primaryKeyColumn) {
        const recordId = String(editingRecord[primaryKeyColumn.column_name] ?? "");
        await updateRecord.mutateAsync({ recordId, payload });
        message.success(`Record ${recordId} updated`);
      } else {
        await createRecord.mutateAsync(payload);
        message.success("Record created");
      }
      recordForm.resetFields();
      setEditingRecord(null);
      setRecordDrawerOpen(false);
    } catch (error) {
      if (error instanceof Error) {
        message.error(error.message);
      }
    }
  };

  const removeRecord = async (record: Record<string, unknown>) => {
    if (!primaryKeyColumn) {
      message.error("Primary key column is required for delete");
      return;
    }
    const recordId = String(record[primaryKeyColumn.column_name] ?? "");
    await deleteRecord.mutateAsync(recordId);
    message.success(`Record ${recordId} deleted`);
  };

  const openCreateConfig = () => {
    setEditingConfigId(null);
    configForm.setFieldsValue({
      config_type: "list_view",
      config_json: JSON.stringify(
        {
          columns: (columnsQuery.data ?? [])
            .filter((column) => column.is_visible_in_list)
            .map((column) => ({ column_name: column.column_name })),
        },
        null,
        2
      ),
      is_default: false,
    });
    setConfigDrawerOpen(true);
  };

  const openEditConfig = (config: { id: string; config_type: string; config_json: unknown; is_default: boolean }) => {
    setEditingConfigId(config.id);
    configForm.setFieldsValue({
      config_type: config.config_type,
      config_json: JSON.stringify(config.config_json, null, 2),
      is_default: config.is_default,
    });
    setConfigDrawerOpen(true);
  };

  const submitConfig = async () => {
    try {
      const values = await configForm.validateFields();
      const input = {
        config_type: values.config_type,
        config_json: JSON.parse(values.config_json),
        is_default: values.is_default,
      };
      if (editingConfigId) {
        await updateDisplayConfig.mutateAsync({ id: editingConfigId, input });
        message.success("Display config updated");
      } else {
        await createDisplayConfig.mutateAsync(input);
        message.success("Display config created");
      }
      setConfigDrawerOpen(false);
      setEditingConfigId(null);
    } catch (error) {
      if (error instanceof Error) {
        message.error(error.message);
      }
    }
  };

  const recordColumns: TableColumnsType<Record<string, unknown>> = [
    ...(columnsQuery.data ?? [])
      .filter((column) => column.is_visible_in_list)
      .map((column) => ({
        title: column.display_name,
        dataIndex: column.column_name,
        render: (value: unknown) =>
          typeof value === "object" && value !== null ? (
            <pre className="json-inline">{JSON.stringify(value, null, 2)}</pre>
          ) : (
            String(value ?? "")
          ),
      })),
    {
      title: "Actions",
      fixed: "right",
      render: (_, record) => (
        <Space>
          <Button size="small" onClick={() => openEditRecord(record)}>
            Edit
          </Button>
          <Popconfirm title="Delete record?" onConfirm={() => removeRecord(record)}>
            <Button size="small" danger>
              Delete
            </Button>
          </Popconfirm>
        </Space>
      ),
    },
  ];

  return (
    <>
      <Row gutter={[16, 16]}>
        <Col xs={24} xl={8}>
          <ShellCard
            title="Tables"
            extra={
              <Button type="primary" icon={<PlusOutlined />} onClick={() => setTableDrawerOpen(true)}>
                New table
              </Button>
            }
          >
            <Table
              rowKey="id"
              loading={isLoading}
              pagination={false}
              dataSource={tables}
              onRow={(record) => ({
                onClick: () => setSelectedTable(record.name),
              })}
              columns={[
                { title: "Display", dataIndex: "display_name" },
                {
                  title: "Schema",
                  dataIndex: "schema_name",
                  render: (value) => <span className="mono-chip">{value}</span>,
                },
                {
                  title: "State",
                  render: (_, record) => (
                    <Space size={4}>
                      <Tag color={record.is_active ? "green" : "default"}>
                        {record.is_active ? "active" : "paused"}
                      </Tag>
                      {record.allow_delete && <Tag color="red">delete</Tag>}
                    </Space>
                  ),
                },
              ]}
            />
          </ShellCard>
        </Col>
        <Col xs={24} xl={16}>
          {activeTable ? (
            <div className="page-stack">
              <ShellCard
                title={activeTable.display_name}
                extra={<span className="mono-chip">{activeTable.name}</span>}
              >
                <Typography.Paragraph>{activeTable.description ?? "No description"}</Typography.Paragraph>
                <Alert
                  type="info"
                  showIcon
                  message={`create=${activeTable.allow_create} update=${activeTable.allow_update} delete=${activeTable.allow_delete}`}
                />
              </ShellCard>
              <ShellCard
                title="Columns"
                extra={
                  <Button icon={<PlusOutlined />} onClick={() => setColumnDrawerOpen(true)}>
                    Add column
                  </Button>
                }
              >
                <Table
                  rowKey="id"
                  loading={columnsQuery.isLoading}
                  pagination={false}
                  dataSource={columnsQuery.data ?? []}
                  columns={[
                    {
                      title: "Column",
                      dataIndex: "column_name",
                      render: (value) => <span className="mono-chip">{value}</span>,
                    },
                    { title: "Display", dataIndex: "display_name" },
                    { title: "Type", dataIndex: "data_type" },
                    { title: "Input", dataIndex: "input_type" },
                  ]}
                />
              </ShellCard>
              <ShellCard title="Sample Records">
                <Space style={{ marginBottom: 16 }}>
                  <Button
                    type="primary"
                    icon={<PlusOutlined />}
                    disabled={!activeTable?.allow_create}
                    onClick={openCreateRecord}
                  >
                    New record
                  </Button>
                </Space>
                <Table
                  rowKey={(record) => getRecordRowKey(record, primaryKeyColumn)}
                  loading={recordsQuery.isLoading}
                  dataSource={recordsQuery.data?.records ?? []}
                  locale={{ emptyText: <Empty description="No records" /> }}
                  scroll={{ x: 960 }}
                  columns={recordColumns}
                />
              </ShellCard>
              <ShellCard title="Audit Trail">
                {(auditLogs.data?.logs ?? []).length > 0 ? (
                  <Timeline
                    items={(auditLogs.data?.logs ?? []).map((log) => ({
                      color:
                        log.operation === "DELETE"
                          ? "red"
                          : log.operation === "UPDATE"
                            ? "blue"
                            : "green",
                      children: (
                        <div>
                          <Space wrap>
                            <strong>{log.operation}</strong>
                            <span className="mono-chip">{log.target_record_id}</span>
                            <span>{log.changed_by}</span>
                          </Space>
                          <div className="audit-meta">
                            <span>{new Date(log.created_at).toLocaleString()}</span>
                            {log.changed_columns?.length ? (
                              <span>changed: {log.changed_columns.join(", ")}</span>
                            ) : null}
                          </div>
                        </div>
                      ),
                    }))}
                  />
                ) : (
                  <Empty description="No audit logs yet" />
                )}
              </ShellCard>
              <ShellCard
                title="Display Configs"
                extra={
                  <Button icon={<PlusOutlined />} onClick={openCreateConfig}>
                    New config
                  </Button>
                }
              >
                <Table
                  rowKey="id"
                  pagination={false}
                  dataSource={displayConfigs.data ?? []}
                  locale={{ emptyText: <Empty description="No display configs yet" /> }}
                  columns={[
                    { title: "Type", dataIndex: "config_type" },
                    {
                      title: "Default",
                      dataIndex: "is_default",
                      render: (value) => (value ? <Tag color="green">default</Tag> : <Tag>secondary</Tag>),
                    },
                    { title: "Updated", dataIndex: "updated_at", render: (value) => new Date(value).toLocaleString() },
                    {
                      title: "Actions",
                      render: (_, record) => (
                        <Space>
                          <Button size="small" onClick={() => openEditConfig(record)}>
                            Edit
                          </Button>
                          <Popconfirm
                            title="Delete display config?"
                            onConfirm={async () => {
                              await deleteDisplayConfig.mutateAsync(record.id);
                              message.success("Display config deleted");
                            }}
                          >
                            <Button size="small" danger>
                              Delete
                            </Button>
                          </Popconfirm>
                        </Space>
                      ),
                    },
                  ]}
                />
              </ShellCard>
            </div>
          ) : (
            <ShellCard title="Tables">
              <Empty description="No table definitions yet" />
            </ShellCard>
          )}
        </Col>
      </Row>

      <Drawer
        title="Create table"
        width={420}
        open={tableDrawerOpen}
        onClose={() => setTableDrawerOpen(false)}
        extra={
          <Button type="primary" loading={createTable.isPending} onClick={() => void submitTable()}>
            Save
          </Button>
        }
      >
        <Form
          form={tableForm}
          layout="vertical"
          initialValues={{
            schema_name: "business",
            allow_create: true,
            allow_update: true,
            allow_delete: false,
            sort_order: 0,
          }}
        >
          <Form.Item label="Internal name" name="name" rules={[{ required: true }]}>
            <Input placeholder="departments" />
          </Form.Item>
          <Form.Item label="Display name" name="display_name" rules={[{ required: true }]}>
            <Input placeholder="Departments" />
          </Form.Item>
          <Form.Item label="Schema" name="schema_name" rules={[{ required: true }]}>
            <Input />
          </Form.Item>
          <Form.Item label="Database name" name="database_name">
            <Input placeholder="default" />
          </Form.Item>
          <Form.Item label="Category" name="category">
            <Input placeholder="organization" />
          </Form.Item>
          <Form.Item label="Description" name="description">
            <Input.TextArea rows={4} />
          </Form.Item>
          <Form.Item label="Sort order" name="sort_order">
            <InputNumber min={0} style={{ width: "100%" }} />
          </Form.Item>
          <Form.Item label="Create" name="allow_create" valuePropName="checked">
            <Switch />
          </Form.Item>
          <Form.Item label="Update" name="allow_update" valuePropName="checked">
            <Switch />
          </Form.Item>
          <Form.Item label="Delete" name="allow_delete" valuePropName="checked">
            <Switch />
          </Form.Item>
        </Form>
      </Drawer>

      <Drawer
        title={`Add column${activeTable ? ` to ${activeTable.display_name}` : ""}`}
        width={420}
        open={columnDrawerOpen}
        onClose={() => setColumnDrawerOpen(false)}
        extra={
          <Button
            type="primary"
            disabled={!activeTable}
            loading={createColumns.isPending}
            onClick={() => void submitColumn()}
          >
            Save
          </Button>
        }
      >
        <Form
          form={columnForm}
          layout="vertical"
          initialValues={{
            data_type: "text",
            input_type: "text",
            is_nullable: true,
            is_sortable: true,
            is_visible_in_list: true,
            is_visible_in_form: true,
          }}
        >
          <Form.Item label="Column name" name="column_name" rules={[{ required: true }]}>
            <Input placeholder="code" />
          </Form.Item>
          <Form.Item label="Display name" name="display_name" rules={[{ required: true }]}>
            <Input placeholder="Code" />
          </Form.Item>
          <Form.Item label="Data type" name="data_type" rules={[{ required: true }]}>
            <Select
              options={[
                "text",
                "integer",
                "decimal",
                "boolean",
                "date",
                "datetime",
                "uuid",
                "jsonb",
              ].map((value) => ({ label: value, value }))}
            />
          </Form.Item>
          <Form.Item label="Input type" name="input_type" rules={[{ required: true }]}>
            <Select
              options={[
                "text",
                "textarea",
                "select",
                "checkbox",
                "date",
                "number",
                "json_editor",
              ].map((value) => ({ label: value, value }))}
            />
          </Form.Item>
          <Form.Item label="Default value" name="default_value">
            <Input />
          </Form.Item>
          <Form.Item label="Display order" name="display_order">
            <InputNumber min={0} style={{ width: "100%" }} />
          </Form.Item>
          <Form.Item label="Primary key" name="is_primary_key" valuePropName="checked">
            <Switch />
          </Form.Item>
          <Form.Item label="Nullable" name="is_nullable" valuePropName="checked">
            <Switch />
          </Form.Item>
          <Form.Item label="Unique" name="is_unique" valuePropName="checked">
            <Switch />
          </Form.Item>
          <Form.Item label="Searchable" name="is_searchable" valuePropName="checked">
            <Switch />
          </Form.Item>
          <Form.Item label="Visible in list" name="is_visible_in_list" valuePropName="checked">
            <Switch />
          </Form.Item>
          <Form.Item label="Visible in form" name="is_visible_in_form" valuePropName="checked">
            <Switch />
          </Form.Item>
        </Form>
      </Drawer>

      <Drawer
        title={
          editingRecord
            ? `Edit record${primaryKeyColumn ? ` ${String(editingRecord?.[primaryKeyColumn.column_name] ?? "")}` : ""}`
            : `Create record${activeTable ? ` in ${activeTable.display_name}` : ""}`
        }
        width={460}
        open={recordDrawerOpen}
        onClose={() => {
          setRecordDrawerOpen(false);
          setEditingRecord(null);
        }}
        extra={
          <Button
            type="primary"
            loading={createRecord.isPending || updateRecord.isPending}
            onClick={() => void submitRecord()}
          >
            Save
          </Button>
        }
      >
        {editingRecord && primaryKeyColumn ? (
          <>
            <Descriptions
              size="small"
              column={1}
              items={[
                {
                  key: "pk",
                  label: "Primary key",
                  children: String(editingRecord[primaryKeyColumn.column_name] ?? ""),
                },
              ]}
            />
            <Divider />
          </>
        ) : null}
        <Form form={recordForm} layout="vertical">
          {editableColumns
            .filter((column) => !column.is_primary_key || !editingRecord)
            .filter((column) => column.is_visible_in_form)
            .map((column) => (
              <Form.Item
                key={column.id}
                label={column.display_name}
                name={column.column_name}
                valuePropName={column.input_type === "checkbox" ? "checked" : "value"}
                rules={[{ required: !column.is_nullable && !column.is_primary_key }]}
              >
                {renderField(column)}
              </Form.Item>
            ))}
        </Form>
      </Drawer>

      <Drawer
        title={editingConfigId ? "Edit display config" : "Create display config"}
        width={460}
        open={configDrawerOpen}
        onClose={() => {
          setConfigDrawerOpen(false);
          setEditingConfigId(null);
        }}
        extra={
          <Button
            type="primary"
            loading={createDisplayConfig.isPending || updateDisplayConfig.isPending}
            onClick={() => void submitConfig()}
          >
            Save
          </Button>
        }
      >
        <Form form={configForm} layout="vertical">
          <Form.Item label="Config type" name="config_type" rules={[{ required: true }]}>
            <Select
              options={["list_view", "form_view", "detail_view"].map((value) => ({
                label: value,
                value,
              }))}
            />
          </Form.Item>
          <Form.Item label="Default" name="is_default" valuePropName="checked">
            <Switch />
          </Form.Item>
          <Form.Item label="Config JSON" name="config_json" rules={[{ required: true }]}>
            <Input.TextArea rows={14} />
          </Form.Item>
        </Form>
      </Drawer>
    </>
  );
}

function renderField(column: ColumnDefinition) {
  switch (column.input_type) {
    case "textarea":
      return <Input.TextArea rows={4} />;
    case "number":
      return <InputNumber style={{ width: "100%" }} />;
    case "checkbox":
      return <Switch />;
    case "date":
      return <DatePicker style={{ width: "100%" }} />;
    case "select":
      return (
        <Select
          options={Array.isArray(column.select_options) ? column.select_options as Array<{ label: string; value: string }> : []}
        />
      );
    case "json_editor":
      return <Input.TextArea rows={6} placeholder='{"key":"value"}' />;
    default:
      return <Input />;
  }
}

function toFormValues(record: Record<string, unknown>, columns: ColumnDefinition[]) {
  const next: Record<string, unknown> = {};
  for (const column of columns) {
    const value = record[column.column_name];
    if ((column.input_type === "date" || column.data_type === "date" || column.data_type === "datetime") && typeof value === "string" && value) {
      next[column.column_name] = dayjs(value);
    } else if (column.input_type === "checkbox" && typeof value === "boolean") {
      next[column.column_name] = value;
    } else if (column.input_type === "json_editor" && value && typeof value === "object") {
      next[column.column_name] = JSON.stringify(value, null, 2);
    } else {
      next[column.column_name] = value;
    }
  }
  return next;
}

function fromFormValues(values: Record<string, unknown>, columns: ColumnDefinition[]) {
  const payload: Record<string, unknown> = {};
  for (const column of columns) {
    const value = values[column.column_name];
    if (value === undefined) {
      continue;
    }
    if (dayjs.isDayjs(value)) {
      payload[column.column_name] = value.toISOString();
    } else if (column.input_type === "json_editor" && typeof value === "string" && value.trim() !== "") {
      payload[column.column_name] = JSON.parse(value);
    } else {
      payload[column.column_name] = value;
    }
  }
  return payload;
}

function getRecordRowKey(record: Record<string, unknown>, primaryKeyColumn?: ColumnDefinition) {
  if (primaryKeyColumn) {
    const primaryValue = record[primaryKeyColumn.column_name];
    if (primaryValue !== undefined && primaryValue !== null && primaryValue !== "") {
      return String(primaryValue);
    }
  }

  const firstScalarEntry = Object.entries(record).find(([, value]) =>
    ["string", "number", "boolean"].includes(typeof value)
  );
  if (firstScalarEntry) {
    return `${firstScalarEntry[0]}:${String(firstScalarEntry[1])}`;
  }

  return JSON.stringify(record);
}
