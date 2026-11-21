import { ArrowUp, Check } from "lucide-react";
import { useState } from "react";

const providers = [
  { id: "anthropic", name: "Anthropic", detail: "Modelos Claude · clave de API" },
  { id: "openai", name: "OpenAI", detail: "Modelos GPT · clave de API" },
  { id: "gemini", name: "Google Gemini", detail: "Clave de API de Google AI Studio" },
  { id: "ollama", name: "Ollama", detail: "Modelos locales en tu PC · sin clave" },
  { id: "compatible", name: "Compatible con OpenAI", detail: "OpenRouter, LM Studio y otros" },
] as const;

const abilities = [
  "Crear y editar elementos del mod",
  "Compilar y abrir Minecraft para probar",
  "Leer la consola y corregir errores",
  "Consultar la base de conocimiento",
];

export function AiPanel() {
  const [picked, setPicked] = useState<string | null>(null);

  return (
    <aside className="flex h-full flex-col bg-surface">
      <div className="flex h-10 shrink-0 items-center justify-between border-b border-line px-4">
        <h2 className="font-display text-h3 font-[650] [font-stretch:110%]">Asistente</h2>
        <span className="text-label text-ink-faint">Sin proveedor</span>
      </div>

      <div className="min-h-0 flex-1 overflow-auto px-4 py-5">
        <h3 className="font-display text-h2 font-[720] [font-stretch:115%]">Conecta tu IA</h3>
        <p className="mt-2 text-small text-ink-muted">
          Usa tu propia clave. La IA trabaja con las mismas herramientas que tú, y cada cambio que hace aparece en vivo
          en el proyecto.
        </p>

        <ul className="mt-5 divide-y divide-line border-y border-line" role="radiogroup" aria-label="Proveedor de IA">
          {providers.map((p) => {
            const on = picked === p.id;
            return (
              <li key={p.id}>
                <button
                  type="button"
                  role="radio"
                  aria-checked={on}
                  onClick={() => setPicked(p.id)}
                  className="flex w-full items-center gap-3 px-1 py-2.5 text-left transition-colors duration-150 hover:bg-hover aria-checked:bg-raised"
                >
                  <span
                    className={`grid size-4 shrink-0 place-items-center rounded-full border ${on ? "border-brand bg-brand text-brand-ink" : "border-line-strong"}`}
                    aria-hidden
                  >
                    {on && <Check className="size-3" strokeWidth={3} />}
                  </span>
                  <span className="min-w-0">
                    <span className="block text-body font-medium leading-tight text-ink">{p.name}</span>
                    <span className="block text-label text-ink-muted">{p.detail}</span>
                  </span>
                </button>
              </li>
            );
          })}
        </ul>

        {picked && (
          <p className="mt-3 text-small text-ink-muted" role="status">
            La conexión con proveedores llega en la fase 2. La clave se guardará en el almacén seguro de Windows, nunca
            dentro del proyecto.
          </p>
        )}

        <h3 className="mt-7 text-small font-semibold text-ink">Qué podrá hacer</h3>
        <ul className="mt-2 space-y-1.5">
          {abilities.map((a) => (
            <li key={a} className="flex gap-2 text-small text-ink-muted">
              <span className="mt-[0.55em] size-1 shrink-0 bg-brand" aria-hidden />
              {a}
            </li>
          ))}
        </ul>
      </div>

      <div className="shrink-0 border-t border-line p-3">
        <div className="flex items-end gap-2 rounded-ui border border-line bg-ground p-2">
          <label className="sr-only" htmlFor="ai-input">
            Mensaje para la IA
          </label>
          <textarea
            id="ai-input"
            disabled
            rows={2}
            placeholder="Conecta un proveedor para escribirle a la IA"
            className="min-h-0 flex-1 resize-none bg-transparent text-small text-ink outline-none placeholder:text-ink-faint disabled:cursor-not-allowed"
          />
          <button
            type="button"
            disabled
            aria-label="Enviar"
            className="grid size-7 shrink-0 place-items-center rounded-ui bg-brand text-brand-ink disabled:cursor-not-allowed disabled:opacity-40"
          >
            <ArrowUp className="size-4" strokeWidth={2.25} aria-hidden />
          </button>
        </div>
      </div>
    </aside>
  );
}
