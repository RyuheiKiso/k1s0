import { PlusOutlined } from "@ant-design/icons";
import {
  Button,
  Card,
  Col,
  Drawer,
  Empty,
  Form,
  Input,
  Popconfirm,
  Row,
  Select,
  Space,
  Switch,
  Table,
  Tag,
  Typography,
  message,
} from "antd";
import { useMemo, useState } from "react";
import { useCreateRelationship, useDeleteRelationship, useRelationships, useTables, useUpdateRelationship } from "../hooks";
import type { TableRelationship } from "../types";

export function RelationshipMapPage() {
  const { data } = useTables();
  const relationshipQuery = useRelationships();
  const createRelationship = useCreateRelationship();
  const deleteRelationship = useDeleteRelationship();
  const updateRelationship = useUpdateRelationship();
  // テーブル一覧をメモ化してレンダーごとの参照変更を防止
  const tables = useMemo(() => data?.tables ?? [], [data?.tables]);
  const [drawerOpen, setDrawerOpen] = useState(false);
  const [editingRelationship, setEditingRelationship] = useState<TableRelationship | null>(null);
  const [form] = Form.useForm();

  const tableOptions = useMemo(
    () => tables.map((table) => ({ label: table.display_name, value: table.name })),
    [tables]
  );

  const submit = async () => {
    try {
      const values = await form.validateFields();
      if (editingRelationship) {
        await updateRelationship.mutateAsync({
          id: editingRelationship.id,
          input: {
            source_column: values.source_column,
            target_column: values.target_column,
            relationship_type: values.relationship_type,
            display_name: values.display_name,
            is_cascade_delete: values.is_cascade_delete,
          },
        });
        message.success("Relationship updated");
      } else {
        await createRelationship.mutateAsync(values);
        message.success("Relationship created");
      }
      form.resetFields();
      setDrawerOpen(false);
      setEditingRelationship(null);
    } catch (error) {
      if (error instanceof Error) {
        message.error(error.message);
      }
    }
  };

  const openCreate = () => {
    setEditingRelationship(null);
    form.resetFields();
    form.setFieldsValue({ relationship_type: "many_to_one", is_cascade_delete: false });
    setDrawerOpen(true);
  };

  const openEdit = (relationship: TableRelationship) => {
    setEditingRelationship(relationship);
    form.setFieldsValue({
      source_column: relationship.source_column,
      target_table: relationship.target_table,
      target_column: relationship.target_column,
      relationship_type: relationship.relationship_type,
      display_name: relationship.display_name ?? undefined,
      is_cascade_delete: relationship.is_cascade_delete,
    });
    setDrawerOpen(true);
  };

  return (
    <>
      <div className="page-stack">
        <Space style={{ justifyContent: "space-between", width: "100%" }}>
          <Typography.Title level={3} style={{ margin: 0 }}>
            Relationship Surface
          </Typography.Title>
          <Button type="primary" icon={<PlusOutlined />} onClick={openCreate}>
            New relationship
          </Button>
        </Space>
        {(relationshipQuery.data ?? []).length === 0 ? (
          <Empty description="No relationships" />
        ) : (
          <Table
            rowKey="id"
            dataSource={relationshipQuery.data ?? []}
            columns={[
              { title: "Source column", dataIndex: "source_column" },
              { title: "Target table", dataIndex: "target_table" },
              { title: "Target column", dataIndex: "target_column" },
              {
                title: "Type",
                dataIndex: "relationship_type",
                render: (value) => <Tag color="blue">{value}</Tag>,
              },
              {
                title: "Cascade",
                dataIndex: "is_cascade_delete",
                render: (value) => (value ? <Tag color="red">cascade</Tag> : <Tag>guarded</Tag>),
              },
              {
                title: "Actions",
                render: (_, record) => (
                  <Space>
                    <Button size="small" onClick={() => openEdit(record)}>
                      Edit
                    </Button>
                    <Popconfirm
                      title="Delete relationship?"
                      onConfirm={async () => {
                        await deleteRelationship.mutateAsync(record.id);
                        message.success("Relationship deleted");
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
        )}
        <Row gutter={[16, 16]}>
          {tables.map((table) => (
            <Col key={table.id} xs={24} md={12} xl={8}>
              <Card className="relationship-card">
                <Typography.Text className="mono-label">{table.schema_name}</Typography.Text>
                <Typography.Title level={4}>{table.display_name}</Typography.Title>
                <Typography.Paragraph type="secondary">
                  {table.description ?? "No relationship narrative yet."}
                </Typography.Paragraph>
                <div className="link-markers">
                  <span>{table.allow_create ? "create" : "read-only"}</span>
                  <span>{table.allow_update ? "mutable" : "stable"}</span>
                  <span>{table.allow_delete ? "destructive" : "guarded"}</span>
                </div>
              </Card>
            </Col>
          ))}
        </Row>
      </div>

      <Drawer
        title={editingRelationship ? "Edit relationship" : "Create relationship"}
        width={420}
        open={drawerOpen}
        onClose={() => {
          setDrawerOpen(false);
          setEditingRelationship(null);
        }}
        extra={
          <Button
            type="primary"
            loading={createRelationship.isPending || updateRelationship.isPending}
            onClick={() => void submit()}
          >
            Save
          </Button>
        }
      >
        <Form
          form={form}
          layout="vertical"
          initialValues={{ relationship_type: "many_to_one", is_cascade_delete: false }}
        >
          <Form.Item
            label="Source table"
            name="source_table"
            rules={[{ required: true }]}
            extra={editingRelationship ? "Source table is immutable after creation." : undefined}
          >
            <Select options={tableOptions} showSearch disabled={Boolean(editingRelationship)} />
          </Form.Item>
          <Form.Item label="Source column" name="source_column" rules={[{ required: true }]}>
            <Input placeholder="department_id" />
          </Form.Item>
          <Form.Item
            label="Target table"
            name="target_table"
            rules={[{ required: true }]}
            extra={editingRelationship ? "Target table is immutable after creation." : undefined}
          >
            <Select options={tableOptions} showSearch disabled={Boolean(editingRelationship)} />
          </Form.Item>
          <Form.Item label="Target column" name="target_column" rules={[{ required: true }]}>
            <Input placeholder="id" />
          </Form.Item>
          <Form.Item label="Relationship type" name="relationship_type" rules={[{ required: true }]}>
            <Select
              options={["one_to_one", "one_to_many", "many_to_one", "many_to_many"].map((value) => ({
                label: value,
                value,
              }))}
            />
          </Form.Item>
          <Form.Item label="Display name" name="display_name">
            <Input placeholder="Department" />
          </Form.Item>
          <Form.Item label="Cascade delete" name="is_cascade_delete" valuePropName="checked">
            <Switch />
          </Form.Item>
        </Form>
      </Drawer>
    </>
  );
}
