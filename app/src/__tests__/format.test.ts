import { describe, expect, it } from "vitest";
import { en } from "../i18n/locales/en";
import { es } from "../i18n/locales/es";
import { demoSummary } from "../lib/demo";
import { formatField, generatorLabel, groupByType, matchesFilter, needsReview, typeLabel } from "../lib/format";

describe("formato", () => {
  it("nombra los generadores como los conoce la comunidad", () => {
    expect(generatorLabel({ id: "neoforge-1.21.1", flavor: "neoforge", minecraft: "1.21.1" })).toBe("NeoForge 1.21.1");
    expect(generatorLabel({ id: "datapack-26.3.x", flavor: "datapack", minecraft: "26.3.x" })).toBe("Data Pack 26.3.x");
    expect(generatorLabel({ id: "addon-26.5x", flavor: "addon", minecraft: "26.5x" })).toBe("Bedrock Add-On 26.5x");
    expect(generatorLabel({ id: "raro", flavor: "raro", minecraft: null })).toBe("raro");
    expect(generatorLabel(null)).toBeNull();
  });

  it("traduce todos los tipos de elemento de MCreator y deja pasar los desconocidos", () => {
    for (const type of ["item", "block", "livingentity", "procedure", "beitem", "code", "rangeditem"]) {
      expect(typeLabel(type, es)).not.toBe(type);
      expect(typeLabel(type, en)).not.toBe(type);
    }
    expect(typeLabel("tipo_de_un_plugin", es)).toBe("tipo_de_un_plugin");
  });

  it("agrupa por tipo y ordena alfabéticamente según el idioma", () => {
    const groups = groupByType(demoSummary.elements, es, "es");
    expect(groups.map((g) => g.label)).toEqual([
      "Bloque",
      "Código personalizado",
      "Herramienta",
      "Objeto",
      "Procedure",
      "Receta",
    ]);
    expect(groups[0].elements.map((e) => e.name)).toEqual(["RubyBlock", "RubyOre"]);
  });

  it("marca para revisar lo que no tiene definición o no compila", () => {
    const flagged = demoSummary.elements.filter(needsReview).map((e) => e.name);
    expect(flagged).toEqual(["RubyPickaxe"]);
  });

  it("el filtro ignora mayúsculas y espacios", () => {
    const ore = demoSummary.elements.find((e) => e.name === "RubyOre")!;
    expect(matchesFilter(ore, "  rubyore ")).toBe(true);
    expect(matchesFilter(ore, "RUBY_ORE")).toBe(true);
    expect(matchesFilter(ore, "pickaxe")).toBe(false);
    expect(matchesFilter(ore, "")).toBe(true);
  });

  it("muestra los valores de MCreator de forma legible", () => {
    const m = es.element;
    expect(formatField(true, m)).toEqual({ kind: "bool", text: "Sí" });
    expect(formatField(false, en.element)).toEqual({ kind: "bool", text: "No" });
    expect(formatField(1.2, m)).toEqual({ kind: "number", text: "1.2" });
    expect(formatField("", m)).toEqual({ kind: "empty", text: "(vacío)" });
    expect(formatField(null, m).kind).toBe("empty");
    expect(formatField([], m).kind).toBe("empty");
    expect(formatField({ value: "METAL" }, m)).toEqual({ kind: "text", text: "METAL" });
    expect(formatField({ texture: "ruby" }, m)).toEqual({ kind: "text", text: "ruby" });
    expect(formatField([{ value: "A" }, { value: "A" }, { value: "B" }], m)).toEqual({ kind: "list", text: "A ×2, B" });
    expect(formatField({ x: 1, y: 2 }, m)).toEqual({ kind: "json", text: '{"x":1,"y":2}' });
  });
});
