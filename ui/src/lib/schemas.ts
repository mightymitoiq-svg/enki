import { z } from "zod";

/**
 * Zod schemas mirroring the Rust types in src-tauri/src.
 * These validate what the backend sends over the Tauri bridge —
 * treat backend output as untrusted input.
 */

export const appSettingsSchema = z.object({
  theme: z.enum(["dark", "light"]),
  accent: z.literal("electric"),
  editor_font_size: z.number().int().min(10).max(28),
  show_agent_thoughts: z.boolean(),
  confirm_terminal_commands: z.boolean(),
  confirm_file_writes: z.boolean(),
  allow_reads_outside_workspace: z.boolean(),
  default_provider_id: z.string(),
  default_model_id: z.string(),
  monthly_budget_usd_cents: z.number().int().min(0),
});
export type AppSettings = z.infer<typeof appSettingsSchema>;

export const projectSchema = z.object({
  id: z.string(),
  name: z.string(),
  root_path: z.string(),
  created_at: z.string(),
  last_opened_at: z.string(),
});
export type Project = z.infer<typeof projectSchema>;

export const sessionSchema = z.object({
  id: z.string(),
  project_id: z.string(),
  engine_id: z.string(),
  title: z.string(),
  created_at: z.string(),
  updated_at: z.string(),
});
export type Session = z.infer<typeof sessionSchema>;

export const protocolSchema = z.enum([
  "openai-completions",
  "openai-responses",
  "anthropic-messages",
  "google-generative",
  "ollama",
]);
export type Protocol = z.infer<typeof protocolSchema>;

export const providerSchema = z.object({
  id: z.string(),
  kind: z.enum(["builtin", "custom"]),
  display_name: z.string(),
  protocol: protocolSchema,
  base_url: z.string().nullable(),
  extra_headers: z.record(z.string(), z.string()),
  extra_params: z.record(z.string(), z.string()),
  enabled: z.boolean(),
  created_at: z.string(),
  has_api_key: z.boolean(),
});
export type Provider = z.infer<typeof providerSchema>;

export const modelSchema = z.object({
  id: z.string(),
  provider_id: z.string(),
  model_id: z.string(),
  display_name: z.string(),
  context_limit: z.number().int().nullable(),
  output_limit: z.number().int().nullable(),
  supports_streaming: z.boolean(),
  supports_tools: z.boolean(),
  supports_vision: z.boolean(),
  supports_json_schema: z.boolean(),
  is_default: z.boolean(),
});
export type Model = z.infer<typeof modelSchema>;

export const engineCapabilitiesSchema = z.object({
  streaming: z.boolean(),
  tool_calls: z.boolean(),
  file_editing: z.boolean(),
  terminal_execution: z.boolean(),
  session_persistence: z.boolean(),
  interrupt: z.boolean(),
  multi_agent: z.boolean(),
  extras: z.record(z.string(), z.boolean()),
});
export type EngineCapabilities = z.infer<typeof engineCapabilitiesSchema>;

export const engineInfoSchema = z.object({
  id: z.string(),
  display_name: z.string(),
  adapter_version: z.number().int(),
  status: z.discriminatedUnion("type", [
    z.object({ type: z.literal("available") }),
    z.object({ type: z.literal("not_installed") }),
    z.object({ type: z.literal("error"), message: z.string() }),
  ]),
  capabilities: engineCapabilitiesSchema,
  detected_version: z.string().nullable(),
  notes: z.array(z.string()),
});
export type EngineInfo = z.infer<typeof engineInfoSchema>;

export const connectionTestResultSchema = z.object({
  ok: z.boolean(),
  detail: z.string(),
  models_discovered: z.number().int().nullable(),
});
export type ConnectionTestResult = z.infer<typeof connectionTestResultSchema>;

export const auditEntrySchema = z.object({
  id: z.number().int(),
  ts: z.string(),
  category: z.string(),
  event: z.string(),
  detail: z.string(),
});
export type AuditEntry = z.infer<typeof auditEntrySchema>;
