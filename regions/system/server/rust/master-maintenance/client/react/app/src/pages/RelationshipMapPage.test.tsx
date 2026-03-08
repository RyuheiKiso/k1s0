import { ConfigProvider } from "antd";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { RelationshipMapPage } from "./RelationshipMapPage";

const createRelationshipMock = vi.fn();
const deleteRelationshipMock = vi.fn();

vi.mock("../hooks", () => ({
  useTables: () => ({
    data: {
      tables: [
        {
          id: "table-1",
          name: "departments",
          display_name: "Departments",
          schema_name: "business",
          description: "Department catalog",
          allow_create: true,
          allow_update: true,
          allow_delete: false,
        },
      ],
    },
  }),
  useRelationships: () => ({
    data: [
      {
        id: "rel-1",
        source_column: "parent_id",
        target_table: "departments",
        target_column: "id",
        relationship_type: "many_to_one",
        is_cascade_delete: false,
      },
    ],
  }),
  useCreateRelationship: () => ({
    isPending: false,
    mutateAsync: createRelationshipMock,
  }),
  useUpdateRelationship: () => ({
    isPending: false,
    mutateAsync: vi.fn(),
  }),
  useDeleteRelationship: () => ({
    mutateAsync: deleteRelationshipMock,
  }),
}));

describe("RelationshipMapPage", () => {
  it("renders relationship catalog and opens the create drawer", async () => {
    const user = userEvent.setup();

    render(
      <ConfigProvider>
        <RelationshipMapPage />
      </ConfigProvider>
    );

    expect(screen.getByText("Relationship Surface")).toBeInTheDocument();
    expect(screen.getByText("many_to_one")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: /New relationship/i }));

    expect(screen.getByText("Create relationship")).toBeInTheDocument();
    expect(screen.getByText("Source table")).toBeInTheDocument();
  });

  it("loads an existing relationship into the edit drawer", async () => {
    const user = userEvent.setup();

    render(
      <ConfigProvider>
        <RelationshipMapPage />
      </ConfigProvider>
    );

    await user.click(screen.getByRole("button", { name: "Edit" }));

    expect(screen.getByText("Edit relationship")).toBeInTheDocument();
    expect(screen.getByDisplayValue("parent_id")).toBeInTheDocument();
    expect(screen.getByDisplayValue("id")).toBeInTheDocument();
    expect(screen.getAllByText("Source table is immutable after creation.").length).toBeGreaterThan(0);
    expect(screen.getAllByText("Target table is immutable after creation.").length).toBeGreaterThan(0);
  });
});
