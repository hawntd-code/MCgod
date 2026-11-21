import { invoke, isTauri } from "@tauri-apps/api/core";
import { demoDefinitions, DEMO_FILE, demoSummary } from "./demo";

export type JavaInfo = {
  version: string;
  major: number | null;
  vendor: string | null;
};

export type Environment = {
  appVersion: string;
  os: string;
  java: JavaInfo | null;
};

export type EngineState =
  | { status: "loading" }
  | { status: "ready"; env: Environment }
  | { status: "offline" }
  | { status: "error"; message: string };

export type RecentWorkspace = { path: string; name: string; modid: string; openedAt: number };
export type AppSettings = { onboardingDone: boolean; recent: RecentWorkspace[] };

export type GeneratorId = { id: string; flavor: string; minecraft: string | null };

export type ElementSummary = {
  name: string;
  type: string;
  registryName: string | null;
  folder: string | null;
  compiles: boolean;
  lockedCode: boolean;
  hasDefinition: boolean;
};

export type WorkspaceSummary = {
  root: string;
  file: string;
  modid: string;
  name: string;
  version: string | null;
  author: string | null;
  description: string | null;
  generator: GeneratorId | null;
  mcreatorVersion: number;
  elements: ElementSummary[];
  variables: number;
  sounds: number;
  tags: number;
  languages: string[];
};

export type ElementDefinition = {
  _fv: number;
  _type: string;
  definition: Record<string, unknown>;
};

/** Error del motor con un código estable que la interfaz traduce. */
export type EngineError = { code: string; detail: string };

export function toEngineError(err: unknown): EngineError {
  if (err && typeof err === "object" && "code" in err) {
    const e = err as { code: unknown; detail?: unknown };
    return { code: String(e.code), detail: e.detail == null ? "" : String(e.detail) };
  }
  return { code: "internal", detail: String(err) };
}

/** Minecraft moderno con NeoForge necesita Java 21 o superior. */
export const REQUIRED_JAVA = 21;

/** Pide al motor en Rust la información del entorno (versión, sistema, Java). */
export async function readEnvironment(): Promise<EngineState> {
  if (!isTauri()) return { status: "offline" };
  try {
    const env = await invoke<Environment>("environment");
    return { status: "ready", env };
  } catch (err) {
    return { status: "error", message: String(err) };
  }
}

/** Operaciones del motor que usa la interfaz. */
export interface Backend {
  settings(): Promise<AppSettings>;
  finishOnboarding(): Promise<AppSettings>;
  /** Abre el selector de archivos. Devuelve `null` si la persona cancela. */
  pickWorkspace(titles: { title: string; filter: string }): Promise<string | null>;
  openWorkspace(path: string): Promise<WorkspaceSummary>;
  closeWorkspace(): Promise<void>;
  readElement(name: string): Promise<ElementDefinition>;
  forgetRecent(path: string): Promise<AppSettings>;
}

const tauriBackend: Backend = {
  settings: () => invoke<AppSettings>("app_settings"),
  finishOnboarding: () => invoke<AppSettings>("finish_onboarding"),
  async pickWorkspace({ title, filter }) {
    const { open } = await import("@tauri-apps/plugin-dialog");
    const picked = await open({ title, multiple: false, directory: false, filters: [{ name: filter, extensions: ["mcreator"] }] });
    return typeof picked === "string" ? picked : null;
  },
  openWorkspace: (path) => invoke<WorkspaceSummary>("open_workspace", { path }),
  closeWorkspace: () => invoke<void>("close_workspace"),
  readElement: (name) => invoke<ElementDefinition>("read_element", { name }),
  forgetRecent: (path) => invoke<AppSettings>("forget_recent", { path }),
};

/**
 * Sustituto para cuando la interfaz corre en un navegador (desarrollo y pruebas).
 * Guarda los ajustes en localStorage. Con `?demo` en la URL (o en las pruebas) ofrece
 * un proyecto de ejemplo; sin eso, abrir proyectos solo funciona en la app de escritorio.
 */
export function createBrowserBackend(demo: boolean): Backend {
  const KEY = "mcgod:browser-settings";
  const read = (): AppSettings => {
    try {
      const raw = localStorage.getItem(KEY);
      if (raw) return JSON.parse(raw) as AppSettings;
    } catch {
      // Sin almacenamiento: ajustes solo en memoria.
    }
    return { onboardingDone: false, recent: [] };
  };
  let memory = read();
  const write = (next: AppSettings) => {
    memory = next;
    try {
      localStorage.setItem(KEY, JSON.stringify(next));
    } catch {
      // Ignorado: el estado sigue en memoria.
    }
    return next;
  };
  const desktopOnly = (): EngineError => ({ code: "desktopOnly", detail: "" });

  return {
    settings: async () => (memory = read()),
    finishOnboarding: async () => write({ ...memory, onboardingDone: true }),
    pickWorkspace: async () => {
      if (!demo) throw desktopOnly();
      return DEMO_FILE;
    },
    openWorkspace: async (path) => {
      if (!demo || path !== DEMO_FILE) throw desktopOnly();
      const recent = [
        { path: DEMO_FILE, name: demoSummary.name, modid: demoSummary.modid, openedAt: Math.floor(Date.now() / 1000) },
        ...memory.recent.filter((r) => r.path !== DEMO_FILE),
      ].slice(0, 8);
      write({ ...memory, recent });
      return structuredClone(demoSummary);
    },
    closeWorkspace: async () => {},
    readElement: async (name) => {
      const def = demoDefinitions[name];
      if (!def) {
        const known = demoSummary.elements.some((e) => e.name === name);
        throw { code: known ? "noDefinition" : "unknownElement", detail: name } satisfies EngineError;
      }
      return structuredClone(def);
    },
    forgetRecent: async (path) => write({ ...memory, recent: memory.recent.filter((r) => r.path !== path) }),
  };
}

function demoEnabled(): boolean {
  if (import.meta.env.MODE === "test") return true;
  return import.meta.env.DEV && typeof location !== "undefined" && new URLSearchParams(location.search).has("demo");
}

export function getBackend(): Backend {
  return isTauri() ? tauriBackend : createBrowserBackend(demoEnabled());
}
