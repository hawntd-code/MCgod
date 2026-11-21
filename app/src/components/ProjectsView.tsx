import { FolderOpen, LoaderCircle, Plus, X } from "lucide-react";
import { useI18n } from "../i18n";
import type { AppSettings, EngineError } from "../lib/engine";
import { delay, stagger } from "../lib/motion";
import { ErrorNotice } from "./ErrorNotice";

type Props = {
  settings: AppSettings;
  busy: boolean;
  error: EngineError | null;
  onPick: () => void;
  onOpen: (path: string) => void;
  onForget: (path: string) => void;
  onDismissError: () => void;
};

/** Pantalla de inicio cuando no hay proyecto abierto. */
export function ProjectsView({ settings, busy, error, onPick, onOpen, onForget, onDismissError }: Props) {
  const { m, lang } = useI18n();
  const p = m.projects;
  const date = new Intl.DateTimeFormat(lang === "es" ? "es-MX" : "en-US", { dateStyle: "medium", timeStyle: "short" });

  return (
    <section className="h-full overflow-auto bg-surface" aria-labelledby="projects-title">
      <div className="mx-auto max-w-[880px] px-[clamp(1.5rem,4vw,3rem)] py-12">
        <h1
          id="projects-title"
          className="animate-enter font-display text-[2.5rem] font-[800] leading-none tracking-[-0.025em] [font-stretch:118%]"
        >
          {p.title}
        </h1>

        <div className="mt-7 flex animate-enter flex-wrap items-center gap-3" style={delay(50)}>
          <button
            type="button"
            onClick={onPick}
            disabled={busy}
            className="inline-flex h-10 items-center gap-2 rounded-ui bg-brand px-4 font-semibold text-brand-ink transition-[filter,transform,opacity] duration-150 hover:brightness-110 active:translate-y-px active:brightness-95 disabled:cursor-wait disabled:opacity-70"
          >
            {busy ? (
              <LoaderCircle className="size-4 animate-spin" strokeWidth={2.25} aria-hidden />
            ) : (
              <FolderOpen className="size-4" strokeWidth={2.25} aria-hidden />
            )}
            {busy ? p.opening : p.open}
          </button>
          <button
            type="button"
            disabled
            title={p.newLater}
            className="inline-flex h-10 items-center gap-2 rounded-ui border border-line px-4 text-ink-muted disabled:cursor-not-allowed"
          >
            <Plus className="size-4" aria-hidden />
            {p.newProject}
          </button>
        </div>
        <p className="mt-3 animate-enter text-small text-ink-muted" style={delay(80)}>
          {p.openHint}
        </p>

        {error && (
          <div className="mt-6">
            <ErrorNotice error={error} onDismiss={onDismissError} />
          </div>
        )}

        <h2 className="mt-14 animate-enter font-display text-h3 font-[680] [font-stretch:110%]" style={delay(120)}>
          {p.recent}
        </h2>
        {settings.recent.length === 0 ? (
          <p className="mt-3 animate-enter text-small text-ink-muted" style={delay(150)}>
            {p.recentEmpty}
          </p>
        ) : (
          <ul className="mt-3 divide-y divide-line border-y border-line" aria-label={p.recent}>
            {settings.recent.map((r, i) => (
              <li key={r.path} className="group flex animate-enter items-center" style={stagger(i, { base: 150, step: 40 })}>
                <button
                  type="button"
                  onClick={() => onOpen(r.path)}
                  disabled={busy}
                  className="min-w-0 flex-1 px-2 py-3 text-left transition-colors duration-150 hover:bg-hover disabled:cursor-wait"
                >
                  <span className="block font-semibold text-ink">{r.name}</span>
                  <span className="mt-0.5 block text-small text-ink-muted">
                    <span className="font-mono">{r.modid}</span> · {p.openedAt(date.format(new Date(r.openedAt * 1000)))}
                  </span>
                  <span className="mt-0.5 block truncate font-mono text-label text-ink-faint">{r.path}</span>
                </button>
                <button
                  type="button"
                  onClick={() => onForget(r.path)}
                  aria-label={p.remove(r.name)}
                  title={p.remove(r.name)}
                  className="mr-1 grid size-8 shrink-0 place-items-center rounded-ui text-ink-faint opacity-0 transition-opacity duration-150 hover:bg-hover hover:text-ink focus-visible:opacity-100 group-hover:opacity-100"
                >
                  <X className="size-4" aria-hidden />
                </button>
              </li>
            ))}
          </ul>
        )}
      </div>
    </section>
  );
}
