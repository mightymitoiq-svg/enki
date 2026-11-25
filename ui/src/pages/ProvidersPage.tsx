import { useCallback, useEffect, useState } from "react";
import {
  CheckCircle2,
  KeyRound,
  Plus,
  RefreshCw,
  Star,
  Trash2,
  XCircle,
} from "lucide-react";
import { api } from "../lib/api";
import type {
  ConnectionTestResult,
  Model,
  Provider,
  Protocol,
} from "../lib/schemas";
import { cn } from "../lib/utils";

const PROTOCOL_LABELS: Record<Protocol, string> = {
  "openai-completions": "OpenAI Chat Completions",
  "openai-responses": "OpenAI Responses",
  "anthropic-messages": "Anthropic Messages",
  "google-generative": "Google Generative",
  ollama: "Ollama",
};

export default function ProvidersPage() {
  const [providers, setProviders] = useState<Provider[]>([]);
  const [models, setModels] = useState<Model[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [selected, setSelected] = useState<string | null>(null);
  const [testResult, setTestResult] = useState<
    Record<string, ConnectionTestResult>
  >({});
  const [testing, setTesting] = useState<string | null>(null);
  // Per-provider API key dialog + save feedback
  const [keyDialogFor, setKeyDialogFor] = useState<string | null>(null);
  const [keyDraft, setKeyDraft] = useState("");
  const [keyStatus, setKeyStatus] = useState<
    Record<string, "idle" | "saving" | "saved" | "error">
  >({});
  // Delete confirmation dialog (replaces window.confirm)
  const [deleteTarget, setDeleteTarget] = useState<Provider | null>(null);

  // Custom provider form
  const [showForm, setShowForm] = useState(false);
  const [form, setForm] = useState({
    display_name: "",
    protocol: "openai-completions" as Protocol,
    base_url: "",
  });
  const [formError, setFormError] = useState<string | null>(null);

  // Model form
  const [modelForm, setModelForm] = useState({
    model_id: "",
    display_name: "",
    context_limit: "",
    output_limit: "",
  });
  const [modelError, setModelError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      const [p, m] = await Promise.all([api.listProviders(), api.listModels()]);
      setProviders(p);
      setModels(m);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const createProvider = async (e: React.FormEvent) => {
    e.preventDefault();
    setFormError(null);
    try {
      await api.createProvider({
        display_name: form.display_name.trim(),
        protocol: form.protocol,
        base_url: form.base_url.trim() || null,
      });
      setForm({ display_name: "", protocol: "openai-completions", base_url: "" });
      setShowForm(false);
      await refresh();
    } catch (err) {
      setFormError(err instanceof Error ? err.message : String(err));
    }
  };

  const addModel = async (e: React.FormEvent, providerId: string) => {
    e.preventDefault();
    setModelError(null);
    try {
      await api.upsertModel({
        provider_id: providerId,
        model_id: modelForm.model_id.trim(),
        display_name:
          modelForm.display_name.trim() || modelForm.model_id.trim(),
        context_limit: modelForm.context_limit
          ? parseInt(modelForm.context_limit, 10)
          : null,
        output_limit: modelForm.output_limit
          ? parseInt(modelForm.output_limit, 10)
          : null,
      });
      setModelForm({ model_id: "", display_name: "", context_limit: "", output_limit: "" });
      await refresh();
    } catch (err) {
      setModelError(err instanceof Error ? err.message : String(err));
    }
  };

  const setApiKey = async (providerId: string, key: string) => {
    setKeyStatus((prev) => ({ ...prev, [providerId]: "saving" }));
    try {
      await api.setProviderApiKey(providerId, key);
      await refresh();
      setKeyStatus((prev) => ({ ...prev, [providerId]: "saved" }));
      setTimeout(
        () => setKeyStatus((prev) => ({ ...prev, [providerId]: "idle" })),
        1500,
      );
    } catch (e) {
      setKeyStatus((prev) => ({ ...prev, [providerId]: "error" }));
      setError(e instanceof Error ? e.message : String(e));
      setTimeout(
        () => setKeyStatus((prev) => ({ ...prev, [providerId]: "idle" })),
        2500,
      );
    }
  };

  const testConnection = async (providerId: string) => {
    setTesting(providerId);
    try {
      const r = await api.testProviderConnection(providerId);
      setTestResult((prev) => ({ ...prev, [providerId]: r }));
    } catch (e) {
      setTestResult((prev) => ({
        ...prev,
        [providerId]: {
          ok: false,
          detail: e instanceof Error ? e.message : String(e),
          models_discovered: null,
        },
      }));
    } finally {
      setTesting(null);
    }
  };

  return (
    <div className="mx-auto max-w-5xl px-8 py-10">
      <div className="mb-6 flex items-center justify-between">
        <div>
          <h1 className="text-lg font-semibold tracking-tight">
            Providers &amp; Models
          </h1>
          <p className="mt-1 text-sm text-ink-dim">
            Built-in endpoints plus your custom OpenAI-compatible services. Add
            the models you actually have access to.
          </p>
        </div>
        <button
          onClick={() => setShowForm((v) => !v)}
          className="flex items-center gap-1.5 rounded-md border border-line bg-surface-1 px-3 py-1.5 text-xs text-ink-dim hover:border-line-strong hover:text-ink"
        >
          <Plus size={14} /> Custom provider
        </button>
      </div>

      {error && (
        <div role="alert" className="mb-4 rounded-md border border-danger/40 bg-danger/10 px-4 py-3 text-sm text-danger">
          {error}
        </div>
      )}

      {showForm && (
        <form onSubmit={createProvider} className="mb-6 rounded-md border border-line bg-surface-1 p-4">
          <div className="grid grid-cols-2 gap-3">
            <label className="text-xs text-ink-dim">
              Display name
              <input
                value={form.display_name}
                onChange={(e) => setForm({ ...form, display_name: e.target.value })}
                placeholder="My LM service"
                className="mt-1 w-full rounded-md border border-line bg-base px-3 py-1.5 text-sm text-ink"
              />
            </label>
            <label className="text-xs text-ink-dim">
              Protocol
              <select
                value={form.protocol}
                onChange={(e) =>
                  setForm({ ...form, protocol: e.target.value as Protocol })
                }
                className="mt-1 w-full rounded-md border border-line bg-base px-3 py-1.5 text-sm text-ink"
              >
                {Object.entries(PROTOCOL_LABELS).map(([v, label]) => (
                  <option key={v} value={v}>
                    {label}
                  </option>
                ))}
              </select>
            </label>
            <label className="col-span-2 text-xs text-ink-dim">
              Base URL
              <input
                value={form.base_url}
                onChange={(e) => setForm({ ...form, base_url: e.target.value })}
                placeholder="https://api.example.com/v1"
                className="mt-1 w-full rounded-md border border-line bg-base px-3 py-1.5 font-mono text-xs text-ink"
              />
            </label>
          </div>
          {formError && <p className="mt-2 text-xs text-danger">{formError}</p>}
          <button
            type="submit"
            className="mt-3 rounded-md bg-accent-strong px-4 py-1.5 text-xs font-medium text-white hover:bg-accent"
          >
            Create
          </button>
        </form>
      )}

      <ul className="space-y-2">
        {providers.map((p) => {
          const providerModels = models.filter((m) => m.provider_id === p.id);
          const isOpen = selected === p.id;
          const result = testResult[p.id];
          return (
            <li key={p.id} className="rounded-md border border-line bg-surface-1">
              <div className="flex items-center gap-3 px-4 py-3">
                <button
                  onClick={() => setSelected(isOpen ? null : p.id)}
                  className="min-w-0 flex-1 text-left"
                  aria-expanded={isOpen}
                >
                  <div className="flex items-center gap-2">
                    <span className="truncate text-sm font-medium">
                      {p.display_name}
                    </span>
                    <span
                      className={cn(
                        "rounded px-1.5 py-0.5 font-mono text-[10px]",
                        p.kind === "builtin"
                          ? "bg-surface-2 text-ink-faint"
                          : "bg-violet/15 text-violet",
                      )}
                    >
                      {p.kind}
                    </span>
                  </div>
                  <div className="truncate font-mono text-xs text-ink-faint">
                    {p.base_url ?? "—"}
                  </div>
                </button>

                {/* API key state */}
                <button
                  onClick={() => {
                    setKeyDraft("");
                    setKeyDialogFor(p.id);
                  }}
                  title={
                    p.has_api_key
                      ? "Replace API key"
                      : "Set API key"
                  }
                  className={cn(
                    "flex items-center gap-1 rounded-md border px-2 py-1 text-[11px]",
                    p.has_api_key
                      ? "border-ok/30 bg-ok/10 text-ok"
                      : "border-line bg-base text-ink-faint hover:text-ink",
                  )}
                >
                  <KeyRound size={12} />
                  {keyStatus[p.id] === "saving"
                    ? "Saving…"
                    : keyStatus[p.id] === "saved"
                      ? "Saved ✓"
                      : keyStatus[p.id] === "error"
                        ? "Failed"
                        : p.has_api_key
                          ? "Key set"
                          : "Set key"}
                </button>

                {/* Test connection */}
                <button
                  onClick={() => testConnection(p.id)}
                  disabled={testing === p.id}
                  className="flex items-center gap-1 rounded-md border border-line bg-base px-2 py-1 text-[11px] text-ink-dim hover:text-ink"
                >
                  <RefreshCw
                    size={12}
                    className={testing === p.id ? "animate-spin" : ""}
                  />
                  Test
                </button>

                {result && (
                  <span
                    className={cn(
                      "flex items-center gap-1 text-[11px]",
                      result.ok ? "text-ok" : "text-danger",
                    )}
                    title={result.detail}
                  >
                    {result.ok ? <CheckCircle2 size={13} /> : <XCircle size={13} />}
                    {result.ok
                      ? `${result.models_discovered ?? "?"} models`
                      : "failed"}
                  </span>
                )}

                {p.kind === "custom" && (
                  <button
                    onClick={() => setDeleteTarget(p)}
                    title="Delete provider"
                    className="rounded p-1.5 text-ink-faint hover:bg-surface-2 hover:text-danger"
                  >
                    <Trash2 size={14} />
                  </button>
                )}
              </div>

              {isOpen && (
                <div className="border-t border-line px-4 py-4">
                  {/* Existing models */}
                  {providerModels.length === 0 ? (
                    <p className="mb-3 text-xs text-ink-faint">
                      No models configured. Add the model IDs your account can
                      actually use — ENKI does not guess.
                    </p>
                  ) : (
                    <table className="mb-4 w-full text-left text-xs">
                      <thead>
                        <tr className="border-b border-line text-ink-faint">
                          <th className="py-1.5 pr-2 font-medium">Model</th>
                          <th className="py-1.5 pr-2 font-medium">Context</th>
                          <th className="py-1.5 pr-2 font-medium">Output</th>
                          <th className="py-1.5 pr-2 font-medium">Tools</th>
                          <th className="py-1.5 pr-2 font-medium">Vision</th>
                          <th className="py-1.5 pr-2 font-medium">Default</th>
                          <th className="py-1.5" />
                        </tr>
                      </thead>
                      <tbody>
                        {providerModels.map((m) => (
                          <tr key={m.id} className="border-b border-line/50">
                            <td className="py-1.5 pr-2">
                              <div className="font-mono">{m.model_id}</div>
                              <div className="text-ink-faint">{m.display_name}</div>
                            </td>
                            <td className="py-1.5 pr-2 font-mono text-ink-dim">
                              {m.context_limit?.toLocaleString() ?? "—"}
                            </td>
                            <td className="py-1.5 pr-2 font-mono text-ink-dim">
                              {m.output_limit?.toLocaleString() ?? "—"}
                            </td>
                            <td className="py-1.5 pr-2 text-ink-dim">
                              {m.supports_tools ? "yes" : "no"}
                            </td>
                            <td className="py-1.5 pr-2 text-ink-dim">
                              {m.supports_vision ? "yes" : "no"}
                            </td>
                            <td className="py-1.5 pr-2">
                              {m.is_default ? (
                                <Star size={13} className="fill-warn text-warn" />
                              ) : (
                                <button
                                  onClick={async () => {
                                    await api.setDefaultModel(m.id);
                                    await refresh();
                                  }}
                                  title="Set as default"
                                  className="text-ink-faint hover:text-warn"
                                >
                                  <Star size={13} />
                                </button>
                              )}
                            </td>
                            <td className="py-1.5 text-right">
                              <button
                                onClick={async () => {
                                  await api.deleteModel(m.id);
                                  await refresh();
                                }}
                                className="text-ink-faint hover:text-danger"
                                title="Remove model"
                              >
                                <Trash2 size={12} />
                              </button>
                            </td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  )}

                  {/* Add model */}
                  <form
                    onSubmit={(e) => addModel(e, p.id)}
                    className="grid grid-cols-[1.2fr_1.2fr_0.8fr_0.8fr_auto] items-end gap-2"
                  >
                    <label className="text-[11px] text-ink-dim">
                      Model ID
                      <input
                        value={modelForm.model_id}
                        onChange={(e) =>
                          setModelForm({ ...modelForm, model_id: e.target.value })
                        }
                        placeholder="gpt-4o"
                        className="mt-1 w-full rounded border border-line bg-base px-2 py-1 font-mono text-xs text-ink"
                      />
                    </label>
                    <label className="text-[11px] text-ink-dim">
                      Display name
                      <input
                        value={modelForm.display_name}
                        onChange={(e) =>
                          setModelForm({
                            ...modelForm,
                            display_name: e.target.value,
                          })
                        }
                        className="mt-1 w-full rounded border border-line bg-base px-2 py-1 text-xs text-ink"
                      />
                    </label>
                    <label className="text-[11px] text-ink-dim">
                      Context limit
                      <input
                        value={modelForm.context_limit}
                        onChange={(e) =>
                          setModelForm({
                            ...modelForm,
                            context_limit: e.target.value,
                          })
                        }
                        inputMode="numeric"
                        className="mt-1 w-full rounded border border-line bg-base px-2 py-1 font-mono text-xs text-ink"
                      />
                    </label>
                    <label className="text-[11px] text-ink-dim">
                      Output limit
                      <input
                        value={modelForm.output_limit}
                        onChange={(e) =>
                          setModelForm({
                            ...modelForm,
                            output_limit: e.target.value,
                          })
                        }
                        inputMode="numeric"
                        className="mt-1 w-full rounded border border-line bg-base px-2 py-1 font-mono text-xs text-ink"
                      />
                    </label>
                    <button
                      type="submit"
                      className="h-[30px] rounded bg-accent-strong px-3 text-[11px] font-medium text-white hover:bg-accent"
                    >
                      Add
                    </button>
                  </form>
                  {modelError && (
                    <p className="mt-2 text-xs text-danger">{modelError}</p>
                  )}
                </div>
              )}
            </li>
          );
        })}
      </ul>

      {/* API key dialog — masked input, no window.prompt */}
      {keyDialogFor && (
        <div
          role="dialog"
          aria-modal="true"
          aria-label="Set API key"
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4"
          onClick={(e) => {
            if (e.target === e.currentTarget) setKeyDialogFor(null);
          }}
        >
          <form
            onSubmit={(e) => {
              e.preventDefault();
              const id = keyDialogFor;
              setKeyDialogFor(null);
              setApiKey(id, keyDraft);
            }}
            className="w-full max-w-md rounded-lg border border-line bg-surface-1 p-5"
          >
            <h2 className="text-sm font-semibold">
              {providers.find((p) => p.id === keyDialogFor)?.has_api_key
                ? "Replace API key"
                : "Set API key"}
            </h2>
            <p className="mt-1 text-xs text-ink-faint">
              Stored in the OS credential store — never in the database or
              logs. Only a key-presence indicator is kept.
            </p>
            <input
              type="password"
              autoFocus
              autoComplete="off"
              value={keyDraft}
              onChange={(e) => setKeyDraft(e.target.value)}
              placeholder="sk-…"
              className="mt-3 w-full rounded-md border border-line bg-base px-3 py-1.5 font-mono text-sm text-ink"
            />
            <div className="mt-4 flex justify-end gap-2">
              <button
                type="button"
                onClick={() => setKeyDialogFor(null)}
                className="rounded-md border border-line px-3 py-1.5 text-xs text-ink-dim hover:text-ink"
              >
                Cancel
              </button>
              <button
                type="submit"
                disabled={!keyDraft.trim()}
                className="rounded-md bg-accent-strong px-4 py-1.5 text-xs font-medium text-white hover:bg-accent disabled:opacity-50"
              >
                Save key
              </button>
            </div>
          </form>
        </div>
      )}

      {/* Delete provider confirmation — no window.confirm */}
      {deleteTarget && (
        <div
          role="dialog"
          aria-modal="true"
          aria-label="Delete provider"
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4"
          onClick={(e) => {
            if (e.target === e.currentTarget) setDeleteTarget(null);
          }}
        >
          <div className="w-full max-w-md rounded-lg border border-line bg-surface-1 p-5">
            <h2 className="text-sm font-semibold">
              Delete provider “{deleteTarget.display_name}”?
            </h2>
            <p className="mt-1 text-xs text-ink-faint">
              Its configured models are removed too. The API key stays in the
              OS credential store until you clear it there.
            </p>
            <div className="mt-4 flex justify-end gap-2">
              <button
                type="button"
                autoFocus
                onClick={() => setDeleteTarget(null)}
                className="rounded-md border border-line px-3 py-1.5 text-xs text-ink-dim hover:text-ink"
              >
                Cancel
              </button>
              <button
                type="button"
                onClick={async () => {
                  const id = deleteTarget.id;
                  setDeleteTarget(null);
                  try {
                    await api.deleteProvider(id);
                    await refresh();
                  } catch (e) {
                    setError(e instanceof Error ? e.message : String(e));
                  }
                }}
                className="rounded-md bg-danger px-4 py-1.5 text-xs font-medium text-white hover:opacity-90"
              >
                Delete
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
