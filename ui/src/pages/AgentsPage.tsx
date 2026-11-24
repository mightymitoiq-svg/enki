import { useCallback, useEffect, useState } from "react";
import { CheckCircle2, CircleSlash, RefreshCw } from "lucide-react";
import { api } from "../lib/api";
import type { EngineInfo } from "../lib/schemas";
import { cn } from "../lib/utils";

const CAPABILITY_LABELS: Record<string, string> = {
  streaming: "Streaming responses",
  tool_calls: "Tool calling",
  file_editing: "File editing",
  terminal_execution: "Terminal execution",
  session_persistence: "Session persistence",
  interrupt: "Interrupt / recovery",
  multi_agent: "Multi-agent",
};

export default function AgentsPage() {
  const [engines, setEngines] = useState<EngineInfo[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [scanning, setScanning] = useState(false);

  const refresh = useCallback(async () => {
    setScanning(true);
    try {
      setEngines(await api.discoverEngines());
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setScanning(false);
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  return (
    <div className="mx-auto max-w-4xl px-8 py-10">
      <div className="mb-6 flex items-center justify-between">
        <div>
          <h1 className="text-lg font-semibold tracking-tight">Agent Engines</h1>
          <p className="mt-1 text-sm text-ink-dim">
            Discovery runs a real probe (e.g. <code className="rounded bg-surface-2 px-1 font-mono text-xs">opencode --version</code>).
            Capabilities shown are what the adapter actually verified — nothing is
            simulated.
          </p>
        </div>
        <button
          onClick={refresh}
          disabled={scanning}
          className="flex items-center gap-1.5 rounded-md border border-line bg-surface-1 px-3 py-1.5 text-xs text-ink-dim hover:border-line-strong hover:text-ink"
        >
          <RefreshCw size={14} className={scanning ? "animate-spin" : ""} />
          Rescan
        </button>
      </div>

      {error && (
        <div role="alert" className="mb-4 rounded-md border border-danger/40 bg-danger/10 px-4 py-3 text-sm text-danger">
          {error}
        </div>
      )}

      <ul className="space-y-3">
        {engines.map((engine) => (
          <li key={engine.id} className="rounded-md border border-line bg-surface-1">
            <div className="flex items-center gap-3 border-b border-line px-4 py-3">
              {engine.status.type === "available" ? (
                <CheckCircle2 size={16} className="text-ok" />
              ) : (
                <CircleSlash size={16} className="text-ink-faint" />
              )}
              <div className="flex-1">
                <span className="text-sm font-medium">{engine.display_name}</span>
                <span className="ml-2 rounded bg-surface-2 px-1.5 py-0.5 font-mono text-[10px] text-ink-faint">
                  adapter v{engine.adapter_version}
                </span>
              </div>
              <span
                className={cn(
                  "font-mono text-xs",
                  engine.status.type === "available" ? "text-ok" : "text-ink-faint",
                )}
              >
                {engine.status.type === "available"
                  ? engine.detected_version ?? "available"
                  : "not installed"}
              </span>
            </div>

            <div className="px-4 py-3">
              <div className="mb-3 flex flex-wrap gap-1.5">
                {Object.entries(CAPABILITY_LABELS).map(([key, label]) => {
                  const supported =
                    engine.capabilities[key as keyof typeof engine.capabilities];
                  return (
                    <span
                      key={key}
                      title={
                        supported === true
                          ? "Supported"
                          : supported === false
                            ? "Not supported by this engine"
                            : "Unknown"
                      }
                      className={cn(
                        "rounded border px-2 py-0.5 text-[11px]",
                        supported
                          ? "border-ok/25 bg-ok/5 text-ok"
                          : "border-line bg-base text-ink-faint line-through",
                      )}
                    >
                      {label}
                    </span>
                  );
                })}
              </div>
              <ul className="space-y-1">
                {engine.notes.map((note, i) => (
                  <li key={i} className="text-xs leading-relaxed text-ink-faint">
                    {note}
                  </li>
                ))}
              </ul>
            </div>
          </li>
        ))}
      </ul>

      <p className="mt-6 text-xs leading-relaxed text-ink-faint">
        Codex, Claude Code and custom engine adapters will appear here as they
        are implemented. Each adapter must implement the same discovery
        contract; unsupported features are shown as unsupported rather than
        hidden or faked.
      </p>
    </div>
  );
}
