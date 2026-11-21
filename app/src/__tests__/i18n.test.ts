import { afterEach, describe, expect, it, vi } from "vitest";
import { initialLang } from "../i18n";
import { en } from "../i18n/locales/en";
import { es } from "../i18n/locales/es";
import { collectStrings, shapeOf } from "../test/strings";

describe("textos de la interfaz", () => {
  it("español e inglés tienen exactamente las mismas claves, listas y funciones", () => {
    expect(shapeOf(en)).toEqual(shapeOf(es));
  });

  it.each([
    ["es", es],
    ["en", en],
  ])("%s: ningún texto vacío, con espacios sobrantes ni marcas de borrador", (_lang, messages) => {
    for (const { path, text } of collectStrings(messages)) {
      expect(text.trim(), path).not.toBe("");
      expect(text, path).toBe(text.trim());
      // Mayúsculas exactas: "todo" es una palabra normal en español.
      expect(text, path).not.toMatch(/\b(TODO|FIXME|XXX)\b|lorem ipsum/);
      expect(text, path).not.toMatch(/ {2,}/);
    }
  });

  it("todo texto largo está traducido (no es igual en los dos idiomas)", () => {
    // Palabras que se escriben igual en ambos idiomas.
    const sameInBoth = new Set(["elementTypes.bescript"]);
    const esTexts = new Map(collectStrings(es).map((s) => [s.path, s.text]));
    const untranslated = collectStrings(en).filter(
      ({ path, text }) => text.length > 12 && esTexts.get(path) === text && !sameInBoth.has(path),
    );
    expect(untranslated).toEqual([]);
  });

  it("el inglés no arrastra letras del español", () => {
    const leaks = collectStrings(en).filter(
      ({ path, text }) => path !== "language.es" && /[áéíóúñ¿¡]/i.test(text),
    );
    expect(leaks).toEqual([]);
  });
});

describe("idioma inicial", () => {
  afterEach(() => vi.restoreAllMocks());

  it("usa español si el sistema está en español", () => {
    vi.spyOn(navigator, "language", "get").mockReturnValue("es-MX");
    expect(initialLang()).toBe("es");
  });

  it("usa inglés para cualquier otro idioma del sistema", () => {
    vi.spyOn(navigator, "language", "get").mockReturnValue("pt-BR");
    expect(initialLang()).toBe("en");
  });

  it("respeta el idioma que la persona eligió antes", () => {
    vi.spyOn(navigator, "language", "get").mockReturnValue("es-MX");
    localStorage.setItem("mcgod:lang", "en");
    expect(initialLang()).toBe("en");
  });

  it("ignora un valor guardado que no es un idioma válido", () => {
    vi.spyOn(navigator, "language", "get").mockReturnValue("en-US");
    localStorage.setItem("mcgod:lang", "fr");
    expect(initialLang()).toBe("en");
  });
});
