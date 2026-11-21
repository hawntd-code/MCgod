import { invoke, isTauri } from "@tauri-apps/api/core";
import { act, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { App } from "../App";
import { I18nProvider, type Lang } from "../i18n";

vi.mock("@tauri-apps/api/core", () => ({ isTauri: vi.fn(), invoke: vi.fn() }));
vi.mock("react-resizable-panels", () => import("../test/panels-mock"));

function renderApp(lang: Lang = "es") {
  return render(
    <I18nProvider lang={lang}>
      <App />
    </I18nProvider>,
  );
}

const panelButton = (key: "sidebar" | "console" | "ai") =>
  document.querySelector<HTMLButtonElement>(`button[data-panel="${key}"]`)!;

describe("App", () => {
  beforeEach(() => {
    vi.mocked(isTauri).mockReturnValue(false);
    vi.mocked(invoke).mockReset();
  });

  it("muestra la bienvenida, el panel de elementos, el asistente y la consola", async () => {
    renderApp();
    expect(screen.getByRole("heading", { level: 1, name: "Crea tu primer mod." })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Elementos" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Asistente" })).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: "Consola de Gradle" })).toHaveAttribute("aria-selected", "true");
    // Fuera de Tauri, la bienvenida explica que el motor no está conectado.
    expect(await screen.findByText(/Abre la app de escritorio/)).toBeInTheDocument();
  });

  it("con el motor conectado muestra la versión de Java en la bienvenida y la barra de estado", async () => {
    vi.mocked(isTauri).mockReturnValue(true);
    vi.mocked(invoke).mockResolvedValue({
      appVersion: "0.1.0",
      os: "Windows x86_64",
      java: { version: "21.0.13.1", major: 21, vendor: "Temurin" },
    });
    renderApp();
    expect(await screen.findByText("Temurin 21.0.13.1")).toBeInTheDocument();
    expect(screen.getByTestId("java-status")).toHaveTextContent("Java 21.0.13.1");
    expect(screen.getByText("v0.1.0")).toBeInTheDocument();
  });

  it("los botones de compilar y probar están desactivados sin proyecto", () => {
    renderApp();
    expect(screen.getByRole("button", { name: /Compilar/ })).toBeDisabled();
    expect(screen.getByRole("button", { name: /Probar en Minecraft/ })).toBeDisabled();
  });

  it.each([
    ["sidebar", "b"],
    ["console", "j"],
    ["ai", "l"],
  ] as const)("Ctrl+%s oculta y vuelve a mostrar el panel %s", async (key, letter) => {
    const user = userEvent.setup();
    renderApp();
    expect(panelButton(key)).toHaveAttribute("aria-pressed", "true");
    await user.keyboard(`{Control>}${letter}{/Control}`);
    await waitFor(() => expect(panelButton(key)).toHaveAttribute("aria-pressed", "false"));
    await user.keyboard(`{Control>}${letter}{/Control}`);
    await waitFor(() => expect(panelButton(key)).toHaveAttribute("aria-pressed", "true"));
  });

  it("los botones de la barra superior también ocultan los paneles", async () => {
    const user = userEvent.setup();
    renderApp();
    await user.click(panelButton("ai"));
    await waitFor(() => expect(panelButton("ai")).toHaveAttribute("aria-pressed", "false"));
  });

  it("la barra lateral cambia de sección y se pliega al tocar la sección activa", async () => {
    const user = userEvent.setup();
    renderApp();
    const nav = screen.getByRole("navigation", { name: "Secciones" });

    await user.click(within(nav).getByRole("button", { name: "Base de conocimiento" }));
    expect(screen.getByRole("heading", { level: 2, name: "Base de conocimiento" })).toBeInTheDocument();

    await user.click(within(nav).getByRole("button", { name: "Base de conocimiento" }));
    await waitFor(() => expect(panelButton("sidebar")).toHaveAttribute("aria-pressed", "false"));

    await user.click(within(nav).getByRole("button", { name: "Variables" }));
    await waitFor(() => expect(panelButton("sidebar")).toHaveAttribute("aria-pressed", "true"));
    expect(screen.getByRole("heading", { level: 2, name: "Variables" })).toBeInTheDocument();
  });

  it("la consola cambia entre Gradle y la actividad de la IA", async () => {
    const user = userEvent.setup();
    renderApp();
    await user.click(screen.getByRole("tab", { name: "Actividad de la IA" }));
    expect(screen.getByRole("tabpanel")).toHaveTextContent(/registro de cada herramienta/);
  });

  it("cambia todo a inglés, lo recuerda y actualiza el idioma del documento", async () => {
    const user = userEvent.setup();
    renderApp("es");
    expect(document.documentElement.lang).toBe("es");

    await user.click(screen.getByRole("button", { name: "English" }));

    expect(screen.getByRole("heading", { level: 1, name: "Make your first mod." })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Assistant" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "English" })).toHaveAttribute("aria-pressed", "true");
    expect(document.documentElement.lang).toBe("en");
    expect(localStorage.getItem("mcgod:lang")).toBe("en");

    await user.click(screen.getByRole("button", { name: "Español" }));
    expect(screen.getByRole("heading", { level: 1, name: "Crea tu primer mod." })).toBeInTheDocument();
  });

  it("los atajos de teclado no reaccionan sin Ctrl ni con Ctrl+Shift", async () => {
    renderApp();
    await act(async () => {
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "b" }));
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "b", ctrlKey: true, shiftKey: true }));
    });
    expect(panelButton("sidebar")).toHaveAttribute("aria-pressed", "true");
  });
});
