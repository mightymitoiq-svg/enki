import { describe, expect, it } from "vitest";
import {
  appSettingsSchema,
  engineInfoSchema,
  modelSchema,
  providerSchema,
} from "./schemas";

/** These tests pin the frontend–backend contract: the Zod schemas must accept
 * exactly what the Rust serde types emit (snake_case field names, enum
 * spellings) and reject corruption. */

describe("appSettingsSchema", () => {
  it("accepts a valid settings payload with snake_case keys", () => {
    const parsed = appSettingsSchema.parse({
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
    expect(parsed.theme).toBe("dark");
    expect(parsed.editor_font_size).toBe(14);
  });

  it("rejects unknown themes", () => {
    expect(() =>
      appSettingsSchema.parse({
        theme: "solarized",
        accent: "electric",
        editor_font_size: 14,
        show_agent_thoughts: true,
        confirm_terminal_commands: true,
        confirm_file_writes: true,
        allow_reads_outside_workspace: false,
        default_provider_id: "",
        default_model_id: "",
        monthly_budget_usd_cents: 0,
      }),
    ).toThrow();
  });

  it("rejects out-of-range font size", () => {
    expect(() =>
      appSettingsSchema.parse({
        theme: "dark",
        accent: "electric",
        editor_font_size: 99,
        show_agent_thoughts: true,
        confirm_terminal_commands: true,
        confirm_file_writes: true,
        allow_reads_outside_workspace: false,
        default_provider_id: "",
        default_model_id: "",
        monthly_budget_usd_cents: 0,
      }),
    ).toThrow();
  });
});

describe("providerSchema", () => {
  it("accepts a builtin provider and reports key presence", () => {
    const p = providerSchema.parse({
      id: "uuid-1",
      kind: "builtin",
      display_name: "Ollama (local)",
      protocol: "ollama",
      base_url: "http://localhost:11434",
      extra_headers: {},
      extra_params: {},
      enabled: true,
      created_at: "2026-11-24 12:00:00",
      has_api_key: false,
    });
    expect(p.kind).toBe("builtin");
    expect(p.has_api_key).toBe(false);
  });

  it("rejects an unknown protocol", () => {
    expect(() =>
      providerSchema.parse({
        id: "uuid-2",
        kind: "custom",
        display_name: "X",
        protocol: "groq",
        base_url: null,
        extra_headers: {},
        extra_params: {},
        enabled: true,
        created_at: "2026-11-24 12:00:00",
        has_api_key: false,
      }),
    ).toThrow();
  });
});

describe("modelSchema", () => {
  it("parses nullable limits", () => {
    const m = modelSchema.parse({
      id: "p1:m1",
      provider_id: "p1",
      model_id: "m1",
      display_name: "M1",
      context_limit: null,
      output_limit: 4096,
      supports_streaming: true,
      supports_tools: false,
      supports_vision: false,
      supports_json_schema: false,
      is_default: false,
    });
    expect(m.context_limit).toBeNull();
    expect(m.output_limit).toBe(4096);
  });
});

describe("engineInfoSchema", () => {
  it("parses available and not_installed statuses", () => {
    const available = engineInfoSchema.parse({
      id: "opencode",
      display_name: "OpenCode",
      adapter_version: 1,
      status: { type: "available" },
      capabilities: {
        streaming: true,
        tool_calls: true,
        file_editing: true,
        terminal_execution: true,
        session_persistence: true,
        interrupt: true,
        multi_agent: false,
        extras: { headless_mode: true },
      },
      detected_version: "opencode 0.14.2",
      notes: [],
    });
    expect(available.status.type).toBe("available");

    const missing = engineInfoSchema.parse({
      id: "opencode",
      display_name: "OpenCode",
      adapter_version: 1,
      status: { type: "not_installed" },
      capabilities: {
        streaming: false,
        tool_calls: false,
        file_editing: false,
        terminal_execution: false,
        session_persistence: false,
        interrupt: false,
        multi_agent: false,
        extras: {},
      },
      detected_version: null,
      notes: [],
    });
    expect(missing.status.type).toBe("not_installed");
  });
});
