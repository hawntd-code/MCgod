import { LANGS, useI18n } from "../i18n";
import type { Messages } from "../i18n/locales/es";
import { REQUIRED_JAVA, type EngineState, type WorkspaceSummary } from "../lib/engine";
import { generatorLabel } from "../lib/format";

export function javaStatus(engine: EngineState, s: Messages["status"]): { text: string; warn: boolean } {
  if (engine.status !== "ready") return { text: s.javaUnknown, warn: false };
  const java = engine.env.java;
  if (!java) return { text: s.javaMissing, warn: true };
  const ok = java.major != null && java.major >= REQUIRED_JAVA;
  return { text: s.java(java.version), warn: !ok };
}

export function StatusBar({
  engine,
  workspace,
  gradleRunning = false,
}: {
  engine: EngineState;
  workspace: WorkspaceSummary | null;
  gradleRunning?: boolean;
}) {
  const { m, lang, setLang } = useI18n();
  const java = javaStatus(engine, m.status);
  const version = engine.status === "ready" ? `v${engine.env.appVersion}` : "";
  const generator = workspace ? generatorLabel(workspace.generator) : null;

  return (
    <footer className="flex h-6 shrink-0 items-center gap-4 border-t border-line bg-ground px-3 text-label text-ink-faint">
      <span data-testid="status-project">{workspace ? workspace.modid : m.status.noProject}</span>
      <span data-testid="status-generator">{generator ?? m.status.noGenerator}</span>
      {workspace && <span className="tabular-nums">{m.status.elements(workspace.elements.length)}</span>}
      {gradleRunning && <span className="text-brand-bright">{m.status.gradleRunning}</span>}
      <span className="ml-auto">{m.status.mcpOff}</span>
      <span className={java.warn ? "text-warn" : undefined} data-testid="java-status">
        {java.text}
      </span>
      {version && <span className="tabular-nums">{version}</span>}

      <div role="group" aria-label={m.language.label} className="flex items-center gap-0.5">
        {LANGS.map((l) => (
          <button
            key={l}
            type="button"
            lang={l}
            aria-pressed={lang === l}
            aria-label={m.language[l]}
            title={m.language[l]}
            onClick={() => setLang(l)}
            className="h-5 rounded-[3px] px-1.5 font-medium uppercase tracking-wide transition-colors duration-150 hover:text-ink aria-pressed:bg-raised aria-pressed:text-ink"
          >
            {l}
          </button>
        ))}
      </div>
    </footer>
  );
}
