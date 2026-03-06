import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { ConfigProvider } from "antd";
import { App } from "./App";

describe("App", () => {
  it("renders navigation shell", async () => {
    const queryClient = new QueryClient();
    render(
      <QueryClientProvider client={queryClient}>
        <MemoryRouter future={{ v7_startTransition: true, v7_relativeSplatPath: true }}>
          <ConfigProvider>
            <App />
          </ConfigProvider>
        </MemoryRouter>
      </QueryClientProvider>
    );

    expect(await screen.findByText("Master Maintenance")).toBeInTheDocument();
    expect(await screen.findByText("Dashboard")).toBeInTheDocument();
  });
});
