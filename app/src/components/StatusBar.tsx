import { REQUIRED_JAVA, type EngineState } from "../lib/engine";

function javaLabel(engine: EngineState): { text: string; warn: boolean } {
  if (engine.status !== "ready") return { text: "Java: —", warn: false };
  const java = engine.env.java;
  if (!java) return { text: "Java no encontrado", warn: true };
  const ok = java.major != null && java.major >= REQUIRED_JAVA;
  return { text: `Java ${java.version}`, warn: !ok };
}

export function StatusBar({ engine }: { engine: EngineState }) {
  const java = javaLabel(engine);
  const version = engine.status === "ready" ? `v${engine.env.appVersion}` : "";

  return (
    <footer className="flex h-6 shrink-0 items-center gap-4 border-t border-line bg-ground px-3 text-label text-ink-faint">
      <span>Sin proyecto</span>
      <span>Generador: ninguno</span>
      <span className="ml-auto">MCP apagado</span>
      <span className={java.warn ? "text-warn" : undefined}>{java.text}</span>
      {version && <span className="tabular-nums">{version}</span>}
    </footer>
  );
}
