import type { Messages } from "../i18n/locales/es";
import type { ElementSummary, GeneratorId } from "./engine";

const FLAVORS: Record<string, string> = {
  neoforge: "NeoForge",
  forge: "Forge",
  fabric: "Fabric",
  quilt: "Quilt",
  datapack: "Data Pack",
  resourcepack: "Resource Pack",
  addon: "Bedrock Add-On",
};

/** `neoforge-1.21.1` → `NeoForge 1.21.1`. */
export function generatorLabel(generator: GeneratorId | null): string | null {
  if (!generator) return null;
  const flavor = FLAVORS[generator.flavor] ?? generator.flavor;
  return generator.minecraft ? `${flavor} ${generator.minecraft}` : flavor;
}

export function typeLabel(type: string, m: Messages): string {
  return m.elementTypes[type] ?? type;
}

export type ElementGroup = { type: string; label: string; elements: ElementSummary[] };

/** Agrupa los elementos por tipo; grupos ordenados por nombre visible y elementos por nombre. */
export function groupByType(elements: ElementSummary[], m: Messages, lang: string): ElementGroup[] {
  const groups = new Map<string, ElementSummary[]>();
  for (const e of elements) {
    const list = groups.get(e.type) ?? [];
    list.push(e);
    groups.set(e.type, list);
  }
  const collator = new Intl.Collator(lang, { sensitivity: "base" });
  return [...groups.entries()]
    .map(([type, list]) => ({
      type,
      label: typeLabel(type, m),
      elements: [...list].sort((a, b) => collator.compare(a.name, b.name)),
    }))
    .sort((a, b) => collator.compare(a.label, b.label));
}

export function needsReview(e: ElementSummary): boolean {
  return !e.hasDefinition || !e.compiles;
}

export function matchesFilter(e: ElementSummary, filter: string): boolean {
  const q = filter.trim().toLowerCase();
  if (!q) return true;
  return e.name.toLowerCase().includes(q) || (e.registryName ?? "").toLowerCase().includes(q);
}

/** Valores tipo MCreator `{ "value": "X" }` (ítems, pestañas, sonidos) se muestran como X. */
function unwrapValue(v: unknown): unknown {
  if (v && typeof v === "object" && !Array.isArray(v)) {
    const keys = Object.keys(v);
    if (keys.length === 1 && (keys[0] === "value" || keys[0] === "texture")) return (v as Record<string, unknown>)[keys[0]];
  }
  return v;
}

export type FieldValue = { kind: "text" | "number" | "bool" | "empty" | "list" | "json"; text: string };

/** Convierte el valor de un campo de MCreator en algo legible. */
export function formatField(raw: unknown, m: Messages["element"]): FieldValue {
  const v = unwrapValue(raw);
  if (v === null || v === undefined || v === "") return { kind: "empty", text: m.empty };
  if (typeof v === "boolean") return { kind: "bool", text: v ? m.yes : m.no };
  if (typeof v === "number") return { kind: "number", text: String(v) };
  if (typeof v === "string") return { kind: "text", text: v };
  if (Array.isArray(v)) {
    if (v.length === 0) return { kind: "empty", text: m.empty };
    const items = v.map(unwrapValue);
    if (items.every((x) => typeof x === "string" || typeof x === "number")) {
      return { kind: "list", text: compactRepeats(items.map(String)).join(", ") };
    }
  }
  return { kind: "json", text: JSON.stringify(v) };
}

/** `[a, a, a, b]` → `["a ×3", "b"]`: las recetas repiten el mismo ítem en varias casillas. */
function compactRepeats(items: string[]): string[] {
  const out: string[] = [];
  let i = 0;
  while (i < items.length) {
    let j = i;
    while (j < items.length && items[j] === items[i]) j++;
    out.push(j - i > 1 ? `${items[i]} ×${j - i}` : items[i]);
    i = j;
  }
  return out;
}
