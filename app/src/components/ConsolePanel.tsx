import { useState } from "react";
import { useI18n } from "../i18n";

type Tab = "gradle" | "ai";

export function ConsolePanel() {
  const { m } = useI18n();
  const [tab, setTab] = useState<Tab>("gradle");
  const tabs: { id: Tab; label: string; empty: string }[] = [
    { id: "gradle", label: m.console.gradle, empty: m.console.gradleEmpty },
    { id: "ai", label: m.console.ai, empty: m.console.aiEmpty },
  ];
  const current = tabs.find((t) => t.id === tab)!;

  return (
    <section className="flex h-full flex-col bg-ground">
      <div role="tablist" className="flex h-9 shrink-0 items-center gap-1 border-b border-line px-2">
        {tabs.map((t) => (
          <button
            key={t.id}
            id={`console-tab-${t.id}`}
            type="button"
            role="tab"
            aria-selected={tab === t.id}
            aria-controls="console-panel"
            onClick={() => setTab(t.id)}
            className="h-7 rounded-ui px-2.5 text-small text-ink-faint transition-colors duration-150 hover:text-ink aria-selected:bg-raised aria-selected:text-ink"
          >
            {t.label}
          </button>
        ))}
      </div>
      <div
        id="console-panel"
        role="tabpanel"
        aria-labelledby={`console-tab-${tab}`}
        className="min-h-0 flex-1 overflow-auto px-4 py-3 font-mono text-[0.8125rem] leading-relaxed"
      >
        <p className="text-ink-faint">{current.empty}</p>
      </div>
    </section>
  );
}
