import { ConfigProvider } from "antd";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ImportStudioPage } from "./ImportStudioPage";

const exportCsvMock = vi.fn();

vi.mock("../hooks", () => ({
  useTables: () => ({
    data: {
      tables: [
        {
          id: "table-1",
          name: "departments",
          display_name: "Departments",
        },
      ],
    },
  }),
  useImportUpload: () => ({
    data: {
      id: "job-1",
      file_name: "departments.csv",
      status: "completed",
      processed_rows: 2,
    },
    mutateAsync: vi.fn(),
  }),
  useCsvExport: () => ({
    mutateAsync: exportCsvMock,
  }),
}));

describe("ImportStudioPage", () => {
  it("renders import controls and the latest job summary", () => {
    render(
      <ConfigProvider>
        <ImportStudioPage />
      </ConfigProvider>
    );

    expect(screen.getByText("Import / Export Studio")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Upload CSV or Excel/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Export CSV/i })).toBeInTheDocument();
    expect(screen.getByText(/Last job: departments\.csv \/ completed \/ processed 2/i)).toBeInTheDocument();
  });

  it("exports CSV for the selected table", async () => {
    const user = userEvent.setup();
    exportCsvMock.mockResolvedValueOnce("id,name\n1,Platform\n");

    const createObjectUrlSpy = vi.fn(() => "blob:master-maintenance");
    const revokeObjectUrlSpy = vi.fn();
    Object.defineProperty(URL, "createObjectURL", {
      writable: true,
      value: createObjectUrlSpy,
    });
    Object.defineProperty(URL, "revokeObjectURL", {
      writable: true,
      value: revokeObjectUrlSpy,
    });
    const clickSpy = vi.spyOn(HTMLAnchorElement.prototype, "click").mockImplementation(() => {});

    render(
      <ConfigProvider>
        <ImportStudioPage />
      </ConfigProvider>
    );

    await user.click(screen.getByRole("button", { name: /Export CSV/i }));

    expect(exportCsvMock).toHaveBeenCalledTimes(1);
    expect(createObjectUrlSpy).toHaveBeenCalledTimes(1);
    expect(clickSpy).toHaveBeenCalledTimes(1);
    expect(revokeObjectUrlSpy).toHaveBeenCalledWith("blob:master-maintenance");

    clickSpy.mockRestore();
  });
});
