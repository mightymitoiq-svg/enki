import { useEffect, useState } from "react";
import {
  Bot,
  Home,
  Package,
  PanelLeftClose,
  PanelLeftOpen,
  Settings as SettingsIcon,
} from "lucide-react";
import HomePage from "./pages/HomePage";
import ProvidersPage from "./pages/ProvidersPage";
import AgentsPage from "./pages/AgentsPage";
import SettingsPage from "./pages/SettingsPage";
import { api } from "./lib/api";
import { cn } from "./lib/utils";

type PageId = "home" | "providers" | "agents" | "settings";

const NAV_ITEMS: {
  id: PageId;
  label: string;
  icon: typeof Home;
}[] = [
  { id: "home", label: "Home", icon: Home },
  { id: "providers", label: "Providers", icon: Package },
  { id: "agents", label: "Agents", icon: Bot },
  { id: "settings", label: "Settings", icon: SettingsIcon },
];

export default function App() {
  const [page, setPage] = useState<PageId>("home");
  const [version, setVersion] = useState<string>("");
  const [expanded, setExpanded] = useState(() => {
    try {
      return window.localStorage.getItem("enki.sidebar.expanded") !== "false";
    } catch {
      return true;
    }
  });

  useEffect(() => {
    api.appVersion().then(setVersion).catch(() => setVersion("?"));
    api.getSettings().then((settings) => {
      document.documentElement.dataset.theme = settings.theme;
    }).catch(() => {
      document.documentElement.dataset.theme = "dark";
    });
  }, []);

  useEffect(() => {
    try {
      window.localStorage.setItem("enki.sidebar.expanded", String(expanded));
    } catch {
      // The layout remains usable when browser storage is unavailable.
    }
  }, [expanded]);

  useEffect(() => {
    const onSettingsSaved = (event: Event) => {
      const settings = (event as CustomEvent<{ theme: "dark" | "light" }>).detail;
      if (settings?.theme) document.documentElement.dataset.theme = settings.theme;
    };
    window.addEventListener("enki:settings-saved", onSettingsSaved);
    return () => window.removeEventListener("enki:settings-saved", onSettingsSaved);
  }, []);

  // Keyboard navigation: Alt+1..4 switch pages
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.altKey && !e.ctrlKey && !e.metaKey) {
        const idx = parseInt(e.key, 10) - 1;
        if (idx >= 0 && idx < NAV_ITEMS.length) {
          e.preventDefault();
          setPage(NAV_ITEMS[idx].id);
        }
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  return (
    <div className="flex h-full w-full overflow-hidden bg-base">
      {/* Workspace navigation */}
      <nav
        aria-label="Primary"
        className={cn(
          "flex shrink-0 flex-col border-r border-line bg-surface py-4 transition-[width] duration-200",
          expanded ? "w-60 px-3" : "w-[4.5rem] items-center px-2",
        )}
      >
        <div className={cn("mb-7 flex items-center", expanded ? "gap-3 px-2" : "justify-center")}>
          <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-md bg-gradient-to-br from-accent to-violet">
            <span className="select-none font-mono text-xs font-bold text-base">E</span>
          </div>
          {expanded && (
            <div className="min-w-0">
              <div className="text-sm font-semibold tracking-[0.18em]">ENKI</div>
              <div className="font-mono text-[9px] uppercase tracking-[0.16em] text-ink-faint">Local development</div>
            </div>
          )}
        </div>
        {expanded && <div className="mb-2 px-3 font-mono text-[9px] uppercase tracking-[0.18em] text-ink-faint">Workspace</div>}
        {NAV_ITEMS.map((item, i) => (
          <button
            key={item.id}
            onClick={() => setPage(item.id)}
            title={`${item.label} (Alt+${i + 1})`}
            aria-label={`${item.label} (Alt+${i + 1})`}
            aria-current={page === item.id ? "page" : undefined}
            className={cn(
              "group relative mb-1 flex h-10 w-full items-center rounded-md transition-colors",
              expanded ? "gap-3 px-3" : "justify-center",
              page === item.id
                ? "bg-surface-2 text-ink"
                : "text-ink-dim hover:bg-surface-1 hover:text-ink",
            )}
          >
            <item.icon size={17} strokeWidth={1.8} aria-hidden="true" />
            {expanded && <span className="text-sm">{item.label}</span>}
            {expanded && <span className="ml-auto font-mono text-[10px] text-ink-faint">Alt+{i + 1}</span>}
            {page === item.id && (
              <span className="absolute left-0 top-1/2 h-5 w-0.5 -translate-y-1/2 rounded-r bg-accent" />
            )}
          </button>
        ))}
        <div className={cn("mt-auto flex items-center border-t border-line pt-4", expanded ? "justify-between px-2" : "flex-col gap-3")}>
          {expanded && (
            <span className="max-w-[9rem] text-[11px] leading-relaxed text-ink-faint">
              Agent sessions are not available in this build.
            </span>
          )}
          <span className="font-mono text-[10px] text-ink-faint">v{version}</span>
        </div>
        <div className={cn("mt-3 flex", expanded ? "justify-end" : "justify-center")}>
          <button
            type="button"
            onClick={() => setExpanded((value) => !value)}
            aria-label={expanded ? "Collapse sidebar" : "Expand sidebar"}
            title={expanded ? "Collapse sidebar" : "Expand sidebar"}
            aria-expanded={expanded}
            className="rounded-md p-2 text-ink-dim hover:bg-surface-1 hover:text-ink"
          >
            {expanded ? <PanelLeftClose size={16} /> : <PanelLeftOpen size={16} />}
          </button>
        </div>
      </nav>

      <main className="min-w-0 flex-1 overflow-y-auto">
        {page === "home" && <HomePage onNavigate={setPage} />}
        {page === "providers" && <ProvidersPage />}
        {page === "agents" && <AgentsPage />}
        {page === "settings" && <SettingsPage />}
      </main>
    </div>
  );
}
