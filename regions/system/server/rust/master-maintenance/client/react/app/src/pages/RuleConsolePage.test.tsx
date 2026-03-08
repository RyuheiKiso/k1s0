import { ConfigProvider } from "antd";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { RuleConsolePage } from "./RuleConsolePage";

const createRuleMock = vi.fn();

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
  useRules: () => ({
    isLoading: false,
    data: [
      {
        id: "rule-1",
        name: "department_name_required",
        rule_type: "conditional",
        evaluation_timing: "before_save",
        severity: "error",
      },
    ],
  }),
  useRuleCheck: () => ({
    data: {
      results: [
        {
          rule_id: "rule-1",
          rule_name: "department_name_required",
          passed: false,
          severity: "error",
          message: "Name is required",
          affected_record_ids: ["dept-1"],
        },
      ],
    },
  }),
  useCreateRule: () => ({
    isPending: false,
    mutateAsync: createRuleMock,
  }),
  useColumns: () => ({
    data: [
      {
        id: "column-1",
        column_name: "name",
        display_name: "Name",
      },
      {
        id: "column-2",
        column_name: "code",
        display_name: "Code",
      },
    ],
  }),
}));

describe("RuleConsolePage", () => {
  it("renders rules, results, and opens the create drawer", async () => {
    const user = userEvent.setup();

    render(
      <ConfigProvider>
        <RuleConsolePage />
      </ConfigProvider>
    );

    expect(screen.getByText("Rule Catalog")).toBeInTheDocument();
    expect(screen.getAllByText("department_name_required").length).toBeGreaterThan(0);
    expect(screen.getByText("Name is required")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: /New rule/i }));

    expect(screen.getByText("Create rule for departments")).toBeInTheDocument();
    expect(screen.getByText("Condition 1")).toBeInTheDocument();
  });

  it("adds another condition in the rule form", async () => {
    const user = userEvent.setup();

    render(
      <ConfigProvider>
        <RuleConsolePage />
      </ConfigProvider>
    );

    await user.click(screen.getByRole("button", { name: /New rule/i }));
    await user.click(screen.getByRole("button", { name: /Add condition/i }));

    expect(screen.getByText("Condition 2")).toBeInTheDocument();
    expect(screen.getByText("Connector")).toBeInTheDocument();
  });
});
