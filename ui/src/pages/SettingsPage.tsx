import { useCallback, useEffect, useState } from "react";
import { api } from "../lib/api";
import { appSettingsSchema, type AppSettings } from "../lib/schemas";
import { cn } from "../lib/utils";

export default function SettingsPage() {
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [status, setStatus] = useState<"idle" | "saving" | "saved" | "error">("idle");
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    api
      .getSettings()
      .then(setSettings)
      .catch((e: unknown) =>
        setError(e instanceof Error ? e.message : String(e)),
      );
  }, []);

  const save = useCallback(async (next: AppSettings) => {
    setStatus("saving");
    setError(null);
    try {
      const parsed = appSettingsSchema.parse(next); // validate before sending
      const saved = await api.saveSettings(parsed);
      setSettings(saved);
      window.dispatchEvent(new CustomEvent("enki:settings-saved", { detail: saved }));
      setStatus("saved");
      setTimeout(() => setStatus("idle"), 1500);
    } catch (e) {
      setStatus("error");
      setError(e instanceof Error ? e.message : String(e));
    }
  }, []);

  if (error && !settings) {
    return (
      <div className="mx-auto max-w-3xl px-8 py-10">
        <div role="alert" className="rounded-md border border-danger/40 bg-danger/10 px-4 py-3 text-sm text-danger">
          Could not load settings: {error}
        </div>
      </div>
    );
  }

  if (!settings) {
    return (
      <div className="mx-auto max-w-3xl px-8 py-10">
        <p className="text-sm text-ink-faint">Loading…</p>
      </div>
    );
  }

  const patch = (p: Partial<AppSettings>) => {
    if (p.theme) document.documentElement.dataset.theme = p.theme;
    setSettings({ ...settings, ...p });
  };

  return (
    <div className="mx-auto max-w-3xl px-8 py-10">
      <div className="mb-6 flex items-center justify-between">
        <h1 className="text-lg font-semibold tracking-tight">Settings</h1>
        <span
          aria-live="polite"
          className={cn(
            "text-xs",
            status === "saved" && "text-ok",
            status === "saving" && "text-ink-dim",
            status === "error" && "text-danger",
          )}
        >
          {status === "saved" && "Saved ✓"}
          {status === "saving" && "Saving…"}
          {status === "error" && "Save failed"}
        </span>
      </div>

      {error && (
        <div role="alert" className="mb-4 rounded-md border border-danger/40 bg-danger/10 px-4 py-3 text-sm text-danger">
          {error}
        </div>
      )}

      <form
        noValidate
        onSubmit={(e) => {
          e.preventDefault();
          save(settings);
        }}
        className="space-y-8"
      >
        {/* Appearance */}
        <fieldset className="rounded-md border border-line bg-surface-1 p-5">
          <legend className="px-2 text-xs font-semibold uppercase tracking-wider text-ink-dim">
            Appearance
          </legend>
          <div className="grid grid-cols-2 gap-6">
            <label className="text-sm">
              <span className="text-ink-dim">Theme</span>
              <select
                value={settings.theme}
                onChange={(e) => patch({ theme: e.target.value as AppSettings["theme"] })}
                className="mt-1.5 w-full rounded-md border border-line bg-base px-3 py-1.5 text-sm text-ink"
              >
                <option value="dark">Dark (recommended)</option>
                <option value="light">Light</option>
              </select>
            </label>
            <label className="text-sm">
              <span className="text-ink-dim">Editor font size ({settings.editor_font_size}px)</span>
              <input
                type="range"
                min={10}
                max={28}
                value={settings.editor_font_size}
                onChange={(e) =>
                  patch({ editor_font_size: parseInt(e.target.value, 10) })
                }
                className="mt-3 w-full accent-[var(--color-accent)]"
              />
            </label>
          </div>
        </fieldset>

        {/* Behavior */}
        <fieldset className="rounded-md border border-line bg-surface-1 p-5">
          <legend className="px-2 text-xs font-semibold uppercase tracking-wider text-ink-dim">
            Behavior &amp; Security
          </legend>
          <div className="space-y-3">
            {(
              [
                {
                  key: "show_agent_thoughts",
                  label: "Show agent reasoning phases",
                  hint: "Conversation views are not available in this build.",
                },
                {
                  key: "confirm_terminal_commands",
                  label: "Confirm terminal commands before execution",
                  hint: "Terminal execution is not available in this build.",
                },
                {
                  key: "confirm_file_writes",
                  label: "Confirm file writes",
                  hint: "Agent file editing is not available in this build.",
                },
                {
                  key: "allow_reads_outside_workspace",
                  label: "Allow reads outside the workspace",
                  hint: "Used by path checks. This build does not read project files.",
                },
              ] as const
            ).map((item) => (
              <label
                key={item.key}
                className={cn(
                  "flex items-start justify-between gap-4 rounded-md px-3 py-2",
                  item.key === "allow_reads_outside_workspace" ? "cursor-pointer hover:bg-surface" : "cursor-not-allowed opacity-65",
                )}
              >
                <span>
                  <span className="block text-sm">{item.label}</span>
                  <span className="block text-xs text-ink-faint">{item.hint}</span>
                </span>
                <input
                  type="checkbox"
                  checked={settings[item.key]}
                  disabled={item.key !== "allow_reads_outside_workspace"}
                  onChange={(e) => patch({ [item.key]: e.target.checked })}
                  className="mt-1 h-4 w-4 shrink-0 accent-[var(--color-accent)] disabled:cursor-not-allowed"
                />
              </label>
            ))}
          </div>
        </fieldset>

        {/* Budget */}
        <fieldset className="rounded-md border border-line bg-surface-1 p-5">
          <legend className="px-2 text-xs font-semibold uppercase tracking-wider text-ink-dim">
            Usage Budget
          </legend>
          <label className="block text-sm">
            <span className="text-ink-dim">
              Monthly budget (USD) — 0 disables the limit
            </span>
            <input
              type="number"
              min={0}
              step={1}
              value={settings.monthly_budget_usd_cents / 100}
              onChange={(e) =>
                patch({
                  monthly_budget_usd_cents: Math.round(
                    Math.max(0, parseFloat(e.target.value) || 0) * 100,
                  ),
                })
              }
              className="mt-1.5 w-40 rounded-md border border-line bg-base px-3 py-1.5 font-mono text-sm text-ink"
            />
          </label>
          <p className="mt-2 text-xs text-ink-faint">
            Usage tracking activates with the AI gateway&apos;s request routing
            (next milestone); the budget value is stored now.
          </p>
        </fieldset>

        <div className="flex justify-end">
          <button
            type="submit"
            disabled={status === "saving"}
            className="rounded-md bg-accent-strong px-5 py-2 text-sm font-medium text-white transition-colors hover:bg-accent disabled:opacity-50"
          >
            {status === "saving" ? "Saving…" : "Save settings"}
          </button>
        </div>
      </form>
    </div>
  );
}
