import { useState } from "react";

const tabs = [
  {
    id: "gradle",
    label: "Consola de Gradle",
    empty: "Sin compilaciones todavía. Cuando compiles o abras Minecraft, la salida aparecerá aquí en vivo.",
  },
  {
    id: "ai",
    label: "Actividad de la IA",
    empty: "Aquí quedará el registro de cada herramienta que use la IA: qué cambió, en qué elemento y con qué resultado.",
  },
] as const;

export function ConsolePanel() {
  const [tab, setTab] = useState<(typeof tabs)[number]["id"]>("gradle");
  const current = tabs.find((t) => t.id === tab)!;

  return (
    <section className="flex h-full flex-col bg-ground">
      <div role="tablist" className="flex h-9 shrink-0 items-center gap-1 border-b border-line px-2">
        {tabs.map((t) => (
          <button
            key={t.id}
            type="button"
            role="tab"
            aria-selected={tab === t.id}
            onClick={() => setTab(t.id)}
            className="h-7 rounded-ui px-2.5 text-small text-ink-faint transition-colors duration-150 hover:text-ink aria-selected:bg-raised aria-selected:text-ink"
          >
            {t.label}
          </button>
        ))}
      </div>
      <div role="tabpanel" className="min-h-0 flex-1 overflow-auto px-4 py-3 font-mono text-[0.8125rem] leading-relaxed">
        <p className="text-ink-faint">{current.empty}</p>
      </div>
    </section>
  );
}
