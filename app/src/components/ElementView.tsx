import { CircleAlert, Lock } from "lucide-react";
import { useEffect, useState } from "react";
import { useI18n } from "../i18n";
import { useBackend } from "../lib/backend-context";
import { toEngineError, type ElementDefinition, type ElementSummary, type EngineError } from "../lib/engine";
import { formatField, needsReview, typeLabel } from "../lib/format";
import { delay } from "../lib/motion";
import { errorMessage } from "./ErrorNotice";

type Load = { status: "loading" } | { status: "ready"; def: ElementDefinition } | { status: "error"; error: EngineError };

/** Vista de solo lectura de un elemento: sus datos tal como los guarda MCreator. */
export function ElementView({ element }: { element: ElementSummary }) {
  const { m } = useI18n();
  const e = m.element;
  const backend = useBackend();
  const [load, setLoad] = useState<Load>({ status: "loading" });
  const [raw, setRaw] = useState(false);

  useEffect(() => {
    let alive = true;
    setLoad({ status: "loading" });
    backend
      .readElement(element.name)
      .then((def) => alive && setLoad({ status: "ready", def }))
      .catch((err) => alive && setLoad({ status: "error", error: toEngineError(err) }));
    return () => {
      alive = false;
    };
  }, [backend, element.name]);

  const notes: { icon: typeof Lock; text: string; tone: "warn" | "muted" }[] = [];
  if (element.lockedCode) notes.push({ icon: Lock, text: e.locked, tone: "muted" });
  if (needsReview(element)) {
    const reasons = [!element.hasDefinition && m.workspace.missingDefinition, !element.compiles && m.workspace.notCompiling];
    notes.push({ icon: CircleAlert, text: reasons.filter(Boolean).join(" · "), tone: "warn" });
  }

  return (
    <article className="mx-auto max-w-[1080px] px-[clamp(1.5rem,4vw,3rem)] py-10" aria-labelledby="element-title">
      <div className="animate-enter">
        <p className="text-small font-medium text-brand-bright">{typeLabel(element.type, m)}</p>
        <h1
          id="element-title"
          className="mt-1 font-display text-[2.25rem] font-[800] leading-none tracking-[-0.025em] [font-stretch:116%]"
        >
          {element.name}
        </h1>
      </div>

      <dl className="mt-5 flex animate-enter flex-wrap gap-x-8 gap-y-2 text-small" style={delay(40)}>
        <div className="flex gap-2">
          <dt className="text-ink-faint">{e.registryName}</dt>
          <dd className="font-mono text-ink">{element.registryName ?? "—"}</dd>
        </div>
        {element.folder && (
          <div className="flex gap-2">
            <dt className="text-ink-faint">{e.folder}</dt>
            <dd className="text-ink">{element.folder}</dd>
          </div>
        )}
        {load.status === "ready" && (
          <div className="flex animate-fade gap-2">
            <dd className="text-ink-faint">{e.format(String(load.def._fv))}</dd>
          </div>
        )}
      </dl>

      {notes.length > 0 && (
        <ul className="mt-4 animate-enter space-y-1.5" style={delay(60)}>
          {notes.map((n) => (
            <li key={n.text} className={`flex items-center gap-2 text-small ${n.tone === "warn" ? "text-warn" : "text-ink-muted"}`}>
              <n.icon className="size-4 shrink-0" aria-hidden />
              {n.text}
            </li>
          ))}
        </ul>
      )}

      <div className="mt-8 flex animate-enter items-center justify-between border-b border-line pb-2" style={delay(80)}>
        <h2 className="font-display text-h3 font-[680] [font-stretch:110%]">{e.fields}</h2>
        {load.status === "ready" && (
          <button
            type="button"
            onClick={() => setRaw((v) => !v)}
            aria-pressed={raw}
            className="h-7 animate-fade rounded-ui px-2.5 text-small text-ink-muted transition-colors duration-150 hover:bg-hover hover:text-ink aria-pressed:bg-raised aria-pressed:text-ink"
          >
            {raw ? e.showFields : e.showRaw}
          </button>
        )}
      </div>

      {/* El aviso de carga espera un instante antes de verse: una lectura rápida no parpadea */}
      {load.status === "loading" && (
        <p className="mt-4 animate-fade text-small text-ink-muted" style={delay(150)}>
          {e.loading}
        </p>
      )}

      {load.status === "error" && (
        <p className="mt-4 animate-fade text-small text-ink-muted" role="status">
          {element.type === "code" && load.error.code === "noDefinition" ? e.noDefinitionType : errorMessage(load.error, m)}
        </p>
      )}

      {load.status === "ready" &&
        (raw ? (
          <pre className="mt-4 max-h-[60vh] animate-fade overflow-auto rounded-ui border border-line bg-ground p-4 font-mono text-label leading-relaxed text-ink-muted select-text">
            {JSON.stringify(load.def, null, 2)}
          </pre>
        ) : (
          <>
            {/* La tabla entra completa: con bordes colapsados, animar filas sueltas deja las líneas solas */}
            <table className="mt-2 w-full animate-fade text-small" style={delay(60)}>
              <tbody className="divide-y divide-line">
                {Object.entries(load.def.definition).map(([key, value]) => {
                  const f = formatField(value, e);
                  return (
                    <tr key={key} className="align-top">
                      <th scope="row" className="w-[38%] py-2 pr-4 text-left font-mono font-normal text-ink-muted">
                        {key}
                      </th>
                      <td
                        className={`py-2 select-text ${
                          f.kind === "json"
                            ? "break-all font-mono text-label text-ink-muted"
                            : f.kind === "empty"
                              ? "text-ink-faint"
                              : f.kind === "number"
                                ? "tabular-nums text-ink"
                                : "text-ink"
                        }`}
                      >
                        {f.text}
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
            <p className="mt-6 text-label text-ink-faint">{e.fieldsNote}</p>
          </>
        ))}
    </article>
  );
}
