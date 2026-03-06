import { ConfigProvider } from "antd";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { TableWorkbenchPage } from "./TableWorkbenchPage";

const createTableMock = vi.fn();
const createColumnsMock = vi.fn();
const createRecordMock = vi.fn();
const updateRecordMock = vi.fn();
const deleteRecordMock = vi.fn();
const createDisplayConfigMock = vi.fn();
const updateDisplayConfigMock = vi.fn();
const deleteDisplayConfigMock = vi.fn();

vi.mock("../hooks", () => ({
  useTables: () => ({
    data: {
      tables: [
        {
          id: "table-1",
          name: "departments",
          display_name: "Departments",
          description: "Department catalog",
          schema_name: "business",
          allow_create: true,
          allow_update: true,
          allow_delete: false,
          is_active: true,
        },
      ],
    },
    isLoading: false,
  }),
  useColumns: () => ({
    isLoading: false,
    data: [
      {
        id: "column-1",
        column_name: "id",
        display_name: "ID",
        data_type: "uuid",
        input_type: "text",
        is_primary_key: true,
        is_nullable: false,
        is_unique: true,
        is_visible_in_list: true,
        is_visible_in_form: false,
        is_readonly: true,
      },
      {
        id: "column-2",
        column_name: "name",
        display_name: "Name",
        data_type: "text",
        input_type: "text",
        is_primary_key: false,
        is_nullable: false,
        is_unique: false,
        is_visible_in_list: true,
        is_visible_in_form: true,
        is_readonly: false,
      },
    ],
  }),
  useCreateColumns: () => ({ isPending: false, mutateAsync: createColumnsMock }),
  useRecords: () => ({
    isLoading: false,
    data: {
      records: [{ id: "dept-1", name: "Platform" }],
      total: 1,
    },
  }),
  useCreateRecord: () => ({ isPending: false, mutateAsync: createRecordMock }),
  useUpdateRecord: () => ({ isPending: false, mutateAsync: updateRecordMock }),
  useDeleteRecord: () => ({ mutateAsync: deleteRecordMock }),
  useTableAuditLogs: () => ({
    data: {
      logs: [
        {
          id: "log-1",
          operation: "INSERT",
          target_record_id: "dept-1",
          changed_by: "tester",
          created_at: "2026-03-06T00:00:00Z",
        },
      ],
    },
  }),
  useDisplayConfigs: () => ({
    data: [
      {
        id: "cfg-1",
        config_type: "list_view",
        is_default: true,
        updated_at: "2026-03-06T00:00:00Z",
        config_json: { columns: [{ column_name: "name" }] },
      },
    ],
  }),
  useCreateDisplayConfig: () => ({ isPending: false, mutateAsync: createDisplayConfigMock }),
  useUpdateDisplayConfig: () => ({ isPending: false, mutateAsync: updateDisplayConfigMock }),
  useDeleteDisplayConfig: () => ({ mutateAsync: deleteDisplayConfigMock }),
  useCreateTable: () => ({ isPending: false, mutateAsync: createTableMock }),
}));

describe("TableWorkbenchPage", () => {
  it("renders records, audit logs, and display configs", async () => {
    const user = userEvent.setup();

    render(
      <ConfigProvider>
        <TableWorkbenchPage />
      </ConfigProvider>
    );

    expect(screen.getAllByText("Departments").length).toBeGreaterThan(0);
    expect(screen.getByText("Platform")).toBeInTheDocument();
    expect(screen.getByText("Audit Trail")).toBeInTheDocument();
    expect(screen.getByText("Display Configs")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: /New config/i }));

    expect(screen.getByText("Create display config")).toBeInTheDocument();
  });

  it("loads an existing display config into the edit drawer", async () => {
    const user = userEvent.setup();

    render(
      <ConfigProvider>
        <TableWorkbenchPage />
      </ConfigProvider>
    );

    const editButtons = screen.getAllByRole("button", { name: "Edit" });
    await user.click(editButtons[editButtons.length - 1]);

    expect(screen.getByText("Edit display config")).toBeInTheDocument();
    expect(screen.getByDisplayValue(/"column_name": "name"/)).toBeInTheDocument();
  });
});
