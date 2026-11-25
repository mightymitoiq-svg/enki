import { useCallback, useEffect, useState } from "react";
import { FolderPlus, MessageSquare, Trash2, Zap } from "lucide-react";
import { api } from "../lib/api";
import type { Project, Session } from "../lib/schemas";
import { cn } from "../lib/utils";

type Props = {
  onNavigate: (page: "home" | "providers" | "agents" | "settings") => void;
};

export default function HomePage({ onNavigate }: Props) {
  const [projects, setProjects] = useState<Project[]>([]);
  const [sessions, setSessions] = useState<Session[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  // New project form state
  const [showForm, setShowForm] = useState(false);
  const [name, setName] = useState("");
  const [path, setPath] = useState("");
  const [formError, setFormError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      const [p, s] = await Promise.all([api.listProjects(), api.listSessions()]);
      setProjects(p);
      setSessions(s.slice(0, 8));
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const createProject = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim() || !path.trim()) {
      setFormError("Name and path are required.");
      return;
    }
    setBusy(true);
    setFormError(null);
    try {
      await api.createProject(name.trim(), path.trim());
      setName("");
      setPath("");
      setShowForm(false);
      await refresh();
    } catch (err) {
      setFormError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  };

  const removeProject = async (id: string, projName: string) => {
    if (!window.confirm(`Remove project "${projName}"? Sessions will also be removed.`)) return;
    try {
      await api.deleteProject(id);
      await refresh();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  };

  return (
    <div className="mx-auto max-w-5xl px-8 py-10">
      <header className="enki-gradient mb-8 rounded-lg border border-line p-8">
        <div className="flex items-start justify-between gap-4">
          <h1 className="text-xl font-semibold tracking-tight">
            ENKI
            <span className="ml-3 align-middle font-mono text-xs font-normal text-ink-faint">
              local-first AI development
            </span>
          </h1>
          <span className="shrink-0 rounded-full border border-warn/30 bg-warn/10 px-3 py-1 text-[11px] font-medium text-warn">
            Milestone 1 · setup only
          </span>
        </div>
        <p className="mt-2 max-w-xl text-sm leading-relaxed text-ink-dim">
          Private coding workspace: agents, models, and your projects — all on
          your machine. Credentials stay in the OS credential store; agent file
          access is scoped to each project.
        </p>
        <p className="mt-2 max-w-xl text-xs leading-relaxed text-ink-faint">
          This release configures providers and checks installed agent
          engines. The coding workspace — conversation, editor, terminal —
          opens in the next milestone.
        </p>
      </header>

      {error && (
        <div className="mb-6 rounded-md border border-danger/40 bg-danger/10 px-4 py-3 text-sm text-danger" role="alert">
          Backend error: {error}
        </div>
      )}

      <section aria-labelledby="projects-h" className="mb-10">
        <div className="mb-3 flex items-center justify-between">
          <h2 id="projects-h" className="text-sm font-semibold uppercase tracking-wider text-ink-dim">
            Projects
          </h2>
          <button
            onClick={() => setShowForm((v) => !v)}
            className="flex items-center gap-1.5 rounded-md border border-line bg-surface-1 px-3 py-1.5 text-xs text-ink-dim transition-colors hover:border-line-strong hover:text-ink"
          >
            <FolderPlus size={14} /> New project
          </button>
        </div>

        {showForm && (
          <form
            onSubmit={createProject}
            className="mb-4 rounded-md border border-line bg-surface-1 p-4"
          >
            <div className="grid grid-cols-[1fr_2fr_auto] gap-3">
              <label className="text-xs text-ink-dim">
                Name
                <input
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  placeholder="My app"
                  className="mt-1 w-full rounded-md border border-line bg-base px-3 py-1.5 text-sm text-ink placeholder:text-ink-faint"
                />
              </label>
              <label className="text-xs text-ink-dim">
                Absolute folder path
                <input
                  value={path}
                  onChange={(e) => setPath(e.target.value)}
                  placeholder="C:\dev\my-app"
                  className="mt-1 w-full rounded-md border border-line bg-base px-3 py-1.5 font-mono text-xs text-ink placeholder:text-ink-faint"
                />
              </label>
              <button
                type="submit"
                disabled={busy}
                className="mt-5 h-8 rounded-md bg-accent-strong px-4 text-xs font-medium text-white transition-colors hover:bg-accent disabled:opacity-50"
              >
                Add
              </button>
            </div>
            {formError && <p className="mt-2 text-xs text-danger">{formError}</p>}
          </form>
        )}

        {projects.length === 0 ? (
          <p className="rounded-md border border-dashed border-line px-4 py-6 text-center text-sm text-ink-faint">
            No projects yet. Add a project folder to begin.
          </p>
        ) : (
          <ul className="grid grid-cols-1 gap-2 sm:grid-cols-2">
            {projects.map((p) => (
              <li
                key={p.id}
                className="group flex items-center justify-between rounded-md border border-line bg-surface-1 px-4 py-3 transition-colors hover:border-line-strong"
              >
                <button
                  onClick={async () => {
                    try {
                      await api.openProject(p.id);
                      await refresh();
                    } catch (e) {
                      setError(e instanceof Error ? e.message : String(e));
                    }
                  }}
                  className="min-w-0 flex-1 text-left"
                >
                  <div className="truncate text-sm font-medium">{p.name}</div>
                  <div className="truncate font-mono text-xs text-ink-faint">
                    {p.root_path}
                  </div>
                </button>
                <button
                  onClick={() => removeProject(p.id, p.name)}
                  title="Remove project"
                  aria-label={`Remove project ${p.name}`}
                  className="ml-2 rounded p-1.5 text-ink-faint opacity-0 transition-opacity hover:bg-surface-2 hover:text-danger group-hover:opacity-100 focus-visible:opacity-100 focus-within:opacity-100"
                >
                  <Trash2 size={14} />
                </button>
              </li>
            ))}
          </ul>
        )}
      </section>

      <section aria-labelledby="sessions-h">
        <div className="mb-3 flex items-center justify-between">
          <h2 id="sessions-h" className="text-sm font-semibold uppercase tracking-wider text-ink-dim">
            Recent sessions
          </h2>
        </div>
        {sessions.length === 0 ? (
          <div className="rounded-md border border-dashed border-line px-4 py-6 text-center">
            <p className="text-sm text-ink-faint">
              No sessions yet. The coding workspace opens in the next milestone —
              agent sessions will appear here.
            </p>
            <button
              onClick={() => onNavigate("agents")}
              className="mt-3 inline-flex items-center gap-1.5 rounded-md border border-line bg-surface-1 px-3 py-1.5 text-xs text-ink-dim hover:border-line-strong hover:text-ink"
            >
              <Zap size={13} /> Check agent engines
            </button>
          </div>
        ) : (
          <ul className="space-y-1">
            {sessions.map((s) => (
              <li
                key={s.id}
                className="flex items-center gap-3 rounded-md border border-line bg-surface-1 px-4 py-2.5"
              >
                <MessageSquare size={14} className="shrink-0 text-ink-faint" />
                <span className="min-w-0 flex-1 truncate text-sm">{s.title}</span>
                <span className="shrink-0 rounded bg-surface-2 px-2 py-0.5 font-mono text-[10px] text-ink-dim">
                  {s.engine_id}
                </span>
                <span className={cn("shrink-0 font-mono text-[10px] text-ink-faint")}>
                  {s.updated_at}
                </span>
              </li>
            ))}
          </ul>
        )}
      </section>
    </div>
  );
}
