import { useEffect, useState } from "react";
import {
  Bot,
  Home,
  Package,
  Settings as SettingsIcon,
  ShieldCheck,
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

  useEffect(() => {
    api.appVersion().then(setVersion).catch(() => setVersion("?"));
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
      {/* Compact icon navigation */}
      <nav
        aria-label="Primary"
        className="flex w-14 shrink-0 flex-col items-center gap-1 border-r border-line bg-surface py-3"
      >
        <div
          className="mb-3 flex h-8 w-8 items-center justify-center rounded-md bg-gradient-to-br from-accent to-violet"
          aria-label="ENKI"
        >
          <span className="select-none font-mono text-[11px] font-bold text-base">
            E
          </span>
        </div>
        {NAV_ITEMS.map((item, i) => (
          <button
            key={item.id}
            onClick={() => setPage(item.id)}
            title={`${item.label} (Alt+${i + 1})`}
            aria-current={page === item.id ? "page" : undefined}
            className={cn(
              "group relative flex h-9 w-9 items-center justify-center rounded-md transition-colors",
              page === item.id
                ? "bg-surface-2 text-ink"
                : "text-ink-faint hover:bg-surface-1 hover:text-ink-dim",
            )}
          >
            <item.icon size={17} strokeWidth={1.8} />
            {page === item.id && (
              <span className="absolute left-0 top-1/2 h-4 w-0.5 -translate-y-1/2 rounded-r bg-accent" />
            )}
          </button>
        ))}
        <div className="mt-auto flex flex-col items-center gap-2">
          <span
            className="text-ink-faint"
            title="Security: project-scoped filesystem access"
          >
            <ShieldCheck size={16} strokeWidth={1.8} />
          </span>
          <span className="font-mono text-[9px] text-ink-faint">v{version}</span>
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
