import { useI18n } from "../i18n";
import type { CodegenEvent } from "../lib/engine";
import { typeLabel } from "../lib/format";
import type { LineTone } from "../lib/gradle-run";

type Line = { tone: LineTone; text: string };

/** Líneas de consola para un aviso de la generación de código, en el idioma de la persona. */
export function codegenLines(event: CodegenEvent, m: ReturnType<typeof useI18n>["m"]): Line[] {
  const c = m.codegen;
  if (event.stage === "indexing") return [{ tone: "task", text: c.indexing }];
  if (event.stage === "generating" || !event.report) return [{ tone: "task", text: c.generating }];

  const r = event.report;
  if (r.errors.length > 0) {
    return [
      { tone: "error", text: c.failed(r.errors.length) },
      ...r.errors.map((e) => ({ tone: "error" as const, text: `  ${e}` })),
    ];
  }

  const out: Line[] = [
    {
      tone: "ok",
      text: r.generated.length > 0 ? c.generated(r.files, r.written, r.generated.join(", ")) : c.generatedNoElements(r.files, r.written),
    },
  ];
  const named = (kind: string) =>
    r.skipped
      .filter((s) => s.reason.kind === kind)
      .map((s) => `${s.name} (${typeLabel(s.elementType, m)})`)
      .join(", ");
  const pending = named("unsupported");
  if (pending) out.push({ tone: "warn", text: c.pending(pending) });
  const missing = named("notInGenerator");
  if (missing) out.push({ tone: "warn", text: c.notInGenerator(missing) });
  const locked = r.skipped.filter((s) => s.reason.kind === "locked").map((s) => s.name);
  if (locked.length > 0) out.push({ tone: "plain", text: c.locked(locked.join(", ")) });
  for (const s of r.skipped) {
    if (s.reason.kind === "noDefinition") out.push({ tone: "warn", text: c.noDefinition(s.name, s.reason.detail) });
  }
  for (const n of r.notes) {
    const text = c.notes[n.code] ?? n.code;
    out.push({ tone: "warn", text: n.detail ? `${text} ${n.detail}` : text });
  }
  return out;
}
