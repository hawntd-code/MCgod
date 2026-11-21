import { ArrowUp, Check } from "lucide-react";
import { useState } from "react";
import { useI18n } from "../i18n";

export const providerIds = ["anthropic", "openai", "gemini", "deepseek", "ollama", "compatible"] as const;
export type ProviderId = (typeof providerIds)[number];

/** Nombres de marca: no se traducen, salvo la opción genérica. */
const brandNames: Record<Exclude<ProviderId, "compatible">, string> = {
  anthropic: "Anthropic",
  openai: "OpenAI",
  gemini: "Google Gemini",
  deepseek: "DeepSeek",
  ollama: "Ollama",
};

export function AiPanel() {
  const { m } = useI18n();
  const a = m.ai;
  const [picked, setPicked] = useState<ProviderId | null>(null);

  return (
    <aside className="flex h-full flex-col bg-surface" aria-labelledby="ai-title">
      <div className="flex h-10 shrink-0 items-center justify-between border-b border-line px-4">
        <h2 id="ai-title" className="font-display text-h3 font-[650] [font-stretch:110%]">
          {a.title}
        </h2>
        <span className="text-label text-ink-faint">{a.noProvider}</span>
      </div>

      <div className="min-h-0 flex-1 overflow-auto px-4 py-5">
        <h3 className="font-display text-h2 font-[720] [font-stretch:115%]">{a.connectTitle}</h3>
        <p className="mt-2 text-small text-ink-muted">{a.connectText}</p>

        <ul className="mt-5 divide-y divide-line border-y border-line" role="radiogroup" aria-label={a.providersLabel}>
          {providerIds.map((id) => {
            const on = picked === id;
            const name = id === "compatible" ? a.compatibleName : brandNames[id];
            return (
              <li key={id}>
                <button
                  type="button"
                  role="radio"
                  aria-checked={on}
                  onClick={() => setPicked(id)}
                  className="flex w-full items-center gap-3 px-1 py-2.5 text-left transition-colors duration-150 hover:bg-hover aria-checked:bg-raised"
                >
                  <span
                    className={`grid size-4 shrink-0 place-items-center rounded-full border ${on ? "border-brand bg-brand text-brand-ink" : "border-line-strong"}`}
                    aria-hidden
                  >
                    {on && <Check className="size-3" strokeWidth={3} />}
                  </span>
                  <span className="min-w-0">
                    <span className="block text-body font-medium leading-tight text-ink">{name}</span>
                    <span className="block text-label text-ink-muted">{a.providers[id]}</span>
                  </span>
                </button>
              </li>
            );
          })}
        </ul>

        {picked && (
          <p className="mt-3 text-small text-ink-muted" role="status">
            {a.pickedNote}
          </p>
        )}

        <h3 className="mt-7 text-small font-semibold text-ink">{a.abilitiesTitle}</h3>
        <ul className="mt-2 space-y-1.5">
          {a.abilities.map((ability) => (
            <li key={ability} className="flex gap-2 text-small text-ink-muted">
              <span className="mt-[0.55em] size-1 shrink-0 bg-brand" aria-hidden />
              {ability}
            </li>
          ))}
        </ul>
      </div>

      <div className="shrink-0 border-t border-line p-3">
        <div className="flex items-end gap-2 rounded-ui border border-line bg-ground p-2">
          <label className="sr-only" htmlFor="ai-input">
            {a.inputLabel}
          </label>
          <textarea
            id="ai-input"
            disabled
            rows={2}
            placeholder={a.inputPlaceholder}
            className="min-h-0 flex-1 resize-none bg-transparent text-small text-ink outline-none placeholder:text-ink-faint disabled:cursor-not-allowed"
          />
          <button
            type="button"
            disabled
            aria-label={a.send}
            className="grid size-7 shrink-0 place-items-center rounded-ui bg-brand text-brand-ink disabled:cursor-not-allowed disabled:opacity-40"
          >
            <ArrowUp className="size-4" strokeWidth={2.25} aria-hidden />
          </button>
        </div>
      </div>
    </aside>
  );
}
