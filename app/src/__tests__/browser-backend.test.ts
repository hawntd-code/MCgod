import { describe, expect, it } from "vitest";
import { DEMO_FILE } from "../lib/demo";
import { createBrowserBackend, toEngineError } from "../lib/engine";

describe("backend del navegador", () => {
  it("empieza con la bienvenida pendiente y la recuerda al terminarla", async () => {
    const b = createBrowserBackend(true);
    expect(await b.settings()).toEqual({ onboardingDone: false, recent: [] });
    await b.finishOnboarding();
    expect((await createBrowserBackend(true).settings()).onboardingDone).toBe(true);
  });

  it("abrir el proyecto de ejemplo lo agrega a recientes sin duplicarlo", async () => {
    const b = createBrowserBackend(true);
    await b.openWorkspace(DEMO_FILE);
    await b.openWorkspace(DEMO_FILE);
    const { recent } = await b.settings();
    expect(recent).toHaveLength(1);
    expect(recent[0]).toMatchObject({ path: DEMO_FILE, name: "Ruby Mod", modid: "rubymod" });
  });

  it("sin modo demo, abrir proyectos pide la app de escritorio", async () => {
    const b = createBrowserBackend(false);
    await expect(b.pickWorkspace({ title: "", filter: "" })).rejects.toMatchObject({ code: "desktopOnly" });
    await expect(b.openWorkspace("C:/x.mcreator")).rejects.toMatchObject({ code: "desktopOnly" });
  });

  it("distingue un elemento sin definición de uno que no existe", async () => {
    const b = createBrowserBackend(true);
    await expect(b.readElement("RubyPickaxe")).rejects.toMatchObject({ code: "noDefinition" });
    await expect(b.readElement("NoExiste")).rejects.toMatchObject({ code: "unknownElement" });
    expect((await b.readElement("Ruby"))._type).toBe("item");
  });

  it("devuelve copias: cambiar el resultado no altera los datos de ejemplo", async () => {
    const b = createBrowserBackend(true);
    const first = await b.readElement("Ruby");
    first.definition.rarity = "EPIC";
    expect((await b.readElement("Ruby")).definition.rarity).toBe("UNCOMMON");
  });

  it("normaliza errores del motor y errores sueltos", () => {
    expect(toEngineError({ code: "notFound", detail: "C:/x" })).toEqual({ code: "notFound", detail: "C:/x" });
    expect(toEngineError(new Error("boom"))).toEqual({ code: "internal", detail: "Error: boom" });
    expect(toEngineError("texto")).toEqual({ code: "internal", detail: "texto" });
  });
});
