import { DownloadOutlined, UploadOutlined } from "@ant-design/icons";
import { Alert, Button, Empty, Select, Space, Typography, Upload, message } from "antd";
import { useMemo, useState } from "react";
import { ShellCard } from "../components/ShellCard";
import { useCsvExport, useImportUpload, useTables } from "../hooks";

export function ImportStudioPage() {
  const { data } = useTables();
  // テーブル一覧をメモ化してレンダーごとの参照変更を防止
  const tables = useMemo(() => data?.tables ?? [], [data?.tables]);
  const [tableName, setTableName] = useState<string | undefined>(tables[0]?.name);
  const upload = useImportUpload(tableName);
  const exportCsv = useCsvExport(tableName);
  // テーブル選択肢のオプション配列を生成
  const options = useMemo(
    () => tables.map((table) => ({ label: table.display_name, value: table.name })),
    [tables]
  );

  const triggerExport = async () => {
    if (!tableName) {
      return;
    }
    const csv = await exportCsv.mutateAsync();
    const blob = new Blob([csv], { type: "text/csv;charset=utf-8" });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement("a");
    anchor.href = url;
    anchor.download = `${tableName}.csv`;
    anchor.click();
    URL.revokeObjectURL(url);
  };

  return (
    <div className="page-stack">
      <ShellCard title="Import / Export Studio">
        <Space direction="vertical" size="large" style={{ width: "100%" }}>
          <Select
            placeholder="Choose a table"
            value={tableName}
            options={options}
            onChange={setTableName}
          />
          {tableName ? (
            <Space wrap>
              <Upload
                showUploadList={false}
                beforeUpload={async (file) => {
                  try {
                    const job = await upload.mutateAsync(file);
                    message.success(`Import job ${job.id} started`);
                  } catch (error) {
                    message.error(error instanceof Error ? error.message : "Import failed");
                  }
                  return false;
                }}
              >
                <Button icon={<UploadOutlined />}>Upload CSV or Excel</Button>
              </Upload>
              <Button icon={<DownloadOutlined />} onClick={() => void triggerExport()}>
                Export CSV
              </Button>
            </Space>
          ) : (
            <Empty description="Select a table first" />
          )}
          <Alert
            type="info"
            showIcon
            message="Accepted formats: CSV, XLSX, XLS, XLSM, XLSB, ODS"
          />
          {upload.data && (
            <Typography.Paragraph>
              Last job: {upload.data.file_name} / {upload.data.status} / processed {upload.data.processed_rows}
            </Typography.Paragraph>
          )}
        </Space>
      </ShellCard>
    </div>
  );
}
