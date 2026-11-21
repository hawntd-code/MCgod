import { Hammer, LoaderCircle, PanelBottom, PanelLeft, PanelRight, Play, Square } from "lucide-react";
import { useI18n } from "../i18n";
import type { GradleTask } from "../lib/engine";
import { BrandMark } from "./BrandMark";

export type PanelKey = "sidebar" | "console" | "ai";

const toggles: { key: PanelKey; shortcut: string; Icon: typeof PanelLeft }[] = [
  { key: "sidebar", shortcut: "Ctrl+B", Icon: PanelLeft },
  { key: "console", shortcut: "Ctrl+J", Icon: PanelBottom },
  { key: "ai", shortcut: "Ctrl+L", Icon: PanelRight },
];

type Props = {
  open: Record<PanelKey, boolean>;
  onToggle: (key: PanelKey) => void;
  projectName: string | null;
  canRun: boolean;
  runningTask: GradleTask | null;
  onRun: (task: GradleTask) => void;
  onStop: () => void;
};

export function TopBar({ open, onToggle, projectName, canRun, runningTask, onRun, onStop }: Props) {
  const { m } = useI18n();
  const t = m.topBar;

  const reason = !projectName ? t.needProject : !canRun ? t.noGradle : runningTask ? t.busy : null;
  const busyIcon = <LoaderCircle className="size-3.5 animate-spin" aria-hidden />;

  return (
    <header className="flex h-11 shrink-0 items-center gap-3 border-b border-line bg-ground px-3">
      <div className="flex items-center gap-2 pr-2">
        <BrandMark className="size-5" />
        <span className="font-display text-[0.95rem] font-[750] tracking-[-0.01em] [font-stretch:118%]">MCgod</span>
      </div>

      <span
        data-testid="project-name"
        className={`flex h-7 max-w-[28ch] items-center truncate rounded-ui border border-line px-2.5 text-small ${projectName ? "font-medium text-ink" : "text-ink-muted"}`}
      >
        {projectName ?? t.noProject}
      </span>

      <div className="ml-auto flex items-center gap-1.5">
        <button
          type="button"
          disabled={reason !== null}
          onClick={() => onRun("build")}
          title={reason ?? t.buildHint}
          className="flex h-7 items-center gap-1.5 rounded-ui px-2.5 text-small font-medium text-ink-muted transition-colors duration-150 hover:bg-hover hover:text-ink disabled:cursor-not-allowed disabled:text-ink-faint disabled:hover:bg-transparent"
        >
          {runningTask === "build" ? busyIcon : <Hammer className="size-3.5" aria-hidden />}
          {runningTask === "build" ? t.building : t.build}
        </button>

        {runningTask ? (
          <button
            type="button"
            onClick={onStop}
            title={t.stopHint}
            className="flex h-7 items-center gap-1.5 rounded-ui border border-signal px-3 text-small font-semibold text-ink transition-colors duration-150 hover:bg-signal/20"
          >
            {runningTask === "runClient" ? busyIcon : <Square className="size-3 fill-current" aria-hidden />}
            {runningTask === "runClient" ? `${t.launching} · ${t.stop}` : t.stop}
          </button>
        ) : (
          <button
            type="button"
            disabled={reason !== null}
            onClick={() => onRun("runClient")}
            title={reason ?? t.playHint}
            className="flex h-7 items-center gap-1.5 rounded-ui bg-signal px-3 text-small font-semibold text-signal-ink transition-[filter,opacity] duration-150 hover:brightness-110 active:brightness-95 disabled:cursor-not-allowed disabled:opacity-45 disabled:hover:brightness-100"
          >
            <Play className="size-3.5 fill-current" aria-hidden />
            {t.play}
          </button>
        )}

        <span className="mx-2 h-5 w-px bg-line" aria-hidden />

        {toggles.map(({ key, shortcut, Icon }) => {
          const label = `${open[key] ? t.hide : t.show} ${t.panels[key]} (${shortcut})`;
          return (
            <button
              key={key}
              type="button"
              onClick={() => onToggle(key)}
              aria-pressed={open[key]}
              aria-label={label}
              title={label}
              data-panel={key}
              className="grid size-7 place-items-center rounded-ui text-ink-faint transition-colors duration-150 hover:bg-hover hover:text-ink aria-pressed:text-ink"
            >
              <Icon className="size-4" aria-hidden />
            </button>
          );
        })}
      </div>
    </header>
  );
}
