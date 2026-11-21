import { invoke, isTauri } from "@tauri-apps/api/core";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { App } from "../App";
import { I18nProvider } from "../i18n";
import { createBrowserBackend, type Backend, type GradleEvent } from "../lib/engine";
import { formatDuration, toneOf } from "../lib/gradle-run";

vi.mock("@tauri-apps/api/core", () => ({ isTauri: vi.fn(), invoke: vi.fn(), Channel: class {} }));
vi.mock("react-resizable-panels", () => import("../test/panels-mock"));

beforeEach(() => {
  vi.mocked(isTauri).mockReturnValue(false);
  vi.mocked(invoke).mockReset();
  localStorage.setItem("mcgod:browser-settings", JSON.stringify({ onboardingDone: true, recent: [] }));
});

async function openProject(backend: Backend = createBrowserBackend(true)) {
  const user = userEvent.setup();
  render(
    <I18nProvider lang="es">
      <App backend={backend} />
    </I18nProvider>,
  );
  await user.click(await screen.findByRole("button", { name: "Abrir proyecto de MCreator" }));
  await screen.findByRole("heading", { level: 1, name: "Ruby Mod" });
  return user;
}

describe("colores de la consola", () => {
  it.each([
    ["BUILD SUCCESSFUL in 1m 48s", "ok"],
    ["BUILD FAILED in 12s", "error"],
    ["FAILURE: Build failed with an exception.", "error"],
    ["C:\\mods\\src\\main\\java\\Ruby.java:12: error: cannot find symbol", "error"],
    ["[Render thread/ERROR] [minecraft/Main]: crash", "error"],
    ["\tat net.minecraft.client.Main.main(Main.java:220)", "error"],
    ["[Render thread/WARN] [minecraft/ShaderInstance]: Shader ...", "warn"],
    ["warning: [removal] foo() is deprecated", "warn"],
    ["> Task :compileJava", "task"],
    ["[Render thread/INFO] Sound engine started", "plain"],
  ])("%s → %s", (text, tone) => {
    expect(toneOf(text)).toBe(tone);
  });

  it("formatea la duración en segundos o minutos", () => {
    expect(formatDuration(9_400)).toBe("9 s");
    expect(formatDuration(108_700)).toBe("1 min 49 s");
    expect(formatDuration(125_300)).toBe("2 min 05 s");
    expect(formatDuration(-5)).toBe("0 s");
  });
});

