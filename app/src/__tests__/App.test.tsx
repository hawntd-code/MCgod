import { invoke, isTauri } from "@tauri-apps/api/core";
import { act, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { App } from "../App";
import { I18nProvider, type Lang } from "../i18n";
import { createBrowserBackend, type Backend } from "../lib/engine";

vi.mock("@tauri-apps/api/core", () => ({ isTauri: vi.fn(), invoke: vi.fn() }));
vi.mock("react-resizable-panels", () => import("../test/panels-mock"));

/** Ajustes del navegador como si la persona ya hubiera pasado la bienvenida. */
function returningUser() {
  localStorage.setItem("mcgod:browser-settings", JSON.stringify({ onboardingDone: true, recent: [] }));
}

function renderApp(lang: Lang = "es", backend?: Backend) {
  return render(
    <I18nProvider lang={lang}>
      <App backend={backend} />
    </I18nProvider>,
  );
}

const panelButton = (key: "sidebar" | "console" | "ai") =>
  document.querySelector<HTMLButtonElement>(`button[data-panel="${key}"]`)!;

async function openDemoProject(user: ReturnType<typeof userEvent.setup>) {
  await user.click(await screen.findByRole("button", { name: "Abrir proyecto de MCreator" }));
  await screen.findByRole("heading", { level: 1, name: "Ruby Mod" });
}

beforeEach(() => {
  vi.mocked(isTauri).mockReturnValue(false);
  vi.mocked(invoke).mockReset();
});

describe("primer arranque", () => {
  it("la primera vez muestra la bienvenida en lugar de la app", async () => {
    renderApp();
    expect(await screen.findByRole("heading", { level: 1, name: "Crea tu primer mod." })).toBeInTheDocument();
    expect(screen.queryByRole("heading", { name: "Proyectos" })).not.toBeInTheDocument();
    expect(screen.getByText(/Abre la app de escritorio/)).toBeInTheDocument();
  });

  it("al pulsar Empezar entra a la app y no vuelve a mostrar la bienvenida", async () => {
    const user = userEvent.setup();
    const { unmount } = renderApp();
    await user.click(await screen.findByRole("button", { name: /Empezar/ }));
    expect(await screen.findByRole("heading", { level: 1, name: "Proyectos" })).toBeInTheDocument();

    unmount();
    renderApp();
    expect(await screen.findByRole("heading", { level: 1, name: "Proyectos" })).toBeInTheDocument();
    expect(screen.queryByRole("heading", { name: "Crea tu primer mod." })).not.toBeInTheDocument();
  });

  it("desde la bienvenida se puede elegir el idioma", async () => {
    const user = userEvent.setup();
    renderApp("es");
    await user.click(await screen.findByRole("button", { name: "English" }));
    expect(screen.getByRole("heading", { level: 1, name: "Make your first mod." })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Get started/ })).toBeInTheDocument();
  });

  it("con el motor conectado muestra la versión de Java en los requisitos", async () => {
    vi.mocked(isTauri).mockReturnValue(true);
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "environment")
        return { appVersion: "0.1.0", os: "Windows x86_64", java: { version: "21.0.13.1", major: 21, vendor: "Temurin" } };
      throw new Error(`comando inesperado ${cmd}`);
    });
    // Los ajustes vienen del backend inyectado; el entorno, del motor simulado.
    renderApp("es", createBrowserBackend(true));
    expect(await screen.findByText("Temurin 21.0.13.1")).toBeInTheDocument();
  });
});

