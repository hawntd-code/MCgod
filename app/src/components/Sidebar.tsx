import { ChevronRight, CircleAlert, Lock, Search } from "lucide-react";
import { useState } from "react";
import { useI18n } from "../i18n";
import type { WorkspaceSummary } from "../lib/engine";
import { groupByType, matchesFilter, needsReview } from "../lib/format";
import type { Activity } from "./ActivityRail";

type Props = {
  activity: Activity;
  workspace: WorkspaceSummary | null;
  activeElement: string | null;
  onOpenElement: (name: string) => void;
};

export function Sidebar({ activity, workspace, activeElement, onOpenElement }: Props) {
  const { m, lang } = useI18n();
  const c = m.sidebar[activity];
  const [filter, setFilter] = useState("");
  const [collapsed, setCollapsed] = useState<Set<string>>(new Set());
  const showTree = activity === "elements" && workspace !== null;

  const toggleGroup = (type: string) =>
    setCollapsed((prev) => {
      const next = new Set(prev);
      if (next.has(type)) next.delete(type);
      else next.add(type);
      return next;
    });

  const groups = showTree
    ? groupByType(workspace.elements.filter((e) => matchesFilter(e, filter)), m, lang)
    : [];

  return (
    <aside className="flex h-full flex-col bg-surface" aria-labelledby="sidebar-title">
      <div className="flex h-10 shrink-0 items-center justify-between px-4">
        <h2 id="sidebar-title" className="font-display text-h3 font-[650] [font-stretch:110%]">
          {c.title}
        </h2>
        {showTree && <span className="text-label tabular-nums text-ink-faint">{workspace.elements.length}</span>}
      </div>

      {activity === "elements" && (
        <div className="px-3 pb-2">
          <label className="flex h-8 items-center gap-2 rounded-ui border border-line bg-ground px-2.5 text-ink-faint focus-within:border-line-strong">
            <Search className="size-3.5 shrink-0" aria-hidden />
            <span className="sr-only">{m.sidebar.filter}</span>
            <input
              value={filter}
              onChange={(ev) => setFilter(ev.target.value)}
              disabled={!workspace}
              placeholder={m.sidebar.filter}
              className="w-full bg-transparent text-small text-ink outline-none placeholder:text-ink-faint disabled:cursor-not-allowed"
            />
          </label>
        </div>
      )}

      {showTree ? (
        <nav aria-label={c.title} className="min-h-0 flex-1 overflow-auto pb-4">
          {groups.length === 0 && <p className="px-4 py-3 text-small text-ink-muted">{m.sidebar.noMatches}</p>}
          {groups.map((g) => {
            const open = !collapsed.has(g.type) || filter.trim() !== "";
            return (
              <div key={g.type} className="mt-1">
                <button
                  type="button"
                  onClick={() => toggleGroup(g.type)}
                  aria-expanded={open}
                  className="flex w-full items-center gap-1.5 px-3 py-1 text-label font-semibold uppercase tracking-[0.04em] text-ink-faint transition-colors duration-150 hover:text-ink"
                >
                  <ChevronRight
                    className={`size-3.5 shrink-0 transition-transform duration-150 ${open ? "rotate-90" : ""}`}
                    aria-hidden
                  />
                  <span className="truncate">{g.label}</span>
                  <span className="ml-auto tabular-nums font-normal">{g.elements.length}</span>
                </button>
                {open && (
                  <ul>
                    {g.elements.map((e) => {
                      const current = e.name === activeElement;
                      const label = [e.name, e.lockedCode && m.sidebar.locked, needsReview(e) && m.sidebar.needsReview]
                        .filter(Boolean)
                        .join(", ");
                      return (
                        <li key={e.name}>
                          <button
                            type="button"
                            onClick={() => onOpenElement(e.name)}
                            aria-label={label}
                            title={e.registryName ?? undefined}
                            aria-current={current ? "page" : undefined}
                            className="flex w-full items-center gap-2 py-1.5 pl-8 pr-3 text-left transition-colors duration-150 hover:bg-hover aria-[current=page]:bg-raised"
                          >
                            <span className="min-w-0 flex-1">
                              <span className="block truncate text-small text-ink">{e.name}</span>
                              {e.registryName && (
                                <span className="block truncate font-mono text-[0.6875rem] text-ink-faint">
                                  {e.registryName}
                                </span>
                              )}
                            </span>
                            {e.lockedCode && <Lock className="size-3.5 shrink-0 text-ink-faint" aria-hidden />}
                            {needsReview(e) && <CircleAlert className="size-3.5 shrink-0 text-warn" aria-hidden />}
                          </button>
                        </li>
                      );
                    })}
                  </ul>
                )}
              </div>
            );
          })}
        </nav>
      ) : (
        <div className="flex flex-1 flex-col justify-center px-6 pb-16">
          <h3 className="text-h3 font-semibold text-ink">{c.emptyTitle}</h3>
          <p className="mt-2 text-small text-ink-muted">{c.emptyText}</p>
          <p className="mt-4 text-label text-ink-faint">{c.phase}</p>
        </div>
      )}
    </aside>
  );
}