describe("compilar y probar en Minecraft", () => {
  it("sin proyecto los botones están desactivados y explican por qué", async () => {
    render(
      <I18nProvider lang="es">
        <App backend={createBrowserBackend(true)} />
      </I18nProvider>,
    );
    await screen.findByRole("heading", { name: "Proyectos" });
    expect(screen.getByRole("button", { name: /Compilar/ })).toBeDisabled();
    expect(screen.getByRole("button", { name: /Probar en Minecraft/ })).toHaveAttribute("title", "Abre un proyecto primero");
  });

  it("un proyecto sin archivos de Gradle no se puede compilar", async () => {
    const backend = createBrowserBackend(true);
    const open = backend.openWorkspace;
    backend.openWorkspace = async (p) => ({ ...(await open(p)), hasGradle: false });
    await openProject(backend);
    expect(screen.getByRole("button", { name: /Probar en Minecraft/ })).toBeDisabled();
    expect(screen.getByRole("button", { name: /Compilar/ })).toHaveAttribute(
      "title",
      expect.stringContaining("no tiene archivos de Gradle"),
    );
  });

  it("compila, muestra la salida en vivo con colores y el resultado", async () => {
    const user = await openProject();
    await user.click(screen.getByRole("button", { name: "Compilar" }));

    // Mientras corre: botón ocupado, opción de detener y aviso en la barra de estado.
    expect(await screen.findByRole("button", { name: /Compilando/ })).toBeDisabled();
    expect(screen.getByText("Gradle en curso")).toBeInTheDocument();

    const output = await screen.findByTestId("gradle-output");
    await waitFor(() => expect(within(output).getByText("BUILD SUCCESSFUL in 1m 48s")).toBeInTheDocument(), {
      timeout: 3000,
    });
    expect(within(output).getByText("BUILD SUCCESSFUL in 1m 48s")).toHaveClass("text-ok");
    expect(within(output).getByText("> Task :compileJava")).toHaveClass("text-ink-faint");

    const status = await screen.findByText(/Compilar: correcto en/);
    expect(status).toBeInTheDocument();
    expect(screen.queryByText("Gradle en curso")).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Compilar" })).toBeEnabled();
  });

  it("antes de compilar genera el código y resume qué se generó y qué falta", async () => {
    const user = await openProject();
    await user.click(screen.getByRole("button", { name: "Compilar" }));
    const output = await screen.findByTestId("gradle-output");
    const mcgod = () => within(output).getAllByText((_, el) => el?.getAttribute("data-source") === "mcgod");
    await waitFor(() => expect(mcgod().length).toBeGreaterThan(3));
    const texts = mcgod().map((el) => el.textContent);
    expect(texts[0]).toBe("MCgod · Generando el código del mod…");
    expect(texts).toContain("MCgod · Código generado: 9 archivos, 0 con cambios. Elementos: Ruby.");
    expect(texts).toContain(
      "MCgod · Todavía sin generar (MCgod aún no genera estos tipos): RubyBlock (Bloque), RubyOre (Bloque), RubyPickaxe (Herramienta), RubyBlockRecipe (Receta).",
    );
    expect(texts).toContain("MCgod · Código bloqueado, se conserva como está: GlowOnStep, RubyUtils.");
    expect(texts).toContain("MCgod · Las variables globales todavía no se generan.");
    const summary = mcgod().find((el) => el.textContent?.includes("Código generado"));
    expect(summary).toHaveClass("text-ok");
  });

  it("si una plantilla falla, la consola lo dice y la tarea termina como fallida", async () => {
    const backend = createBrowserBackend(true);
    backend.runGradle = async (_task, onEvent) => {
      onEvent({ kind: "codegen", stage: "generating" });
      onEvent({
        kind: "codegen",
        stage: "generated",
        report: { files: 0, written: 0, unchanged: 0, generated: [], skipped: [], notes: [], errors: ["Ruby: item/item.java.ftl, línea 3: falta x"] },
      });
      onEvent({ kind: "finished", outcome: "failed", exitCode: null, durationMs: 10 });
    };
    const user = await openProject(backend);
    await user.click(screen.getByRole("button", { name: "Compilar" }));
    const output = await screen.findByTestId("gradle-output");
    const failed = await within(output).findByText(/No se pudo generar el código: falló 1 plantilla/);
    expect(failed.closest("div")).toHaveClass("text-error");
    expect(within(output).getByText(/item\/item\.java\.ftl, línea 3/)).toBeInTheDocument();
    expect(await screen.findByText(/Compilar: falló/)).toBeInTheDocument();
  });

  it("abre Minecraft y lo detiene", async () => {
    const user = await openProject();
    await user.click(screen.getByRole("button", { name: /Probar en Minecraft/ }));
    const stop = await screen.findByRole("button", { name: /Minecraft en curso… · Detener/ });
    expect(screen.getByRole("button", { name: "Compilar" })).toBeDisabled();
    await user.click(stop);
    expect(await screen.findByText("Minecraft: detenido")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Probar en Minecraft/ })).toBeEnabled();
  });

  it("si Gradle no puede arrancar, explica el motivo en la consola", async () => {
    const backend = createBrowserBackend(true);
    backend.runGradle = async () => {
      throw { code: "javaMissing", detail: "java no está en el PATH" };
    };
    const user = await openProject(backend);
    await user.click(screen.getByRole("button", { name: "Compilar" }));
    expect(
      await screen.findByText("Compilar: no se pudo iniciar: No se encontró Java. Instala un JDK 21 para compilar mods."),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Compilar" })).toBeEnabled();
  });

  it("muestra el código de salida cuando la compilación falla", async () => {
    const backend = createBrowserBackend(true);
    backend.runGradle = async (_task, onEvent) => {
      onEvent({ kind: "lines", lines: [{ stream: "stderr", text: "Ruby.java:3: error: ';' expected" }] });
      onEvent({ kind: "finished", outcome: "failed", exitCode: 1, durationMs: 4000 });
    };
    const user = await openProject(backend);
    await user.click(screen.getByRole("button", { name: "Compilar" }));
    expect(await screen.findByText("Compilar: falló (código 1)")).toBeInTheDocument();
    expect(screen.getByText("Ruby.java:3: error: ';' expected")).toHaveClass("text-error");
  });

  it("guarda solo las últimas líneas cuando la salida es enorme", async () => {
    const backend = createBrowserBackend(true);
    backend.runGradle = async (_task, onEvent) => {
      const many: GradleEvent = {
        kind: "lines",
        lines: Array.from({ length: 4500 }, (_, i) => ({ stream: "stdout" as const, text: `linea ${i + 1}` })),
      };
      onEvent(many);
      onEvent({ kind: "finished", outcome: "success", exitCode: 0, durationMs: 1000 });
    };
    const user = await openProject(backend);
    await user.click(screen.getByRole("button", { name: "Compilar" }));
    expect(await screen.findByText("500 líneas anteriores ocultas para no saturar la consola")).toBeInTheDocument();
    expect(screen.queryByText("linea 1")).not.toBeInTheDocument();
    expect(screen.getByText("linea 4500")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Limpiar" }));
    expect(screen.queryByTestId("gradle-output")).not.toBeInTheDocument();
  });

  it("si la consola estaba oculta, se muestra al compilar", async () => {
    const user = await openProject();
    const consoleToggle = document.querySelector<HTMLButtonElement>('button[data-panel="console"]')!;
    await user.click(consoleToggle);
    await waitFor(() => expect(consoleToggle).toHaveAttribute("aria-pressed", "false"));
    await user.click(screen.getByRole("button", { name: "Compilar" }));
    await waitFor(() => expect(consoleToggle).toHaveAttribute("aria-pressed", "true"));
  });

  it("cerrar el proyecto detiene Gradle", async () => {
    const backend = createBrowserBackend(true);
    const stop = vi.spyOn(backend, "stopGradle");
    const user = await openProject(backend);
    await user.click(screen.getByRole("button", { name: /Probar en Minecraft/ }));
    // Se puede detener desde la barra superior y desde la consola.
    expect(await screen.findAllByRole("button", { name: /Detener/ })).toHaveLength(2);
    await user.click(screen.getByRole("button", { name: "Cerrar proyecto" }));
    expect(stop).toHaveBeenCalled();
    expect(await screen.findByRole("heading", { name: "Proyectos" })).toBeInTheDocument();
  });
});
