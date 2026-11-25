import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import App from "./App";

// Mock the Tauri bridge so the UI can render outside the desktop shell.
const invokeDefault = (cmd: string) => {
  switch (cmd) {
    case "app_version":
      return Promise.resolve("0.1.0");
    case "list_projects":
      return Promise.resolve([
        {
          id: "p1",
          name: "Demo project",
          root_path: "C:\\dev\\demo",
          created_at: "2026-11-24 10:00:00",
          last_opened_at: "2026-11-24 10:00:00",
        },
      ]);
    case "list_sessions":
      return Promise.resolve([]);
    case "get_settings":
      return Promise.resolve({
        theme: "dark",
        accent: "electric",
        editor_font_size: 14,
        show_agent_thoughts: true,
        confirm_terminal_commands: true,
        confirm_file_writes: true,
        allow_reads_outside_workspace: false,
        default_provider_id: "",
        default_model_id: "",
        monthly_budget_usd_cents: 0,
      });
    default:
      return Promise.resolve([]);
  }
};
const invokeMock = vi.fn(invokeDefault);

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, _args?: unknown) => invokeMock(cmd),
}));

describe("App shell", () => {
  beforeEach(() => {
    invokeMock.mockClear();
    // Theme class persists on documentElement across renders in jsdom —
    // every test starts from the dark default.
    document.documentElement.classList.remove("light");
  });

  it("renders navigation and the Home page with projects from the backend", async () => {
    render(<App />);
    // Nav buttons
    expect(screen.getByTitle(/Home \(Alt\+1\)/)).toBeDefined();
    expect(screen.getByTitle(/Providers \(Alt\+2\)/)).toBeDefined();
    expect(screen.getByTitle(/Agents \(Alt\+3\)/)).toBeDefined();
    expect(screen.getByTitle(/Settings \(Alt\+4\)/)).toBeDefined();
    // Home content from the mocked backend
    await waitFor(() => {
      expect(screen.getByText("Demo project")).toBeDefined();
      expect(screen.getByText("C:\\dev\\demo")).toBeDefined();
    });
    expect(screen.getByText(/v0\.1\.0/)).toBeDefined();
  });

  it("switches pages via nav buttons", async () => {
    render(<App />);
    const navBtn = screen.getByTitle(/Settings \(Alt\+4\)/);
    navBtn.click();
    await waitFor(() => {
      expect(screen.getByText("Appearance")).toBeDefined();
    });
  });

  it("applies the light theme class when the saved theme is light", async () => {
    invokeMock.mockImplementation((cmd: string) => {
      switch (cmd) {
        case "get_settings":
          return Promise.resolve({
            theme: "light",
            accent: "electric",
            editor_font_size: 14,
            show_agent_thoughts: true,
            confirm_terminal_commands: true,
            confirm_file_writes: true,
            allow_reads_outside_workspace: false,
            default_provider_id: "",
            default_model_id: "",
            monthly_budget_usd_cents: 0,
          });
        default:
          return Promise.resolve([]);
      }
    });
    render(<App />);
    await waitFor(() => {
      expect(document.documentElement.classList.contains("light")).toBe(true);
    });
    // Restore the default mock for subsequent tests.
    invokeMock.mockImplementation((cmd: string) => invokeDefault(cmd));
  });

  it("stays dark (no light class) for the default theme", async () => {
    render(<App />);
    // Wait for settings to load (they resolve dark), then the class must
    // never have been applied.
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("get_settings");
    });
    expect(document.documentElement.classList.contains("light")).toBe(false);
  });
});
