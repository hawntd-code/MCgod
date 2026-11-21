import { Search } from "lucide-react";
import { useI18n } from "../i18n";
import type { Activity } from "./ActivityRail";

export function Sidebar({ activity }: { activity: Activity }) {
  const { m } = useI18n();
  const c = m.sidebar[activity];

  return (
    <aside className="flex h-full flex-col bg-surface" aria-labelledby="sidebar-title">
      <div className="flex h-10 shrink-0 items-center px-4">
        <h2 id="sidebar-title" className="font-display text-h3 font-[650] [font-stretch:110%]">
          {c.title}
        </h2>
      </div>

      {activity === "elements" && (
        <div className="px-3 pb-3">
          <label className="flex h-8 items-center gap-2 rounded-ui border border-line bg-ground px-2.5 text-ink-faint">
            <Search className="size-3.5 shrink-0" aria-hidden />
            <span className="sr-only">{m.sidebar.filter}</span>
            <input
              disabled
              placeholder={m.sidebar.filter}
              className="w-full bg-transparent text-small text-ink outline-none placeholder:text-ink-faint disabled:cursor-not-allowed"
            />
          </label>
        </div>
      )}

      <div className="flex flex-1 flex-col justify-center px-6 pb-16">
        <h3 className="text-h3 font-semibold text-ink">{c.emptyTitle}</h3>
        <p className="mt-2 text-small text-ink-muted">{c.emptyText}</p>
        <p className="mt-4 text-label text-ink-faint">{c.phase}</p>
      </div>
    </aside>
  );
}
