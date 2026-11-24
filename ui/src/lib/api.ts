import { invoke } from "@tauri-apps/api/core";
import { z } from "zod";
import {
  appSettingsSchema,
  auditEntrySchema,
  connectionTestResultSchema,
  engineInfoSchema,
  modelSchema,
  projectSchema,
  providerSchema,
  sessionSchema,
  type AppSettings,
  type AuditEntry,
  type ConnectionTestResult,
  type EngineInfo,
  type Model,
  type Project,
  type Provider,
  type Protocol,
  type Session,
} from "./schemas";

/**
 * Typed wrappers around Tauri `invoke`. Every response is Zod-validated:
 * if the backend ever sends something unexpected we fail loudly instead of
 * rendering corrupt state.
 */

/** Input shape for provider create/update commands. */
export interface ProviderInput {
  display_name: string;
  protocol: Protocol;
  base_url?: string | null;
  extra_headers?: Record<string, string> | null;
  extra_params?: Record<string, unknown> | null;
  enabled?: boolean | null;
}

export const api = {
  appVersion(): Promise<string> {
    return invoke<string>("app_version");
  },

  // settings
  getSettings(): Promise<AppSettings> {
    return invoke("get_settings").then((r) => appSettingsSchema.parse(r));
  },
  saveSettings(settings: AppSettings): Promise<AppSettings> {
    return invoke("save_settings", { settings }).then((r) =>
      appSettingsSchema.parse(r),
    );
  },

  // projects
  listProjects(): Promise<Project[]> {
    return invoke("list_projects").then((r) =>
      z.array(projectSchema).parse(r),
    );
  },
  createProject(name: string, rootPath: string): Promise<Project> {
    return invoke("create_project", {
      input: { name, root_path: rootPath },
    }).then((r) => projectSchema.parse(r));
  },
  openProject(projectId: string): Promise<Project> {
    return invoke("open_project", { projectId }).then((r) =>
      projectSchema.parse(r),
    );
  },
  deleteProject(projectId: string): Promise<void> {
    return invoke("delete_project", { projectId });
  },

  // sessions
  listSessions(projectId?: string): Promise<Session[]> {
    return invoke("list_sessions", { projectId: projectId ?? null }).then(
      (r) => z.array(sessionSchema).parse(r),
    );
  },
  createSession(projectId: string, engineId: string, title?: string): Promise<Session> {
    return invoke("create_session", {
      input: { project_id: projectId, engine_id: engineId, title: title ?? null },
    }).then((r) => sessionSchema.parse(r));
  },
  renameSession(sessionId: string, title: string): Promise<Session> {
    return invoke("rename_session", { sessionId, title }).then((r) =>
      sessionSchema.parse(r),
    );
  },
  deleteSession(sessionId: string): Promise<void> {
    return invoke("delete_session", { sessionId });
  },

  // providers & models
  listProviders(): Promise<Provider[]> {
    return invoke("list_providers").then((r) =>
      z.array(providerSchema).parse(r),
    );
  },
  createProvider(input: ProviderInput): Promise<Provider> {
    return invoke("create_provider", { input }).then((r) =>
      providerSchema.parse(r),
    );
  },
  updateProvider(providerId: string, input: ProviderInput): Promise<Provider> {
    return invoke("update_provider", { providerId, input }).then((r) =>
      providerSchema.parse(r),
    );
  },
  deleteProvider(providerId: string): Promise<void> {
    return invoke("delete_provider", { providerId });
  },
  listModels(providerId?: string): Promise<Model[]> {
    return invoke("list_models", { providerId: providerId ?? null }).then(
      (r) => z.array(modelSchema).parse(r),
    );
  },
  upsertModel(input: {
    provider_id: string;
    model_id: string;
    display_name: string;
    context_limit?: number | null;
    output_limit?: number | null;
    supports_streaming?: boolean | null;
    supports_tools?: boolean | null;
    supports_vision?: boolean | null;
    supports_json_schema?: boolean | null;
  }): Promise<Model> {
    return invoke("upsert_model", { input }).then((r) => modelSchema.parse(r));
  },
  deleteModel(modelId: string): Promise<void> {
    return invoke("delete_model", { modelId });
  },
  setDefaultModel(modelId: string): Promise<void> {
    return invoke("set_default_model", { modelId });
  },
  setProviderApiKey(providerId: string, apiKey: string): Promise<void> {
    return invoke("set_provider_api_key", { providerId, apiKey });
  },
  deleteProviderApiKey(providerId: string): Promise<void> {
    return invoke("delete_provider_api_key", { providerId });
  },
  testProviderConnection(providerId: string): Promise<ConnectionTestResult> {
    return invoke("test_provider_connection", { providerId }).then((r) =>
      connectionTestResultSchema.parse(r),
    );
  },

  // engines
  discoverEngines(): Promise<EngineInfo[]> {
    return invoke("discover_engines").then((r) =>
      z.array(engineInfoSchema).parse(r),
    );
  },

  // audit
  recentAudit(limit?: number): Promise<AuditEntry[]> {
    return invoke("recent_audit", { limit: limit ?? null }).then((r) =>
      z.array(auditEntrySchema).parse(r),
    );
  },

  // security
  checkWorkspacePath(
    projectId: string,
    path: string,
    write: boolean,
  ): Promise<{ allowed: boolean }> {
    return invoke("check_workspace_path", {
      args: { project_id: projectId, path, write },
    });
  },
};
