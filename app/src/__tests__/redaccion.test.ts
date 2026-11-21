import { readdirSync, readFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { describe, expect, it } from "vitest";
import { en } from "../i18n/locales/en";
import { es } from "../i18n/locales/es";
import { collectStrings } from "../test/strings";

// Vitest corre desde app/; la raíz del repositorio está un nivel arriba.
const repo = resolve(process.cwd(), "..");

type Rule = { patron: string; motivo: string };
const list: { es: Rule[]; en: Rule[] } = JSON.parse(
  readFileSync(join(repo, "style", "frases-prohibidas.json"), "utf8"),
);
const rules = [...list.es, ...list.en].map((r) => ({
  ...r,
  // Bordes de palabra que entienden acentos (\b de JavaScript no los entiende).
  regex: new RegExp(`(?<![\\p{L}\\p{N}])(?:${r.patron})(?![\\p{L}\\p{N}])`, "iu"),
}));

function findBanned(text: string) {
  return rules.flatMap((r) => {
    const hit = text.match(r.regex);
    return hit ? [{ frase: hit[0], motivo: r.motivo }] : [];
  });
}

/** Quita bloques de código, código en línea y URLs: no son prosa. */
function proseOf(markdown: string) {
  return markdown
    .replace(/```[\s\S]*?```/g, "")
    .replace(/`[^`]*`/g, "")
    .replace(/\]\([^)]*\)/g, "]")
    .replace(/https?:\/\/\S+/g, "");
}

const GUIDE = "05-guia-de-redaccion.md";
const docs = [
  "README.md",
  "NOTICE.md",
  ...readdirSync(join(repo, "docs"))
    .filter((f) => f.endsWith(".md") && f !== GUIDE)
    .map((f) => `docs/${f}`),
];

describe("guía de redacción", () => {
  it("el detector encuentra frases prohibidas en español e inglés", () => {
    for (const bad of [
      "Cabe destacar que el mod compila.",
      "Lleva tu mod al siguiente nivel.",
      "Una experiencia fluida para todos.",
      "Es importante mencionar los requisitos.",
      "En resumen, funciona.",
      "Let's delve into modding.",
      "A seamless workflow.",
      "Unlock the full potential of your mod.",
    ]) {
      expect(findBanned(bad), bad).not.toEqual([]);
    }
  });

  it("el detector no marca usos legítimos", () => {
    for (const ok of [
      "La potencia de la señal de redstone llega a 15.",
      "Compila el mod y ábrelo en Minecraft.",
      "El bloque tiene una dureza de 3.",
      "Build the mod and launch Minecraft.",
      "Necesitas Java 21.",
    ]) {
      expect(findBanned(ok), ok).toEqual([]);
    }
  });

  it.each([
    ["español", es],
    ["inglés", en],
  ])("los textos de la interfaz en %s no usan frases prohibidas", (_lang, messages) => {
    const hits = collectStrings(messages).flatMap(({ path, text }) =>
      findBanned(text).map((h) => ({ path, ...h })),
    );
    expect(hits).toEqual([]);
  });

  it.each(docs)("%s no usa frases prohibidas", (file) => {
    const lines = proseOf(readFileSync(join(repo, file), "utf8")).split("\n");
    const hits = lines.flatMap((line, i) => findBanned(line).map((h) => ({ linea: i + 1, ...h })));
    expect(hits).toEqual([]);
  });
});
