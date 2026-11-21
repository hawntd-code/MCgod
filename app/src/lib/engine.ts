import { invoke, isTauri } from "@tauri-apps/api/core";

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

/** Minecraft moderno con NeoForge necesita Java 21 o superior. */
export const REQUIRED_JAVA = 21;
