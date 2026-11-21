import { Channel, invoke, isTauri } from "@tauri-apps/api/core";
import { demoCodegenReport, demoDefinitions, DEMO_FILE, demoGradleOutput, demoSummary } from "./demo";

export type JavaInfo = {
  version: string;
  major: number | null;
  vendor: string | null;
  home: string | null;
};

export type GradleTask = "build" | "runClient";
export type GradleOutcome = "success" | "failed" | "cancelled";

/** Por qué un elemento no se generó. */
export type SkipReason =
  | { kind: "locked" | "unsupported" | "notInGenerator" }
  | { kind: "noDefinition"; detail: string };

export type CodegenNoteCode =
  | "soundsPending"
  | "variablesPending"
  | "tagsPending"
  | "procedurePending"
  | "noImportIndex"
  | "invalidJson"
  | "langFormatPending"
  | "unknownElement";

export type CodegenReport = {
  files: number;
  written: number;
  unchanged: number;
  generated: string[];
  skipped: { name: string; elementType: string; reason: SkipReason }[];
  notes: { code: CodegenNoteCode; detail: string | null }[];
  /** Plantillas que fallaron: si hay alguna, no se escribió nada. */
  errors: string[];
};

/** Avance de la generación de código que precede a cada compilación. */
export type CodegenEvent = {
  kind: "codegen";
  stage: "indexing" | "generating" | "generated";
  report?: CodegenReport;
};

/** Una línea de Gradle, o un aviso de MCgod que la consola escribe en el idioma de la persona. */
export type OutputLine =
  | { stream: "stdout" | "stderr"; text: string }
  | { stream: "mcgod"; text: string; codegen: CodegenEvent };

export type GradleEvent =
  | { kind: "lines"; lines: OutputLine[] }
  | { kind: "finished"; outcome: GradleOutcome; exitCode: number | null; durationMs: number }
  | CodegenEvent;

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
  hasGradle: boolean;
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
  /** Lanza Gradle. Responde al arrancar; la salida y el final llegan por `onEvent`. */
  runGradle(task: GradleTask, onEvent: (event: GradleEvent) => void): Promise<void>;
  stopGradle(): Promise<void>;
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
  runGradle: async (task, onEvent) => {
    const channel = new Channel<GradleEvent>();
    channel.onmessage = onEvent;
    await invoke<void>("gradle_run", { task, onEvent: channel });
  },
  stopGradle: () => invoke<void>("gradle_stop"),
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
  let gradleTimer: ReturnType<typeof setTimeout> | null = null;
  let gradleFinish: (() => void) | null = null;

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
    // Simulación para la vista del navegador: repite una salida real de Gradle, más rápido.
    runGradle: async (task, onEvent) => {
      if (!demo) throw desktopOnly();
      if (gradleTimer) throw { code: "gradleBusy", detail: task } satisfies EngineError;
      const lines = demoGradleOutput(task);
      const started = Date.now();
      // Como la app: primero se genera el código del mod.
      onEvent({ kind: "codegen", stage: "generating" });
      onEvent({ kind: "codegen", stage: "generated", report: structuredClone(demoCodegenReport) });
      let i = 0;
      const step = () => {
        const chunk = lines.slice(i, i + 6);
        i += chunk.length;
        if (chunk.length) onEvent({ kind: "lines", lines: chunk.map((text) => ({ stream: "stdout", text })) });
        if (i < lines.length) {
          gradleTimer = setTimeout(step, 120);
        } else if (task === "runClient") {
          // Como el juego real: sigue abierto hasta que se detiene.
          gradleTimer = setTimeout(() => {}, 2_147_483_647);
        } else {
          gradleTimer = null;
          gradleFinish = null;
          onEvent({ kind: "finished", outcome: "success", exitCode: 0, durationMs: Date.now() - started });
        }
      };
      gradleFinish = () => onEvent({ kind: "finished", outcome: "cancelled", exitCode: null, durationMs: Date.now() - started });
      gradleTimer = setTimeout(step, 50);
    },
    stopGradle: async () => {
      if (gradleTimer) clearTimeout(gradleTimer);
      gradleTimer = null;
      gradleFinish?.();
      gradleFinish = null;
    },
  };
}

function demoEnabled(): boolean {
  if (import.meta.env.MODE === "test") return true;
  return import.meta.env.DEV && typeof location !== "undefined" && new URLSearchParams(location.search).has("demo");
}

export function getBackend(): Backend {
  return isTauri() ? tauriBackend : createBrowserBackend(demoEnabled());
}
