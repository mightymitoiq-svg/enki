import { useCallback, useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { FolderOpen, FolderPlus, MessageSquare, Trash2, Zap } from "lucide-react";
import { api } from "../lib/api";
import type { Project, Session } from "../lib/schemas";
import { cn } from "../lib/utils";
import Dialog from "../components/Dialog";

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
  const [notice, setNotice] = useState<string | null>(null);
  const [projectToRemove, setProjectToRemove] = useState<Project | null>(null);
  const [deleteBusy, setDeleteBusy] = useState(false);
  const [deleteError, setDeleteError] = useState<string | null>(null);
  const [folderBusy, setFolderBusy] = useState(false);

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
      setNotice("Project added.");
      await refresh();
    } catch (err) {
      setFormError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  };

  const removeProject = async (id: string) => {
    const project = projects.find((item) => item.id === id);
    if (project) {
      setProjectToRemove(project);
      setDeleteError(null);
    }
  };

  const confirmRemoveProject = async () => {
    if (!projectToRemove) return;
    setDeleteBusy(true);
    setDeleteError(null);
    try {
      await api.deleteProject(projectToRemove.id);
      setProjectToRemove(null);
      setNotice("Project removed from ENKI.");
      await refresh();
    } catch (e) {
      setDeleteError(e instanceof Error ? e.message : String(e));
    } finally {
      setDeleteBusy(false);
    }
  };

  const chooseFolder = async () => {
    setFolderBusy(true);
    setFormError(null);
    try {
      const selected = await open({ directory: true, multiple: false, title: "Choose a project folder" });
      if (typeof selected === "string") setPath(selected);
    } catch {
      setFormError("The folder picker could not open. You can enter the absolute path instead.");
    } finally {
      setFolderBusy(false);
    }
  };

  return (
    <div className="mx-auto max-w-5xl px-8 py-10">
      <header className="enki-gradient mb-8 overflow-hidden rounded-xl border border-line px-7 py-8 sm:px-9">
        <div className="mb-4 inline-flex items-center gap-2 rounded-full border border-line-strong bg-base/70 px-3 py-1 font-mono text-[10px] uppercase tracking-[0.14em] text-ink-dim">
          <span className="h-1.5 w-1.5 rounded-full bg-accent" /> Desktop foundation
        </div>
        <h1 className="max-w-2xl text-2xl font-semibold tracking-tight sm:text-3xl">
          A calmer place to set up your next build.
        </h1>
        <p className="mt-3 max-w-2xl text-sm leading-relaxed text-ink-dim">
          ENKI keeps your project list and provider settings on this device. This build helps you prepare a workspace and check installed engines; running coding sessions is still in development.
        </p>
      </header>

      {error && (
        <div className="mb-6 rounded-md border border-danger/40 bg-danger/10 px-4 py-3 text-sm text-danger" role="alert">
          Backend error: {error}
        </div>
      )}
      {notice && (
        <div role="status" className="mb-5 flex items-center justify-between rounded-md border border-ok/30 bg-ok/5 px-4 py-3 text-sm text-ok">
          {notice}
          <button type="button" onClick={() => setNotice(null)} className="ml-3 text-xs underline underline-offset-2">Dismiss</button>
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
            noValidate
            className="mb-4 rounded-md border border-line bg-surface-1 p-4"
          >
            <div className="grid grid-cols-1 gap-3 md:grid-cols-[minmax(8rem,1fr)_minmax(14rem,2fr)_auto_auto]">
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
                Project folder
                <input
                  value={path}
                  onChange={(e) => setPath(e.target.value)}
                  placeholder="Select a folder or enter its absolute path"
                  className="mt-1 w-full rounded-md border border-line bg-base px-3 py-1.5 font-mono text-xs text-ink placeholder:text-ink-faint"
                />
              </label>
              <button
                type="button"
                onClick={chooseFolder}
                disabled={folderBusy || busy}
                className="mt-5 inline-flex h-8 items-center justify-center gap-1.5 rounded-md border border-line px-3 text-xs text-ink-dim hover:border-line-strong hover:text-ink disabled:cursor-wait disabled:opacity-60"
              >
                <FolderOpen size={13} /> {folderBusy ? "Opening…" : "Browse"}
              </button>
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
                  onClick={() => removeProject(p.id)}
                  title="Remove project"
                  aria-label={`Remove ${p.name} from ENKI`}
                  className="ml-2 rounded p-1.5 text-ink-faint opacity-0 transition-opacity hover:bg-surface-2 hover:text-danger group-hover:opacity-100 group-focus-within:opacity-100"
                >
                  <Trash2 size={14} />
                </button>
              </li>
            ))}
          </ul>
        )}
      </section>

      <Dialog
        open={projectToRemove !== null}
        title="Remove project?"
        description={projectToRemove ? `“${projectToRemove.name}” and its saved session records will be removed from ENKI. The folder and its files will stay on your computer.` : undefined}
        onClose={() => setProjectToRemove(null)}
        footer={(
          <>
            <button type="button" onClick={() => setProjectToRemove(null)} disabled={deleteBusy} data-dialog-focus className="rounded-md border border-line px-4 py-2 text-sm text-ink-dim hover:bg-surface-2 disabled:opacity-50">Keep project</button>
            <button type="button" onClick={confirmRemoveProject} disabled={deleteBusy} className="rounded-md bg-danger px-4 py-2 text-sm font-medium text-white hover:brightness-110 disabled:opacity-60">{deleteBusy ? "Removing…" : "Remove project"}</button>
          </>
        )}
      >
        {deleteError && <p role="alert" className="text-sm text-danger">{deleteError}</p>}
      </Dialog>

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