describe("proyectos", () => {
  beforeEach(returningUser);

  it("sin proyecto abierto muestra la pantalla de proyectos con la lista de recientes vacía", async () => {
    renderApp();
    expect(await screen.findByRole("heading", { level: 1, name: "Proyectos" })).toBeInTheDocument();
    expect(screen.getByText("Los proyectos que abras aparecerán aquí.")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Nuevo proyecto/ })).toBeDisabled();
    expect(screen.getByTestId("project-name")).toHaveTextContent("Sin proyecto abierto");
  });

  it("abre un proyecto y muestra su resumen, el árbol y la barra de estado", async () => {
    const user = userEvent.setup();
    renderApp();
    await openDemoProject(user);

    expect(screen.getByTestId("project-name")).toHaveTextContent("Ruby Mod");
    expect(screen.getByTestId("status-project")).toHaveTextContent("rubymod");
    expect(screen.getByTestId("status-generator")).toHaveTextContent("NeoForge 1.21.1");
    expect(screen.getByText("7 elementos")).toBeInTheDocument();

    // Resumen: datos del mod y elementos para revisar.
    expect(screen.getByText("Workspace de ejemplo para las pruebas de MCgod")).toBeInTheDocument();
    const review = screen.getByRole("region", { name: "Para revisar" });
    expect(within(review).getByText("RubyPickaxe")).toBeInTheDocument();
    expect(within(review).getByText(/Falta su archivo de definición · MCreator lo marcó como que no compila/)).toBeInTheDocument();

    // Árbol agrupado por tipo con nombres traducidos.
    const tree = screen.getByRole("navigation", { name: "Elementos" });
    expect(within(tree).getByRole("button", { name: /Bloque/, expanded: true })).toBeInTheDocument();
    expect(within(tree).getByRole("button", { name: /RubyOre/ })).toBeInTheDocument();
  });

  it("recuerda el proyecto en recientes y lo vuelve a abrir desde ahí", async () => {
    const user = userEvent.setup();
    renderApp();
    await openDemoProject(user);
    await user.click(screen.getByRole("button", { name: "Cerrar proyecto" }));

    const recent = await screen.findByRole("list", { name: "Recientes" });
    expect(within(recent).getByText("Ruby Mod")).toBeInTheDocument();
    await user.click(within(recent).getByRole("button", { name: /^Ruby Mod/ }));
    expect(await screen.findByRole("heading", { level: 1, name: "Ruby Mod" })).toBeInTheDocument();
  });

  it("quita un proyecto de recientes", async () => {
    const user = userEvent.setup();
    renderApp();
    await openDemoProject(user);
    await user.click(screen.getByRole("button", { name: "Cerrar proyecto" }));
    await user.click(await screen.findByRole("button", { name: "Quitar Ruby Mod de recientes" }));
    expect(await screen.findByText("Los proyectos que abras aparecerán aquí.")).toBeInTheDocument();
  });

  it("si cancela el selector de archivos no pasa nada", async () => {
    const user = userEvent.setup();
    const backend = createBrowserBackend(true);
    backend.pickWorkspace = async () => null;
    renderApp("es", backend);
    await user.click(await screen.findByRole("button", { name: "Abrir proyecto de MCreator" }));
    expect(screen.getByRole("heading", { level: 1, name: "Proyectos" })).toBeInTheDocument();
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("explica el error si el archivo no es un workspace válido, con el detalle técnico", async () => {
    const user = userEvent.setup();
    const backend = createBrowserBackend(true);
    backend.openWorkspace = async () => {
      throw { code: "invalidJson", detail: "C:/mods/roto.mcreator: expected value at line 1" };
    };
    renderApp("es", backend);
    await user.click(await screen.findByRole("button", { name: "Abrir proyecto de MCreator" }));
    const alert = await screen.findByRole("alert");
    expect(alert).toHaveTextContent("El archivo no tiene el formato de un workspace de MCreator.");
    expect(within(alert).getByTestId("error-detail")).toHaveTextContent("roto.mcreator");
    await user.click(within(alert).getByRole("button", { name: "Cerrar aviso" }));
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("en el navegador sin modo demo explica que abrir proyectos es de la app de escritorio", async () => {
    const user = userEvent.setup();
    renderApp("es", createBrowserBackend(false));
    await user.click(await screen.findByRole("button", { name: "Abrir proyecto de MCreator" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("solo funciona en la app de escritorio");
  });
});

describe("elementos", () => {
  beforeEach(returningUser);

  it("abre un elemento en una pestaña y muestra sus campos legibles", async () => {
    const user = userEvent.setup();
    renderApp();
    await openDemoProject(user);
    const tree = screen.getByRole("navigation", { name: "Elementos" });
    await user.click(within(tree).getByRole("button", { name: "Ruby" }));

    expect(await screen.findByRole("heading", { level: 1, name: "Ruby" })).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: "Ruby", selected: true })).toBeInTheDocument();
    const row = (key: string) => screen.getByRole("rowheader", { name: key }).closest("tr")!;
    expect(row("rarity")).toHaveTextContent("UNCOMMON");
    expect(row("immuneToFire")).toHaveTextContent("Sí");
    expect(row("creativeTabs")).toHaveTextContent("INGREDIENTS");
    expect(row("texture")).toHaveTextContent("ruby");
    expect(screen.getByText("Formato 92")).toBeInTheDocument();
  });

  it("agrupa las casillas repetidas de una receta", async () => {
    const user = userEvent.setup();
    renderApp();
    await openDemoProject(user);
    await user.click(within(screen.getByRole("navigation", { name: "Elementos" })).getByRole("button", { name: /RubyBlockRecipe/ }));
    const slots = (await screen.findByRole("rowheader", { name: "recipeSlots" })).closest("tr")!;
    expect(slots).toHaveTextContent("CUSTOM:Ruby ×9");
  });

  it("muestra el JSON completo al pedirlo", async () => {
    const user = userEvent.setup();
    renderApp();
    await openDemoProject(user);
    await user.click(within(screen.getByRole("navigation", { name: "Elementos" })).getByRole("button", { name: /RubyOre/ }));
    await user.click(await screen.findByRole("button", { name: "Ver JSON" }));
    expect(screen.getByText(/"maxGenerateHeight": 16/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Ver campos" })).toHaveAttribute("aria-pressed", "true");
  });

  it("un elemento sin definición explica qué pasa en lugar de fallar", async () => {
    const user = userEvent.setup();
    renderApp();
    await openDemoProject(user);
    // Desde la lista "Para revisar" del resumen.
    await user.click(within(screen.getByRole("region", { name: "Para revisar" })).getByRole("button", { name: /RubyPickaxe/ }));
    expect(await screen.findByRole("status")).toHaveTextContent("no tiene archivo de definición");
  });

  it("el código personalizado aclara que su contenido son archivos Java", async () => {
    const user = userEvent.setup();
    renderApp();
    await openDemoProject(user);
    await user.click(within(screen.getByRole("navigation", { name: "Elementos" })).getByRole("button", { name: /RubyUtils/ }));
    expect(await screen.findByRole("status")).toHaveTextContent("su contenido son archivos Java");
    expect(screen.getByText(/Código bloqueado: MCreator no regenera/)).toBeInTheDocument();
  });

  it("las pestañas se cierran y vuelven a la anterior o al resumen", async () => {
    const user = userEvent.setup();
    renderApp();
    await openDemoProject(user);
    const tree = screen.getByRole("navigation", { name: "Elementos" });
    await user.click(within(tree).getByRole("button", { name: /RubyOre/ }));
    await user.click(within(tree).getByRole("button", { name: /GlowOnStep/ }));
    // Abrir de nuevo un elemento ya abierto no duplica la pestaña.
    await user.click(within(tree).getByRole("button", { name: /RubyOre/ }));
    const editorTabs = screen.getByRole("tab", { name: "Resumen" }).closest<HTMLElement>("[role=tablist]")!;
    expect(within(editorTabs).getAllByRole("tab")).toHaveLength(3);

    await user.click(screen.getByRole("button", { name: "Cerrar RubyOre" }));
    expect(screen.getByRole("tab", { name: "GlowOnStep", selected: true })).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Cerrar GlowOnStep" }));
    expect(screen.getByRole("tab", { name: "Resumen", selected: true })).toBeInTheDocument();
  });

  it("el filtro busca por nombre y por nombre de registro", async () => {
    const user = userEvent.setup();
    renderApp();
    await openDemoProject(user);
    const tree = screen.getByRole("navigation", { name: "Elementos" });
    await user.type(screen.getByPlaceholderText("Filtrar elementos"), "glow_on");
    expect(within(tree).getAllByRole("button", { name: /GlowOnStep/ })).toHaveLength(1);
    expect(within(tree).queryByRole("button", { name: /RubyOre/ })).not.toBeInTheDocument();
    await user.clear(screen.getByPlaceholderText("Filtrar elementos"));
    await user.type(screen.getByPlaceholderText("Filtrar elementos"), "zzz");
    expect(within(tree).getByText("Ningún elemento coincide con el filtro.")).toBeInTheDocument();
  });

  it("los grupos del árbol se pliegan", async () => {
    const user = userEvent.setup();
    renderApp();
    await openDemoProject(user);
    const tree = screen.getByRole("navigation", { name: "Elementos" });
    await user.click(within(tree).getByRole("button", { name: /^Bloque/ }));
    expect(within(tree).getByRole("button", { name: /^Bloque/ })).toHaveAttribute("aria-expanded", "false");
    expect(within(tree).queryByRole("button", { name: /RubyOre/ })).not.toBeInTheDocument();
  });

  it("en inglés traduce los tipos de elemento", async () => {
    const user = userEvent.setup();
    renderApp("en");
    await user.click(await screen.findByRole("button", { name: "Open MCreator project" }));
    const tree = await screen.findByRole("navigation", { name: "Elements" });
    expect(within(tree).getByRole("button", { name: /^Block/ })).toBeInTheDocument();
    expect(within(tree).getByRole("button", { name: /^Custom code/ })).toBeInTheDocument();
  });
});

describe("paneles y atajos", () => {
  beforeEach(returningUser);

  it.each([
    ["sidebar", "b"],
    ["console", "j"],
    ["ai", "l"],
  ] as const)("Ctrl+%s oculta y vuelve a mostrar el panel %s", async (key, letter) => {
    const user = userEvent.setup();
    renderApp();
    await screen.findByRole("heading", { name: "Proyectos" });
    expect(panelButton(key)).toHaveAttribute("aria-pressed", "true");
    await user.keyboard(`{Control>}${letter}{/Control}`);
    await waitFor(() => expect(panelButton(key)).toHaveAttribute("aria-pressed", "false"));
    await user.keyboard(`{Control>}${letter}{/Control}`);
    await waitFor(() => expect(panelButton(key)).toHaveAttribute("aria-pressed", "true"));
  });

  it("la barra lateral cambia de sección y se pliega al tocar la sección activa", async () => {
    const user = userEvent.setup();
    renderApp();
    const nav = await screen.findByRole("navigation", { name: "Secciones" });
    await user.click(within(nav).getByRole("button", { name: "Base de conocimiento" }));
    expect(screen.getByRole("heading", { level: 2, name: "Base de conocimiento" })).toBeInTheDocument();
    await user.click(within(nav).getByRole("button", { name: "Base de conocimiento" }));
    await waitFor(() => expect(panelButton("sidebar")).toHaveAttribute("aria-pressed", "false"));
  });

  it("los atajos no reaccionan sin Ctrl ni con Ctrl+Shift", async () => {
    renderApp();
    await screen.findByRole("heading", { name: "Proyectos" });
    await act(async () => {
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "b" }));
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "b", ctrlKey: true, shiftKey: true }));
    });
    expect(panelButton("sidebar")).toHaveAttribute("aria-pressed", "true");
  });

  it("cambia todo a inglés desde la barra de estado y lo recuerda", async () => {
    const user = userEvent.setup();
    renderApp("es");
    await screen.findByRole("heading", { name: "Proyectos" });
    await user.click(screen.getByRole("button", { name: "English" }));
    expect(screen.getByRole("heading", { level: 1, name: "Projects" })).toBeInTheDocument();
    expect(document.documentElement.lang).toBe("en");
    expect(localStorage.getItem("mcgod:lang")).toBe("en");
  });
});
